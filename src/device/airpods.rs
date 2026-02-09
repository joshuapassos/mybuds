//! AirPods device handlers.
//!
//! These handlers process Apple Audio Control Protocol (AACP) packets
//! that have been mapped to HuaweiSppPacket format by the L2CAP transport.
//!
//! Command ID mapping:
//! - [0xAA, opcode] — general AAP opcodes (battery, ear detection, device info)
//! - [0xA9, identifier] — control command subtypes (ANC, conversational awareness, etc.)

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::aap;
use crate::protocol::commands::CommandId;
use crate::protocol::HuaweiSppPacket;

// --- Command IDs (AAP opcode mapped to 2-byte command_id) ---

const CMD_BATTERY: CommandId = [aap::CMD_PREFIX, aap::OP_BATTERY_INFO];
const CMD_EAR_DETECTION: CommandId = [aap::CMD_PREFIX, aap::OP_EAR_DETECTION];
const CMD_DEVICE_INFO: CommandId = [aap::CMD_PREFIX, aap::OP_DEVICE_INFO];
const CMD_CA_NOTIFY: CommandId = [aap::CMD_PREFIX, aap::OP_CONVERSATION_AWARENESS];
const CMD_LISTENING_MODE: CommandId = [aap::CMD_CC_PREFIX, aap::CC_LISTENING_MODE];
const CMD_CONVERSATION_DETECT: CommandId = [aap::CMD_CC_PREFIX, aap::CC_CONVERSATION_DETECT];
const CMD_ADAPTIVE_VOLUME: CommandId = [aap::CMD_CC_PREFIX, aap::CC_ADAPTIVE_VOLUME];
const CMD_EAR_DETECT_CONFIG: CommandId = [aap::CMD_CC_PREFIX, aap::CC_EAR_DETECTION_CONFIG];
const CMD_ANC_STRENGTH: CommandId = [aap::CMD_CC_PREFIX, aap::CC_AUTO_ANC_STRENGTH];
const CMD_LISTENING_CONFIGS: CommandId = [aap::CMD_CC_PREFIX, aap::CC_LISTENING_MODE_CONFIGS];
const CMD_ONE_BUD_ANC: CommandId = [aap::CMD_CC_PREFIX, aap::CC_ONE_BUD_ANC];

// ============================================================
// Battery handler
// ============================================================

/// Parses AirPods battery notifications (opcode 0x04).
///
/// Payload format: [count] ([component_type] 01 [level] [status] 01)...
/// Component types: 0x02=right, 0x04=left, 0x08=case
pub struct AirPodsBatteryHandler;

#[async_trait]
impl DeviceHandler for AirPodsBatteryHandler {
    fn handler_id(&self) -> &'static str {
        "battery"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_BATTERY]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        // Battery notifications arrive automatically after subscribing
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);
        if data.is_empty() {
            return Ok(());
        }

        let mut out = HashMap::new();
        let count = data[0] as usize;
        let mut pos = 1;

        for _ in 0..count {
            if pos + 4 >= data.len() {
                break;
            }

            let component = data[pos];
            // data[pos+1] is always 0x01
            let level = data[pos + 2];
            let status = data[pos + 3];
            // data[pos+4] is always 0x01
            pos += 5;

            let key = match component {
                aap::BATT_LEFT => "left",
                aap::BATT_RIGHT => "right",
                aap::BATT_CASE => "case",
                _ => continue,
            };

            if status != aap::BATT_DISCONNECTED {
                out.insert(key.to_string(), level.to_string());
            }

            if component == aap::BATT_LEFT || component == aap::BATT_RIGHT {
                let is_charging = status == aap::BATT_CHARGING;
                out.insert(
                    format!("{}_charging", key),
                    is_charging.to_string(),
                );
            }
        }

        // Compute global as average of left/right
        let left: Option<u8> = out.get("left").and_then(|s| s.parse().ok());
        let right: Option<u8> = out.get("right").and_then(|s| s.parse().ok());
        if let (Some(l), Some(r)) = (left, right) {
            out.insert("global".to_string(), ((l as u16 + r as u16) / 2).to_string());
        }

        let is_charging = out
            .get("left_charging")
            .map_or(false, |s| s == "true")
            || out
                .get("right_charging")
                .map_or(false, |s| s == "true");
        out.insert("is_charging".to_string(), is_charging.to_string());

        put_properties(props, "battery", out).await;
        Ok(())
    }
}

// ============================================================
// Ear detection handler
// ============================================================

/// Parses AirPods ear detection (opcode 0x06).
///
/// Payload: [primary_pod_state] [secondary_pod_state]
/// Values: 0x00=in-ear, 0x01=out, 0x02=in-case
pub struct AirPodsEarDetectionHandler;

