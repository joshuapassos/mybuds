use anyhow::Result;
use bluer::{Address, Session};
use tracing::{debug, info};

/// A discovered/paired Bluetooth device.
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: Address,
    pub paired: bool,
    pub connected: bool,
}

/// List paired Bluetooth devices, optionally filtering by known device names.
pub async fn list_paired_devices(filter_known: bool) -> Result<Vec<BluetoothDevice>> {
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let devices = adapter.device_addresses().await?;
    let mut result = Vec::new();

    for addr in devices {
        let device = adapter.device(addr)?;
        let name = device.name().await?.unwrap_or_default();
        let paired = device.is_paired().await?;
        let connected = device.is_connected().await?;

        if !paired {
            continue;
        }

        if filter_known && !is_known_device(&name) {
            continue;
        }

        debug!("Found device: {} ({}), connected={}", name, addr, connected);
        result.push(BluetoothDevice {
            name,
            address: addr,
            paired,
            connected,
        });
    }

    info!("Found {} paired devices", result.len());
    Ok(result)
}

/// Check if a device name matches a known supported device.
pub fn is_known_device(name: &str) -> bool {
    name.starts_with("HUAWEI Free")
        || name.starts_with("HUAWEI FreeClip")
        || name.starts_with("HONOR Earbuds")
        || name.starts_with("HUAWEI FreeLace")
}
