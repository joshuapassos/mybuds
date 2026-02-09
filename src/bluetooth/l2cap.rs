use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;

use anyhow::{Context, Result};
use bluer::Address;
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::protocol::aap::AapPacket;
use crate::protocol::HuaweiSppPacket;

const AF_BLUETOOTH: libc::c_int = 31;
const BTPROTO_L2CAP: libc::c_int = 0;

/// sockaddr_l2 for L2CAP sockets (from <bluetooth/l2cap.h>)
#[repr(C)]
struct SockaddrL2 {
    l2_family: u16,
    l2_psm: u16,        // PSM in little-endian
    l2_bdaddr: [u8; 6], // BT address in little-endian
    l2_cid: u16,
    l2_bdaddr_type: u8,
}

/// L2CAP connection to a device (SEQPACKET — preserves message boundaries).
pub struct L2capConnection {
    fd: Arc<AsyncFd<OwnedFd>>,
}

impl L2capConnection {
    /// Connect to a device via L2CAP on the given PSM.
    pub async fn connect(address: Address, psm: u16) -> Result<Self> {
        info!("Connecting to {} on L2CAP PSM 0x{:04X}", address, psm);

        let addr_bytes = address.0;

        let raw_fd = tokio::task::spawn_blocking(move || -> Result<OwnedFd> {
            unsafe {
                let fd = libc::socket(AF_BLUETOOTH, libc::SOCK_SEQPACKET, BTPROTO_L2CAP);
                if fd < 0 {
                    anyhow::bail!(
                        "Failed to create L2CAP socket: {}",
                        std::io::Error::last_os_error()
                    );
                }

                // 10-second timeout for connect and I/O
                let timeout = libc::timeval {
                    tv_sec: 10,
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

                // Build sockaddr_l2 with address in little-endian
                let mut l2_bdaddr = addr_bytes;
                l2_bdaddr.reverse();

                let addr = SockaddrL2 {
                    l2_family: AF_BLUETOOTH as u16,
                    l2_psm: psm.to_le(),
                    l2_bdaddr,
                    l2_cid: 0,
                    l2_bdaddr_type: 0, // BR/EDR
                };

                let ret = libc::connect(
                    fd,
                    &addr as *const _ as *const libc::sockaddr,
                    std::mem::size_of::<SockaddrL2>() as libc::socklen_t,
                );
                if ret < 0 {
                    let err = std::io::Error::last_os_error();
                    libc::close(fd);
                    anyhow::bail!("L2CAP connect failed: {}", err);
                }

                // Set non-blocking for tokio
                let flags = libc::fcntl(fd, libc::F_GETFL);
                libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

                Ok(OwnedFd::from_raw_fd(fd))
            }
        })
        .await
        .context("spawn_blocking panicked")?
        .context("L2CAP connect")?;

        let async_fd = AsyncFd::new(raw_fd)?;

        info!(
            "Connected to {} on L2CAP PSM 0x{:04X} (blocking connect OK)",
            address, psm
        );
        Ok(Self {
            fd: Arc::new(async_fd),
        })
    }

    /// Perform AAP protocol initialization (handshake + feature flags + notification subscription).
    /// Must be called before `into_split()`.
    pub async fn initialize(&self) -> Result<()> {
        let mut buf = [0u8; 1024];

        // Send handshake
        self.send_raw(&AapPacket::handshake()).await?;

        // Read handshake response
        let n = self.recv_raw(&mut buf).await?;
        debug!("Handshake response: {} bytes", n);

        // Send feature flags (enables conversational awareness etc.)
        self.send_raw(&AapPacket::feature_flags()).await?;

        // Subscribe to all notifications
        self.send_raw(&AapPacket::request_notifications()).await?;

        info!("AAP initialization complete");
        Ok(())
    }

    async fn send_raw(&self, data: &[u8]) -> Result<()> {
        loop {
            let mut guard = self.fd.writable().await?;
            match guard.try_io(|inner| {
                let n = unsafe {
                    libc::send(
                        inner.get_ref().as_raw_fd(),
                        data.as_ptr() as *const libc::c_void,
                        data.len(),
                        0,
                    )
                };
                if n < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }) {
                Ok(Ok(_)) => return Ok(()),
                Ok(Err(e)) => return Err(e.into()),
                Err(_would_block) => continue,
            }
        }
    }

    async fn recv_raw(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let mut guard = self.fd.readable().await?;
            match guard.try_io(|inner| {
                let n = unsafe {
                    libc::recv(
                        inner.get_ref().as_raw_fd(),
                        buf.as_mut_ptr() as *mut libc::c_void,
                        buf.len(),
                        0,
                    )
                };
                if n < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }) {
                Ok(result) => return Ok(result?),
                Err(_would_block) => continue,
            }
        }
    }