fn ear_state_str(v: u8) -> &'static str {
    match v {
        aap::EAR_IN => "in_ear",
        aap::EAR_OUT => "out",
        aap::EAR_IN_CASE => "in_case",
        _ => "unknown",
    }
}

#[async_trait]
impl DeviceHandler for AirPodsEarDetectionHandler {
    fn handler_id(&self) -> &'static str {
        "ear_detection"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_EAR_DETECTION, CMD_EAR_DETECT_CONFIG]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);

        if packet.command_id == CMD_EAR_DETECTION {
            if data.len() >= 2 {
                let mut out = HashMap::new();
                out.insert("primary".to_string(), ear_state_str(data[0]).to_string());
                out.insert("secondary".to_string(), ear_state_str(data[1]).to_string());
                put_properties(props, "ear_detection", out).await;
            }
        } else if packet.command_id == CMD_EAR_DETECT_CONFIG {
            // Config response: value 0x01=enabled, 0x02=disabled
            if !data.is_empty() {
                let enabled = data[0] == 0x01;
                let mut out = HashMap::new();
                out.insert("enabled".to_string(), enabled.to_string());
                put_properties(props, "ear_detection", out).await;
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
            let byte_val = if value == "true" { 0x01 } else { 0x02 };
            let pkt = build_control_command(aap::CC_EAR_DETECTION_CONFIG, byte_val);
            sender.send(pkt).await?;
        }
        Ok(())
    }
}

// ============================================================
// ANC / Listening mode handler
// ============================================================

/// AirPods noise control modes.
///
/// Listens to control command identifier 0x0D (LISTENING_MODE).
/// Modes: 0x01=Off, 0x02=ANC, 0x03=Transparency, 0x04=Adaptive
pub struct AirPodsAncHandler {
    /// Whether this model supports adaptive mode.
    with_adaptive: bool,
}

impl AirPodsAncHandler {
    pub fn new(with_adaptive: bool) -> Self {
        Self { with_adaptive }
    }

    fn mode_options(&self) -> Vec<&'static str> {
        let mut opts = vec!["off", "anc", "transparency"];
        if self.with_adaptive {
            opts.push("adaptive");
        }
        opts
    }
}

fn listening_mode_str(v: u8) -> &'static str {
    match v {
        aap::LM_OFF => "off",
        aap::LM_ANC => "anc",
        aap::LM_TRANSPARENCY => "transparency",
        aap::LM_ADAPTIVE => "adaptive",
        _ => "unknown",
    }
}

fn listening_mode_byte(s: &str) -> Option<u8> {
    match s {
        "off" => Some(aap::LM_OFF),
        "anc" => Some(aap::LM_ANC),
        "transparency" => Some(aap::LM_TRANSPARENCY),
        "adaptive" => Some(aap::LM_ADAPTIVE),
        _ => None,
    }
}

