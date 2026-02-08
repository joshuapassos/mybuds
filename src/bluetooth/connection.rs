use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::Duration;

use anyhow::{Context, Result};
use bluer::Address;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::protocol::HuaweiSppPacket;

// Bluetooth socket constants (from Linux kernel headers)
const AF_BLUETOOTH: libc::c_int = 31;
const BTPROTO_RFCOMM: libc::c_int = 3;

/// sockaddr_rc — RFCOMM socket address (from <bluetooth/rfcomm.h>)
#[repr(C)]
struct SockaddrRc {
    rc_family: u16,
    rc_bdaddr: [u8; 6], // Bluetooth address in little-endian
    rc_channel: u8,
}

/// RFCOMM connection to a device.
/// Uses raw blocking sockets for connect (like Python/OpenFreebuds),
/// then wraps in tokio async I/O for the read/write phase.
pub struct RfcommConnection {
    stream: UnixStream,
}

impl RfcommConnection {
    /// Connect to a device via RFCOMM on the given channel.
    /// Uses a blocking connect (matching Python's socket.connect behavior)
    /// to ensure the RFCOMM DLC handshake completes before returning.
    pub async fn connect(address: Address, channel: u8) -> Result<Self> {
        info!("Connecting to {} on RFCOMM channel {}", address, channel);

        // Do the blocking connect in a spawn_blocking context
        // This matches Python's: sock = socket.socket(AF_BLUETOOTH, SOCK_STREAM, BTPROTO_RFCOMM)
        //                         sock.settimeout(2)
        //                         sock.connect((address, port))
        let addr_bytes = address.0; // [u8; 6] in big-endian

        let raw_fd = tokio::task::spawn_blocking(move || -> Result<OwnedFd> {
            unsafe {
                // Create RFCOMM socket (blocking mode, like Python)
                let fd = libc::socket(AF_BLUETOOTH, libc::SOCK_STREAM, BTPROTO_RFCOMM);
                if fd < 0 {
                    anyhow::bail!(
                        "Failed to create RFCOMM socket: {}",
                        std::io::Error::last_os_error()
                    );
                }

                // Set 5-second timeout for connect and I/O
                let timeout = libc::timeval {
                    tv_sec: 5,
                    tv_usec: 0,
                };
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_SNDTIMEO,
                    &timeout as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::timeval>() as libc::socklen_t,
                );
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO,
                    &timeout as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::timeval>() as libc::socklen_t,
                );

                // Build sockaddr_rc with address in little-endian (BlueZ convention)
                let mut rc_bdaddr = addr_bytes;
                rc_bdaddr.reverse(); // Big-endian -> little-endian for BlueZ
                let addr = SockaddrRc {
                    rc_family: AF_BLUETOOTH as u16,
                    rc_bdaddr,
                    rc_channel: channel,
                };

                // Blocking connect — waits for full RFCOMM DLC handshake
                let ret = libc::connect(
                    fd,
                    &addr as *const _ as *const libc::sockaddr,
                    std::mem::size_of::<SockaddrRc>() as libc::socklen_t,
                );
                if ret < 0 {
                    let err = std::io::Error::last_os_error();
                    libc::close(fd);
                    anyhow::bail!("RFCOMM connect failed: {}", err);
                }

                // Set non-blocking for tokio async I/O
                let flags = libc::fcntl(fd, libc::F_GETFL);
                libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

                Ok(OwnedFd::from_raw_fd(fd))
            }
        })
        .await
        .context("spawn_blocking panicked")?
        .context("RFCOMM connect")?;

        // Wrap in tokio UnixStream for async I/O
        // (UnixStream is just an AsyncFd<OwnedFd> wrapper — works for any stream socket)
        let std_stream =
            unsafe { std::os::unix::net::UnixStream::from_raw_fd(raw_fd.as_raw_fd()) };
        // Prevent double-close: forget the OwnedFd since std_stream now owns it
        std::mem::forget(raw_fd);
        let stream = UnixStream::from_std(std_stream)?;

        info!(
            "Connected to {} on RFCOMM channel {} (blocking connect OK)",
            address, channel
        );
        Ok(Self { stream })
    }

    /// Split into read/write tasks. Returns a receiver for incoming packets
    /// and a sender for outgoing packets.
    pub fn into_split(
        self,
    ) -> (
        mpsc::Receiver<HuaweiSppPacket>,
        mpsc::Sender<HuaweiSppPacket>,
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let (read_half, write_half) = tokio::io::split(self.stream);
        let (incoming_tx, incoming_rx) = mpsc::channel::<HuaweiSppPacket>(64);
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<HuaweiSppPacket>(32);

        let read_task = tokio::spawn(recv_loop(read_half, incoming_tx));
        let write_task = tokio::spawn(send_loop(write_half, outgoing_rx));

        (incoming_rx, outgoing_tx, read_task, write_task)
    }
}