    /// Split into read/write tasks. Returns same tuple as RfcommConnection::into_split().
    pub fn into_split(
        self,
    ) -> (
        mpsc::Receiver<HuaweiSppPacket>,
        mpsc::Sender<HuaweiSppPacket>,
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let (incoming_tx, incoming_rx) = mpsc::channel::<HuaweiSppPacket>(64);
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<HuaweiSppPacket>(32);

        let read_fd = self.fd.clone();
        let write_fd = self.fd;

        let read_task = tokio::spawn(aap_recv_loop(read_fd, incoming_tx));
        let write_task = tokio::spawn(aap_send_loop(write_fd, outgoing_rx));

        (incoming_rx, outgoing_tx, read_task, write_task)
    }
}

/// Receive one L2CAP packet (SEQPACKET preserves message boundaries).
async fn recv_l2cap(fd: &AsyncFd<OwnedFd>, buf: &mut [u8]) -> std::io::Result<usize> {
    loop {
        let mut guard = fd.readable().await?;
        match guard.try_io(|inner| {
            let n = unsafe {
                libc::recv(
                    inner.get_ref().as_raw_fd(),
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    0,
                )
            };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }) {
            Ok(result) => return result,
            Err(_would_block) => continue,
        }
    }
}

/// Send one L2CAP packet.
async fn send_l2cap(fd: &AsyncFd<OwnedFd>, data: &[u8]) -> std::io::Result<()> {
    loop {
        let mut guard = fd.writable().await?;
        match guard.try_io(|inner| {
            let n = unsafe {
                libc::send(
                    inner.get_ref().as_raw_fd(),
                    data.as_ptr() as *const libc::c_void,
                    data.len(),
                    0,
                )
            };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }) {
            Ok(result) => return result,
            Err(_would_block) => continue,
        }
    }
}

/// Read loop: receive AAP packets → convert to HuaweiSppPacket → send to handlers.
async fn aap_recv_loop(fd: Arc<AsyncFd<OwnedFd>>, tx: mpsc::Sender<HuaweiSppPacket>) {
    let mut buf = [0u8; 2048];

    loop {
        match recv_l2cap(&fd, &mut buf).await {
            Ok(0) => {
                info!("L2CAP connection closed (EOF)");
                return;
            }
            Ok(n) => {
                let data = &buf[..n];
                debug!(
                    "AAP RX: {} bytes [{:02x?}...]",
                    n,
                    &data[..n.min(12)]
                );

                if n < 5 {
                    warn!("AAP packet too short: {} bytes", n);
                    continue;
                }

                if let Some(aap) = AapPacket::from_bytes(data) {
                    let handler_pkt = aap.to_handler_packet();
                    debug!("AAP → handler: {}", handler_pkt);

                    if tx.send(handler_pkt).await.is_err() {
                        info!("Handler channel closed, stopping AAP recv loop");
                        return;
                    }
                } else {
                    warn!("Failed to parse AAP packet ({} bytes)", n);
                }
            }
            Err(e) => {
                error!("L2CAP read error: {}", e);
                return;
            }
        }
    }
}

/// Write loop: receive HuaweiSppPacket from handlers → convert to AAP → send over L2CAP.
async fn aap_send_loop(fd: Arc<AsyncFd<OwnedFd>>, mut rx: mpsc::Receiver<HuaweiSppPacket>) {
    while let Some(pkt) = rx.recv().await {
        if let Some(bytes) = AapPacket::from_handler_packet(&pkt) {
            debug!("AAP TX: {} bytes", bytes.len());
            if let Err(e) = send_l2cap(&fd, &bytes).await {
                error!("L2CAP write error: {}", e);
                return;
            }
        } else {
            warn!(
                "Cannot convert handler packet to AAP: {:02X}{:02X}",
                pkt.command_id[0], pkt.command_id[1]
            );
        }
    }
    info!("Outgoing channel closed, stopping AAP send loop");
}
