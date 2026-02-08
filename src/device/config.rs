use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Auto-pause handler: pause when earbuds are removed.
pub struct AutoPauseHandler;

#[async_trait]
impl DeviceHandler for AutoPauseHandler {
    fn handler_id(&self) -> &'static str {
        "tws_auto_pause"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_AUTO_PAUSE_READ, CMD_AUTO_PAUSE_WRITE]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_AUTO_PAUSE_READ, &[1]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(1);
        if data.len() == 1 {
            let enabled = data[0] == 1;
            let mut out = HashMap::new();
            out.insert("auto_pause".into(), enabled.to_string());
            put_properties(props, "config", out).await;
        }
        Ok(())
    }

    async fn set_property(
        &mut self,
        sender: &PacketSender,
        props: &PropertyStore,
        group: &str,
        prop: &str,
        value: &str,
    ) -> Result<()> {
        let byte_val = if value == "true" { 1u8 } else { 0u8 };
        let pkt = HuaweiSppPacket::write_request(CMD_AUTO_PAUSE_WRITE, &[(1, vec![byte_val])]);
        sender.send(pkt).await?;

        let mut out = HashMap::new();
        out.insert(prop.to_string(), value.to_string());
        put_properties(props, group, out).await;
        Ok(())
    }
}

/// Low latency mode toggle.
pub struct LowLatencyHandler;

#[async_trait]
impl DeviceHandler for LowLatencyHandler {
    fn handler_id(&self) -> &'static str {
        "low_latency"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_LOW_LATENCY]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_LOW_LATENCY, &[2]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let value = packet.find_param(2);
        if !value.is_empty() {
            let enabled = value[0] == 1;
            let mut out = HashMap::new();
            out.insert("low_latency".into(), enabled.to_string());
            put_properties(props, "config", out).await;
        }
        Ok(())
    }

    async fn set_property(
        &mut self,
        sender: &PacketSender,
        _props: &PropertyStore,
        _group: &str,
        _prop: &str,
        value: &str,
    ) -> Result<()> {
        let byte_val = if value == "true" {
            vec![0x01]
        } else {
            vec![0x00]
        };
        let pkt = HuaweiSppPacket::write_request(CMD_LOW_LATENCY, &[(1, byte_val)]);
        sender.send(pkt).await?;

        // Re-read after a delay (device needs time to apply)
        let read_pkt = HuaweiSppPacket::read_request(CMD_LOW_LATENCY, &[2]);
        sender.send(read_pkt).await?;
        Ok(())
    }
}

/// Sound quality preference: connectivity vs quality.
pub struct SoundQualityHandler;

fn quality_pref_name(value: u8) -> &'static str {
    match value {
        0 => "sqp_connectivity",
        1 => "sqp_quality",
        _ => "unknown",
    }
}

fn quality_pref_value(name: &str) -> Option<u8> {
    match name {
        "sqp_connectivity" => Some(0),
        "sqp_quality" => Some(1),
        _ => None,
    }
}

#[async_trait]
impl DeviceHandler for SoundQualityHandler {
    fn handler_id(&self) -> &'static str {
        "config_sound_quality"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_SOUND_QUALITY_READ]
    }

    fn ignore_commands(&self) -> &[CommandId] {
        &[CMD_SOUND_QUALITY_WRITE]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_SOUND_QUALITY_READ, &[1]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let value = packet.find_param(2);
        if value.len() == 1 {
            let name = quality_pref_name(value[0]);
            let mut out = HashMap::new();
            out.insert("quality_preference".into(), name.to_string());
            out.insert(
                "quality_preference_options".into(),
                "sqp_connectivity,sqp_quality".to_string(),
            );
            put_properties(props, "sound", out).await;
        }
        Ok(())
    }

    async fn set_property(
        &mut self,
        sender: &PacketSender,
        _props: &PropertyStore,
        _group: &str,
        _prop: &str,
        value: &str,
    ) -> Result<()> {
        let byte_val =
            quality_pref_value(value).ok_or_else(|| anyhow::anyhow!("Unknown quality pref: {}", value))?;
        let pkt =
            HuaweiSppPacket::write_request(CMD_SOUND_QUALITY_WRITE, &[(1, vec![byte_val])]);
        sender.send(pkt).await?;

        // Re-read
        let read_pkt = HuaweiSppPacket::read_request(CMD_SOUND_QUALITY_READ, &[1]);
        sender.send(read_pkt).await?;
        Ok(())
    }
}