async fn recv_loop(
    mut reader: tokio::io::ReadHalf<UnixStream>,
    tx: mpsc::Sender<HuaweiSppPacket>,
) {
    let mut buf = [0u8; 1024];

    loop {
        // Read header (4 bytes: magic + length(2) + reserved)
        match reader.read(&mut buf[..4]).await {
            Ok(0) => {
                info!("RFCOMM connection closed (EOF)");
                return;
            }
            Ok(n) if n < 4 => {
                // Try to read remaining header bytes
                let mut total = n;
                while total < 4 {
                    match reader.read(&mut buf[total..4]).await {
                        Ok(0) => return,
                        Ok(m) => total += m,
                        Err(e) => {
                            error!("RFCOMM read error: {}", e);
                            return;
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("RFCOMM read error: {}", e);
                return;
            }
        }

        // Check magic byte
        if buf[0] != 0x5A {
            warn!("Invalid magic byte: 0x{:02X}, skipping", buf[0]);
            continue;
        }

        // Parse length
        let length = u16::from_be_bytes([buf[1], buf[2]]) as usize;
        if length < 3 || length > 1000 {
            warn!("Invalid packet length: {}, skipping", length);
            continue;
        }

        // Read remaining body + CRC (length - 1 bytes for body after reserved byte, + 2 for CRC)
        let remaining = length - 1 + 2; // body (without the 0x00 byte already read) + CRC
        if 4 + remaining > buf.len() {
            warn!("Packet too large: {}", 4 + remaining);
            continue;
        }

        let mut total_read = 0;
        while total_read < remaining {
            match reader.read(&mut buf[4 + total_read..4 + remaining]).await {
                Ok(0) => {
                    info!("RFCOMM connection closed during read");
                    return;
                }
                Ok(n) => total_read += n,
                Err(e) => {
                    error!("RFCOMM read error: {}", e);
                    return;
                }
            }
        }

        let packet_data = &buf[..4 + remaining];
        match HuaweiSppPacket::from_bytes(packet_data) {
            Ok(pkt) => {
                debug!("RX: {}", pkt);
                if tx.send(pkt).await.is_err() {
                    info!("Packet channel closed, stopping recv loop");
                    return;
                }
            }
            Err(e) => {
                warn!("Failed to parse packet: {}", e);
            }
        }
    }
}

async fn send_loop(
    mut writer: tokio::io::WriteHalf<UnixStream>,
    mut rx: mpsc::Receiver<HuaweiSppPacket>,
) {
    while let Some(pkt) = rx.recv().await {
        let bytes = pkt.to_bytes();
        debug!("TX: {}", pkt);
        if let Err(e) = writer.write_all(&bytes).await {
            error!("RFCOMM write error: {}", e);
            return;
        }
        if let Err(e) = writer.flush().await {
            error!("RFCOMM flush error: {}", e);
            return;
        }
    }
    info!("Outgoing channel closed, stopping send loop");
}