#[async_trait]
impl DeviceHandler for AirPodsAncHandler {
    fn handler_id(&self) -> &'static str {
        "anc"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_LISTENING_MODE, CMD_LISTENING_CONFIGS, CMD_ANC_STRENGTH, CMD_ONE_BUD_ANC]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);

        if packet.command_id == CMD_LISTENING_MODE {
            if !data.is_empty() {
                let mode = listening_mode_str(data[0]);
                let mut out = HashMap::new();
                out.insert("mode".to_string(), mode.to_string());
                out.insert("mode_options".to_string(), self.mode_options().join(","));
                put_properties(props, "anc", out).await;
            }
        } else if packet.command_id == CMD_ANC_STRENGTH {
            if !data.is_empty() {
                let strength = data[0];
                let mut out = HashMap::new();
                out.insert("anc_strength".to_string(), strength.to_string());
                put_properties(props, "anc", out).await;
            }
        } else if packet.command_id == CMD_ONE_BUD_ANC {
            if !data.is_empty() {
                let enabled = data[0] == 0x01;
                let mut out = HashMap::new();
                out.insert("one_bud_anc".to_string(), enabled.to_string());
                put_properties(props, "anc", out).await;
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
        match prop {
            "mode" => {
                let byte_val = listening_mode_byte(value)
                    .ok_or_else(|| anyhow::anyhow!("Unknown listening mode: {}", value))?;
                let pkt = build_control_command(aap::CC_LISTENING_MODE, byte_val);
                sender.send(pkt).await?;
            }
            "anc_strength" => {
                let strength: u8 = value.parse()?;
                let pkt = build_control_command(aap::CC_AUTO_ANC_STRENGTH, strength);
                sender.send(pkt).await?;
            }
            "one_bud_anc" => {
                let byte_val = if value == "true" { 0x01 } else { 0x02 };
                let pkt = build_control_command(aap::CC_ONE_BUD_ANC, byte_val);
                sender.send(pkt).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

// ============================================================
// Conversational Awareness handler
// ============================================================

/// Conversational Awareness toggle (control command 0x28) and
/// notification (opcode 0x4B).
pub struct AirPodsConversationAwarenessHandler;

#[async_trait]
impl DeviceHandler for AirPodsConversationAwarenessHandler {
    fn handler_id(&self) -> &'static str {
        "conversation_awareness"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_CONVERSATION_DETECT, CMD_CA_NOTIFY]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);

        if packet.command_id == CMD_CONVERSATION_DETECT {
            // Control command response: [status] [trailing...]
            // 0x01 = enabled, 0x02 = disabled
            if !data.is_empty() {
                let enabled = data[0] == 0x01;
                let mut out = HashMap::new();
                out.insert("enabled".to_string(), enabled.to_string());
                put_properties(props, "conversation_awareness", out).await;
            }
        } else if packet.command_id == CMD_CA_NOTIFY {
            // CA notification: payload contains activity level
            // 0x01/0x02 = speaking (volume reduced), 0x03 = stopped, 0x08/0x09 = normal
            if data.len() >= 3 {
                let level = data[2]; // third byte after 02 00
                let speaking = level == 0x01 || level == 0x02;
                let mut out = HashMap::new();
                out.insert("speaking".to_string(), speaking.to_string());
                put_properties(props, "conversation_awareness", out).await;
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
            let byte_val = if value == "true" { 0x01 } else { 0x02 };
            let pkt = build_control_command(aap::CC_CONVERSATION_DETECT, byte_val);
            sender.send(pkt).await?;
        }
        Ok(())
    }
}

// ============================================================
// Personalized Volume handler
// ============================================================

/// Personalized/Adaptive Volume toggle (control command 0x26).
pub struct AirPodsPersonalizedVolumeHandler;

#[async_trait]
impl DeviceHandler for AirPodsPersonalizedVolumeHandler {
    fn handler_id(&self) -> &'static str {
        "personalized_volume"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_ADAPTIVE_VOLUME]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);
        if !data.is_empty() {
            let enabled = data[0] == 0x01;
            let mut out = HashMap::new();
            out.insert("enabled".to_string(), enabled.to_string());
            put_properties(props, "personalized_volume", out).await;
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
            let byte_val = if value == "true" { 0x01 } else { 0x02 };
            let pkt = build_control_command(aap::CC_ADAPTIVE_VOLUME, byte_val);
            sender.send(pkt).await?;
        }
        Ok(())
    }
}

// ============================================================
// Device Info handler
// ============================================================

/// Parses AirPods device info (opcode 0x1D).
///
/// Payload: sequence of null-terminated UTF-8 strings:
/// [name, model, manufacturer, serial, fw1, fw2, hw_rev, ...]
pub struct AirPodsInfoHandler;

#[async_trait]
impl DeviceHandler for AirPodsInfoHandler {
    fn handler_id(&self) -> &'static str {
        "device_info"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_DEVICE_INFO]
    }

    async fn on_init(&mut self, _sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let data = packet.find_param(0);
        if data.is_empty() {
            return Ok(());
        }

        // Split null-terminated strings
        let strings: Vec<String> = data
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect();

        let fields = [
            "device_name",
            "device_model",
            "manufacturer",
            "serial_number",
            "firmware_ver_1",
            "firmware_ver_2",
            "hardware_ver",
            "updater_id",
            "left_serial",
            "right_serial",
        ];

        let mut out = HashMap::new();
        for (i, value) in strings.iter().enumerate() {
            if let Some(&key) = fields.get(i) {
                out.insert(key.to_string(), value.clone());
            }
        }

        // Alias for UI compatibility with Huawei info display
        if let Some(fw) = out.get("firmware_ver_1").cloned() {
            out.insert("software_ver".to_string(), fw);
        }

        put_properties(props, "info", out).await;
        Ok(())
    }
}

// ============================================================
// Helpers
// ============================================================

/// Build a control command HuaweiSppPacket (for sending via handler).
fn build_control_command(identifier: u8, value: u8) -> HuaweiSppPacket {
    let mut pkt = HuaweiSppPacket::new([aap::CMD_CC_PREFIX, identifier]);
    pkt.parameters.insert(0, vec![value, 0x00, 0x00, 0x00]);
    pkt
}
