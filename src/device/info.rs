use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Known parameter type to property name mapping for device info.
fn param_descriptor(key: u8) -> &'static str {
    match key {
        3 => "hardware_ver",
        7 => "software_ver",
        9 => "serial_number",
        10 => "device_submodel",
        15 => "device_model",
        _ => "",
    }
}

/// Device info handler. Reads model, hardware/software version, serial numbers.
pub struct InfoHandler;

#[async_trait]
impl DeviceHandler for InfoHandler {
    fn handler_id(&self) -> &'static str {
        "device_info"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_DEVICE_INFO]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        // Request all possible parameter types (0..31)
        let params: Vec<u8> = (0..32).collect();
        let pkt = HuaweiSppPacket::read_request(CMD_DEVICE_INFO, &params);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let mut out = HashMap::new();

        for (&key, value) in &packet.parameters {
            // Special case: per-earphone serial numbers (param 24)
            if key == 24 {
                if let Ok(s) = String::from_utf8(value.clone()) {
                    if s.starts_with("L-") {
                        parse_per_earphone_sn(&mut out, &s);
                        continue;
                    }
                }
            }

            let name = param_descriptor(key);
            let name = if name.is_empty() {
                format!("field_{}", key)
            } else {
                name.to_string()
            };

            // Try to decode as UTF-8, fall back to hex
            let decoded = String::from_utf8(value.clone())
                .unwrap_or_else(|_| value.iter().map(|b| format!("{:02x}", b)).collect());

            out.insert(name, decoded);
        }

        put_properties(props, "info", out).await;
        Ok(())
    }
}

fn parse_per_earphone_sn(out: &mut HashMap<String, String>, data: &str) {
    if let Some((left, right)) = data.split_once(',') {
        if left.starts_with("L-") {
            out.insert("left_serial_number".into(), left[2..].to_string());
        }
        if right.starts_with("R-") {
            out.insert("right_serial_number".into(), right[2..].to_string());
        }
    }
}
