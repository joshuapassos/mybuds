use std::collections::HashMap;

use anyhow::{bail, Result};
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// ANC mode values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AncMode {
    Normal = 0,
    Cancellation = 1,
    Awareness = 2,
}

impl AncMode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Normal),
            1 => Some(Self::Cancellation),
            2 => Some(Self::Awareness),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Cancellation => "cancellation",
            Self::Awareness => "awareness",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "cancellation" => Some(Self::Cancellation),
            "awareness" => Some(Self::Awareness),
            _ => None,
        }
    }
}

/// Cancellation level values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelLevel {
    Normal = 0,
    Comfort = 1,
    Ultra = 2,
    Dynamic = 3,
}

impl CancelLevel {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Normal),
            1 => Some(Self::Comfort),
            2 => Some(Self::Ultra),
            3 => Some(Self::Dynamic),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Comfort => "comfort",
            Self::Ultra => "ultra",
            Self::Dynamic => "dynamic",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "comfort" => Some(Self::Comfort),
            "ultra" => Some(Self::Ultra),
            "dynamic" => Some(Self::Dynamic),
            _ => None,
        }
    }
}

/// Awareness level values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwarenessLevel {
    VoiceBoost = 1,
    Normal = 2,
}

impl AwarenessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VoiceBoost => "voice_boost",
            Self::Normal => "normal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "voice_boost" => Some(Self::VoiceBoost),
            "normal" => Some(Self::Normal),
            _ => None,
        }
    }
}

/// ANC mode switching handler.
pub struct AncHandler {
    with_cancel_levels: bool,
    with_cancel_dynamic: bool,
    with_voice_boost: bool,
    active_mode: u8,
}

impl AncHandler {
    pub fn new(with_cancel_levels: bool, with_cancel_dynamic: bool, with_voice_boost: bool) -> Self {
        Self {
            with_cancel_levels,
            with_cancel_dynamic,
            with_voice_boost,
            active_mode: 0,
        }
    }

    fn mode_options(&self) -> Vec<&'static str> {
        vec!["normal", "cancellation", "awareness"]
    }

    fn cancel_level_options(&self) -> Vec<&'static str> {
        let mut opts = vec!["comfort", "normal", "ultra"];
        if self.with_cancel_dynamic {
            opts.push("dynamic");
        }
        opts
    }

    fn awareness_level_options(&self) -> Vec<&'static str> {
        vec!["voice_boost", "normal"]
    }
}

impl Default for AncHandler {
    fn default() -> Self {
        Self::new(false, false, false)
    }
}

#[async_trait]
impl DeviceHandler for AncHandler {
    fn handler_id(&self) -> &'static str {
        "anc"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_ANC_READ]
    }

    fn ignore_commands(&self) -> &[CommandId] {
        &[CMD_ANC_WRITE]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_ANC_READ, &[1, 2]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(1);
        if data.len() != 2 {
            return Ok(());
        }

        let level_byte = data[0];
        let mode_byte = data[1];
        self.active_mode = mode_byte;

        let mode = AncMode::from_byte(mode_byte)
            .map(|m| m.as_str())
            .unwrap_or("unknown");

        let mut new_props = HashMap::new();
        new_props.insert("mode".into(), mode.to_string());
        new_props.insert("mode_options".into(), self.mode_options().join(","));

        // If cancellation is active and we support levels
        if mode_byte == 1 && self.with_cancel_levels {
            let level = CancelLevel::from_byte(level_byte)
                .map(|l| l.as_str())
                .unwrap_or("unknown");
            new_props.insert("level".into(), level.to_string());
            new_props.insert(
                "level_options".into(),
                self.cancel_level_options().join(","),
            );
        } else if mode_byte == 2 && self.with_voice_boost {
            let level = match level_byte {
                1 => "voice_boost",
                2 => "normal",
                _ => "unknown",
            };
            new_props.insert("level".into(), level.to_string());
            new_props.insert(
                "level_options".into(),
                self.awareness_level_options().join(","),
            );
        }

        put_properties(props, "anc", new_props).await;
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
        let data = if prop == "mode" {
            let mode = AncMode::from_str(value)
                .ok_or_else(|| anyhow::anyhow!("Unknown ANC mode: {}", value))?;
            let mode_byte = mode as u8;
            let level_byte = if mode_byte == 0 { 0x00 } else { 0xFF };
            vec![mode_byte, level_byte]
        } else {
            // Change level within current mode
            let level_byte = if self.active_mode != 2 {
                // Cancellation levels
                CancelLevel::from_str(value)
                    .ok_or_else(|| anyhow::anyhow!("Unknown cancel level: {}", value))?
                    as u8
            } else {
                // Awareness levels
                match value {
                    "voice_boost" => 1,
                    "normal" => 2,
                    _ => bail!("Unknown awareness level: {}", value),
                }
            };
            vec![self.active_mode, level_byte]
        };

        let pkt = HuaweiSppPacket::write_request(CMD_ANC_WRITE, &[(1, data)]);
        sender.send(pkt).await?;

        // Re-read current state
        let read_pkt = HuaweiSppPacket::read_request(CMD_ANC_READ, &[1, 2]);
        sender.send(read_pkt).await?;
        Ok(())
    }
}

/// Handles legacy ANC change notifications (0x2B03) from on-device button presses.
pub struct AncLegacyChangeHandler;

#[async_trait]
impl DeviceHandler for AncLegacyChangeHandler {
    fn handler_id(&self) -> &'static str {
        "anc_change"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_ANC_LEGACY_NOTIFY]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, _props: &PropertyStore) -> Result<()> {
        // When we get a legacy ANC change, we should trigger a re-read
        // The device manager will handle dispatching this
        let data = packet.find_param(1);
        if data.len() == 1 && data[0] <= 2 {
            tracing::debug!("ANC legacy change detected: mode={}", data[0]);
        }
        Ok(())
    }
}
