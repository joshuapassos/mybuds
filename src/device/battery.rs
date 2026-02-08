use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Battery read handler.
///
/// Reads global battery percentage, per-earbud levels (left/right/case),
/// and charging state.
pub struct BatteryHandler {
    /// Whether to parse per-earbud (TWS) battery data.
    with_tws: bool,
}

impl BatteryHandler {
    pub fn new(with_tws: bool) -> Self {
        Self { with_tws }
    }
}

impl Default for BatteryHandler {
    fn default() -> Self {
        Self::new(true)
    }
}

#[async_trait]
impl DeviceHandler for BatteryHandler {
    fn handler_id(&self) -> &'static str {
        "battery"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_BATTERY_READ, CMD_BATTERY_NOTIFY]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_BATTERY_READ, &[1, 2, 3]);
        sender.send(pkt).await?;
        // Response will arrive via on_packet
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let mut out = HashMap::new();

        // Param 1: global battery percentage (1 byte)
        let global = packet.find_param(1);
        if global.len() == 1 {
            out.insert("global".into(), global[0].to_string());
        }

        // Param 2: per-earbud battery [left, right, case] (3 bytes)
        let per_bud = packet.find_param(2);
        if per_bud.len() == 3 && self.with_tws {
            out.insert("left".into(), per_bud[0].to_string());
            out.insert("right".into(), per_bud[1].to_string());
            out.insert("case".into(), per_bud[2].to_string());
        }

        // Param 3: charging state
        let charging = packet.find_param(3);
        if !charging.is_empty() {
            let is_charging = charging.contains(&0x01);
            out.insert("is_charging".into(), is_charging.to_string());
        }

        put_properties(props, "battery", out).await;
        Ok(())
    }
}
