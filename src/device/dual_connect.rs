use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Dual connect command IDs for execute actions.
pub mod commands {
    pub const CONNECT: u8 = 1;
    pub const DISCONNECT: u8 = 2;
    pub const UNPAIR: u8 = 3;
    pub const ENABLE_AUTO: u8 = 4;
    pub const DISABLE_AUTO: u8 = 5;
}

/// A dual-connected device entry.
#[derive(Debug, Clone)]
struct DualConnectDevice {
    mac: String,
    name: String,
    connected: bool,
    playing: bool,
    preferred: bool,
    auto_connect: bool,
}

impl DualConnectDevice {
    fn from_packet(packet: &HuaweiSppPacket, _with_auto: bool) -> Option<Self> {
        let mac_bytes = packet.find_param(4);
        if mac_bytes.len() < 6 {
            return None;
        }
        let mac: String = mac_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let name = packet
            .find_param(5)
            .iter()
            .copied()
            .take_while(|&b| b != 0)
            .collect::<Vec<_>>();
        let name = String::from_utf8(name).unwrap_or_default();

        let status = packet.find_param(6);
        let connected = status.first().copied().unwrap_or(0) == 1;
        let playing = status.get(1).copied().unwrap_or(0) == 1;
        let preferred = packet.find_param(7).first().copied().unwrap_or(0) == 1;
        let auto_connect = packet.find_param(8).first().copied().unwrap_or(0) == 1;

        Some(Self {
            mac,
            name,
            connected,
            playing,
            preferred,
            auto_connect,
        })
    }

    fn to_json_value(&self) -> String {
        format!(
            r#"{{"name":"{}","connected":{},"playing":{},"auto_connect":{}}}"#,
            self.name, self.connected, self.playing, self.auto_connect
        )
    }
}

/// Dual connect handler: manages multi-device connections.
pub struct DualConnectHandler {
    with_auto_connect: bool,
    pending_devices: HashMap<i8, DualConnectDevice>,
    devices_count: i8,
}

impl DualConnectHandler {
    pub fn new(with_auto_connect: bool) -> Self {
        Self {
            with_auto_connect,
            pending_devices: HashMap::new(),
            devices_count: 0,
        }
    }
}

impl Default for DualConnectHandler {
    fn default() -> Self {
        Self::new(true)
    }
}

#[async_trait]
impl DeviceHandler for DualConnectHandler {
    fn handler_id(&self) -> &'static str {
        "dual_connect"
    }

    fn commands(&self) -> &[CommandId] {
        &[
            CMD_DUAL_CONNECT_ENUMERATE,
            CMD_DUAL_CONNECT_CHANGE_EVENT,
            CMD_DUAL_CONNECT_ENABLED_READ,
        ]
    }

    fn ignore_commands(&self) -> &[CommandId] {
        &[
            CMD_DUAL_CONNECT_PREFERRED_WRITE,
            CMD_DUAL_CONNECT_EXECUTE,
            CMD_DUAL_CONNECT_ENABLED_WRITE,
        ]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        // Read enabled state
        let pkt = HuaweiSppPacket::read_request(CMD_DUAL_CONNECT_ENABLED_READ, &[1]);
        sender.send(pkt).await?;

        // Start enumeration
        self.pending_devices.clear();
        self.devices_count = 0;
        let pkt = HuaweiSppPacket::write_request(CMD_DUAL_CONNECT_ENUMERATE, &[(1, vec![])]);
        sender.send(pkt).await?;

        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        if packet.command_id == CMD_DUAL_CONNECT_ENABLED_READ {
            let value = packet.find_param(1);
            if value.len() == 1 {
                let enabled = value[0] == 1;
                let mut out = HashMap::new();
                out.insert("enabled".into(), enabled.to_string());
                put_properties(props, "dual_connect", out).await;
            }
            return Ok(());
        }

        if packet.command_id == CMD_DUAL_CONNECT_CHANGE_EVENT {
            // Device list changed, need re-init (caller should handle this)
            tracing::debug!("Dual connect change event received");
            return Ok(());
        }

        // Enumeration response
        if let Some(device) = DualConnectDevice::from_packet(packet, self.with_auto_connect) {
            let dev_index = packet
                .find_param(3)
                .first()
                .copied()
                .unwrap_or(0) as i8;
            self.devices_count = packet
                .find_param(2)
                .first()
                .copied()
                .unwrap_or(0) as i8;

            self.pending_devices.insert(dev_index, device);

            // Check if all devices received
            if self.pending_devices.len() as i8 >= self.devices_count {
                self.process_devices(props).await;
            }
        }

        Ok(())
    }

    async fn set_property(
        &mut self,
        sender: &PacketSender,
        _props: &PropertyStore,
        _group: &str,
        prop: &str,
        value: &str,
    ) -> Result<()> {
        if prop == "enabled" {
            let byte_val = if value == "true" { 1u8 } else { 0u8 };
            let pkt = HuaweiSppPacket::write_request(
                CMD_DUAL_CONNECT_ENABLED_WRITE,
                &[(1, vec![byte_val])],
            );
            sender.send(pkt).await?;
        } else if prop == "preferred_device" {
            let mac_bytes = hex_to_bytes(value)?;
            let pkt = HuaweiSppPacket::write_request(
                CMD_DUAL_CONNECT_PREFERRED_WRITE,
                &[(1, mac_bytes)],
            );
            sender.send(pkt).await?;
        } else if let Some((address, sub_prop)) = prop.split_once(':') {
            let mac_bytes = hex_to_bytes(address)?;
            let cmd_id = match (sub_prop, value) {
                ("connected", "true") => commands::CONNECT,
                ("connected", "false") => commands::DISCONNECT,
                ("auto_connect", "true") => commands::ENABLE_AUTO,
                ("auto_connect", "false") => commands::DISABLE_AUTO,
                ("name", "") => commands::UNPAIR,
                _ => return Ok(()),
            };
            let pkt = HuaweiSppPacket::write_request(
                CMD_DUAL_CONNECT_EXECUTE,
                &[(cmd_id, mac_bytes)],
            );
            sender.send(pkt).await?;
        }
        Ok(())
    }
}

impl DualConnectHandler {
    async fn process_devices(&self, props: &PropertyStore) {
        let mut devices_json = HashMap::new();
        let mut preferred = String::new();

        for i in 0..self.devices_count {
            if let Some(device) = self.pending_devices.get(&i) {
                devices_json.insert(device.mac.clone(), device.to_json_value());
                if device.preferred {
                    preferred = device.mac.clone();
                }
            }
        }

        let json_str = format!(
            "{{{}}}",
            devices_json
                .iter()
                .map(|(k, v)| format!(r#""{}": {}"#, k, v))
                .collect::<Vec<_>>()
                .join(",")
        );

        let mut out = HashMap::new();
        out.insert("devices".into(), json_str);
        out.insert("preferred_device".into(), preferred);
        put_properties(props, "dual_connect", out).await;
    }
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.replace([':', '-'], "");
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))
        })
        .collect()
}
