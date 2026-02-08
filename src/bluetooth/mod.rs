pub mod connection;
pub mod scanner;

use std::time::Duration;

use anyhow::Result;
use bluer::Address;
use tracing::{info, warn};

use crate::device::models::DeviceProfile;
use crate::device::DeviceManager;
use connection::RfcommConnection;

/// Reset the BT link to clear stale RFCOMM state.
/// Disconnects and reconnects the device to force BlueZ to clean up.
async fn reset_bt_link(address: Address) -> anyhow::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let device = adapter.device(address)?;

    if device.is_connected().await.unwrap_or(false) {
        info!("Disconnecting device to reset BT link...");
        device.disconnect().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    info!("Reconnecting device...");
    device.connect().await?;
    // Wait longer for RFCOMM/SPP profiles to re-register after reconnect
    tokio::time::sleep(Duration::from_secs(5)).await;
    info!("BT link reset complete");
    Ok(())
}

/// High-level Bluetooth manager that orchestrates connection and packet routing.
pub struct BluetoothManager {
    device_manager: DeviceManager,
    address: Address,
    spp_port: u8,
    prop_rx: Option<tokio::sync::mpsc::Receiver<(String, String, String)>>,
}

impl BluetoothManager {
    pub fn new(
        address: Address,
        profile: DeviceProfile,
        props: crate::device::handler::PropertyStore,
        prop_rx: tokio::sync::mpsc::Receiver<(String, String, String)>,
    ) -> Self {
        let spp_port = profile.spp_port as u8;
        let device_manager = DeviceManager::new(profile.handlers, props);

        Self {
            device_manager,
            address,
            spp_port,
            prop_rx: Some(prop_rx),
        }
    }

    /// Get property store.
    pub fn props(&self) -> crate::device::handler::PropertyStore {
        self.device_manager.props()
    }

    /// Run the connection loop: connect, init handlers, route packets.
    /// Returns when the connection is lost.
    pub async fn run(&mut self) -> Result<()> {
        // Reset channels so run() can be called again after reconnect
        self.device_manager.reset_channels();

        // Try the configured channel first, then fallback to the other common one
        let alt = if self.spp_port == 16 { 1 } else { 16 };
        let channels = [self.spp_port, alt];

        let mut conn_result = None;
        for &ch in &channels {
            match RfcommConnection::connect(self.address, ch).await {
                Ok(c) => {
                    conn_result = Some(c);
                    break;
                }
                Err(e) => {
                    warn!("RFCOMM channel {} failed: {}", ch, e);
                }
            }
        }

        let conn = match conn_result {
            Some(c) => c,
            None => anyhow::bail!("No RFCOMM channel worked (tried {:?})", &channels),
        };

        let (mut incoming_rx, outgoing_tx, read_task, write_task) = conn.into_split();

        // Connect the device manager's outgoing packets to the RFCOMM write channel
        let mut dm_packet_rx = self.device_manager.take_packet_rx().unwrap();
        let outgoing_tx_clone = outgoing_tx.clone();
        let forward_task = tokio::spawn(async move {
            while let Some(pkt) = dm_packet_rx.recv().await {
                if outgoing_tx_clone.send(pkt).await.is_err() {
                    break;
                }
            }
        });

        // Initialize all handlers — abort if connection dies during init
        if let Err(e) = self.device_manager.init_handlers().await {
            warn!("Handler init failed: {}", e);
            forward_task.abort();
            read_task.abort();
            write_task.abort();
            return Err(e);
        }

        // Take prop_rx for this run (will be None on reconnect if not reset)
        let mut prop_rx = self.prop_rx.take();

        // Route incoming packets and property changes
        loop {
            tokio::select! {
                pkt = incoming_rx.recv() => {
                    match pkt {
                        Some(packet) => {
                            self.device_manager.handle_packet(&packet).await;
                        }
                        None => break, // Connection lost
                    }
                }
                change = async {
                    match prop_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some((group, prop, value)) = change {
                        info!("UI property change: {}.{} = {}", group, prop, value);
                        if let Err(e) = self.device_manager.set_property(&group, &prop, &value).await {
                            warn!("Failed to set property: {}", e);
                        }
                    }
                }
            }
        }

        // Put prop_rx back for potential reconnect
        self.prop_rx = prop_rx;

        info!("Connection lost, cleaning up");
        forward_task.abort();
        read_task.abort();
        write_task.abort();

        // Clear property store so UI shows disconnected state
        self.device_manager.clear_props().await;

        Ok(())
    }

    /// Run with auto-reconnect. Retries on disconnect with exponential backoff.
    /// After repeated failures, resets the BT link to clear stale RFCOMM state.
    pub async fn run_with_reconnect(&mut self) {
        let mut backoff = Duration::from_secs(2);
        let max_backoff = Duration::from_secs(30);
        let mut failures = 0u32;

        loop {
            // After 3 consecutive failures, reset the BT link
            if failures >= 3 {
                warn!("Multiple connection failures, resetting Bluetooth link");
                if let Err(e) = reset_bt_link(self.address).await {
                    warn!("BT link reset failed: {}", e);
                }
                failures = 0;
                backoff = Duration::from_secs(3);
            }

            match self.run().await {
                Ok(()) => {
                    info!("Connection ended normally");
                    backoff = Duration::from_secs(2);
                    failures = 0;
                }
                Err(e) => {
                    warn!("Connection error: {}", e);
                    failures += 1;
                }
            }

            info!("Reconnecting in {:?}...", backoff);
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(max_backoff);
        }
    }
}
