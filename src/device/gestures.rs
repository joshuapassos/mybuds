use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Tap action options (signed byte values).
fn tap_action_name(value: i8) -> &'static str {
    match value {
        -1 => "tap_action_off",
        0 => "tap_action_assistant",
        1 => "tap_action_pause",
        2 => "tap_action_next",
        7 => "tap_action_prev",
        _ => "unknown",
    }
}

fn tap_action_value(name: &str) -> Option<i8> {
    match name {
        "tap_action_off" => Some(-1),
        "tap_action_assistant" => Some(0),
        "tap_action_pause" => Some(1),
        "tap_action_next" => Some(2),
        "tap_action_prev" => Some(7),
        _ => None,
    }
}

fn call_action_name(value: i8) -> &'static str {
    match value {
        -1 => "tap_action_off",
        0 => "tap_action_answer",
        _ => "unknown",
    }
}

fn call_action_value(name: &str) -> Option<i8> {
    match name {
        "tap_action_off" => Some(-1),
        "tap_action_answer" => Some(0),
        _ => None,
    }
}

/// Generic multi-tap handler (double tap / triple tap).
pub struct TapActionHandler {
    prop_prefix: &'static str,
    cmd_read: CommandId,
    cmd_write: CommandId,
    with_in_call: bool,
}

impl TapActionHandler {
    pub fn double_tap(with_in_call: bool) -> Self {
        Self {
            prop_prefix: "double_tap",
            cmd_read: CMD_DUAL_TAP_READ,
            cmd_write: CMD_DUAL_TAP_WRITE,
            with_in_call,
        }
    }

    pub fn triple_tap() -> Self {
        Self {
            prop_prefix: "triple_tap",
            cmd_read: CMD_TRIPLE_TAP_READ,
            cmd_write: CMD_TRIPLE_TAP_WRITE,
            with_in_call: false,
        }
    }
}

#[async_trait]
impl DeviceHandler for TapActionHandler {
    fn handler_id(&self) -> &'static str {
        match self.prop_prefix {
            "double_tap" => "gesture_double",
            "triple_tap" => "gesture_triple",
            _ => "gesture_tap",
        }
    }

    fn commands(&self) -> &[CommandId] {
        // We return a static slice - need to leak or use a different pattern
        // For now, we'll match on the handler type
        match self.prop_prefix {
            "double_tap" => &[CMD_DUAL_TAP_READ, CMD_DUAL_TAP_WRITE],
            _ => &[CMD_TRIPLE_TAP_READ, CMD_TRIPLE_TAP_WRITE],
        }
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(self.cmd_read, &[1, 2]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        if packet.command_id != self.cmd_read {
            return Ok(());
        }

        let mut out = HashMap::new();

        let left = packet.find_param(1);
        if left.len() == 1 {
            let value = left[0] as i8;
            out.insert(
                format!("{}_left", self.prop_prefix),
                tap_action_name(value).to_string(),
            );
        }

        let right = packet.find_param(2);
        if right.len() == 1 {
            let value = right[0] as i8;
            out.insert(
                format!("{}_right", self.prop_prefix),
                tap_action_name(value).to_string(),
            );
        }

        let available = packet.find_param(3);
        if !available.is_empty() {
            let options: Vec<String> = available
                .iter()
                .map(|&b| tap_action_name(b as i8).to_string())
                .collect();
            out.insert(format!("{}_options", self.prop_prefix), options.join(","));
        }

        if self.with_in_call {
            let in_call = packet.find_param(4);
            if in_call.len() == 1 {
                let value = in_call[0] as i8;
                out.insert(
                    format!("{}_in_call", self.prop_prefix),
                    call_action_name(value).to_string(),
                );
                out.insert(
                    format!("{}_in_call_options", self.prop_prefix),
                    "tap_action_off,tap_action_answer".to_string(),
                );
            }
        }

        put_properties(props, "action", out).await;
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
        let (p_type, is_call) = if prop.ends_with("_left") {
            (1u8, false)
        } else if prop.ends_with("_right") {
            (2u8, false)
        } else if prop.ends_with("_in_call") {
            (4u8, true)
        } else {
            return Ok(());
        };

        let byte_val = if is_call {
            call_action_value(value)
        } else {
            tap_action_value(value)
        }
        .ok_or_else(|| anyhow::anyhow!("Unknown action: {}", value))?;

        let pkt =
            HuaweiSppPacket::write_request(self.cmd_write, &[(p_type, vec![byte_val as u8])]);
        sender.send(pkt).await?;

        // Update local prop
        let mut out = HashMap::new();
        out.insert(prop.to_string(), value.to_string());
        put_properties(props, group, out).await;
        Ok(())
    }
}

/// Long tap handler with split left/right + ANC mode cycle configuration.
pub struct LongTapSplitHandler {
    with_left: bool,
    with_right: bool,
    with_in_call: bool,
    with_anc: bool,
}

impl LongTapSplitHandler {
    pub fn new(with_left: bool, with_right: bool, with_in_call: bool, with_anc: bool) -> Self {
        Self {
            with_left,
            with_right,
            with_in_call,
            with_anc,
        }
    }
}

impl Default for LongTapSplitHandler {
    fn default() -> Self {
        Self::new(true, false, false, true)
    }
}

fn long_tap_action_name(value: i8) -> &'static str {
    match value {
        -1 => "tap_action_off",
        10 => "tap_action_switch_anc",
        _ => "unknown",
    }
}

fn long_tap_action_value(name: &str) -> Option<i8> {
    match name {
        "tap_action_off" => Some(-1),
        "tap_action_switch_anc" => Some(10),
        _ => None,
    }
}

fn anc_cycle_name(value: i8) -> &'static str {
    match value {
        1 => "noise_control_off_on",
        2 => "noise_control_off_on_aw",
        3 => "noise_control_on_aw",
        4 => "noise_control_off_aw",
        _ => "unknown",
    }
}

fn anc_cycle_value(name: &str) -> Option<i8> {
    match name {
        "noise_control_off_on" => Some(1),
        "noise_control_off_on_aw" => Some(2),
        "noise_control_on_aw" => Some(3),
        "noise_control_off_aw" => Some(4),
        _ => None,
    }
}

#[async_trait]
impl DeviceHandler for LongTapSplitHandler {
    fn handler_id(&self) -> &'static str {
        "gesture_long_split"
    }

    fn commands(&self) -> &[CommandId] {
        &[
            CMD_LONG_TAP_SPLIT_READ_BASE,
            CMD_LONG_TAP_SPLIT_READ_ANC,
            CMD_LONG_TAP_SPLIT_WRITE_BASE,
            CMD_LONG_TAP_SPLIT_WRITE_ANC,
        ]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_LONG_TAP_SPLIT_READ_BASE, &[1, 2]);
        sender.send(pkt).await?;

        if self.with_anc {
            let pkt = HuaweiSppPacket::read_request(CMD_LONG_TAP_SPLIT_READ_ANC, &[1, 2]);
            sender.send(pkt).await?;
        }
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let mut out = HashMap::new();

        if packet.command_id == CMD_LONG_TAP_SPLIT_READ_BASE {
            let left = packet.find_param(1);
            if left.len() == 1 && self.with_left {
                let value = left[0] as i8;
                out.insert(
                    "long_tap_left".into(),
                    long_tap_action_name(value).to_string(),
                );
            }

            let right = packet.find_param(2);
            if right.len() == 1 && self.with_right {
                let value = right[0] as i8;
                out.insert(
                    "long_tap_right".into(),
                    long_tap_action_name(value).to_string(),
                );
            }

            if self.with_in_call {
                let in_call = packet.find_param(4);
                if in_call.len() == 1 {
                    let value = in_call[0] as i8;
                    out.insert(
                        "long_tap_in_call".into(),
                        call_action_name(value).to_string(),
                    );
                    out.insert(
                        "long_tap_in_call_options".into(),
                        "tap_action_off,tap_action_answer".to_string(),
                    );
                }
            }
            out.insert(
                "long_tap_options".into(),
                "tap_action_off,tap_action_switch_anc".to_string(),
            );
        } else if packet.command_id == CMD_LONG_TAP_SPLIT_READ_ANC {
            let left = packet.find_param(1);
            if left.len() == 1 {
                let value = left[0] as i8;
                out.insert(
                    "noise_control_left".into(),
                    anc_cycle_name(value).to_string(),
                );
            }

            let right = packet.find_param(2);
            if right.len() == 1 && self.with_right {
                let value = right[0] as i8;
                out.insert(
                    "noise_control_right".into(),
                    anc_cycle_name(value).to_string(),
                );
            }
            out.insert(
                "noise_control_options".into(),
                "noise_control_off_on,noise_control_off_on_aw,noise_control_on_aw,noise_control_off_aw".to_string(),
            );
        }

        if !out.is_empty() {
            put_properties(props, "action", out).await;
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
        if prop.starts_with("long_tap") {
            let p_type = if prop.contains("left") {
                1u8
            } else if prop.contains("right") {
                2u8
            } else if prop.contains("in_call") {
                4u8
            } else {
                return Ok(());
            };

            let byte_val = if prop.contains("in_call") {
                call_action_value(value)
            } else {
                long_tap_action_value(value)
            }
            .ok_or_else(|| anyhow::anyhow!("Unknown long tap action: {}", value))?;

            let pkt = HuaweiSppPacket::write_request(
                CMD_LONG_TAP_SPLIT_WRITE_BASE,
                &[(p_type, vec![byte_val as u8])],
            );
            sender.send(pkt).await?;
        } else if prop.starts_with("noise_control") {
            let p_type = if prop.contains("left") { 1u8 } else { 2u8 };
            let byte_val =
                anc_cycle_value(value).ok_or_else(|| anyhow::anyhow!("Unknown ANC cycle: {}", value))?;

            let pkt = HuaweiSppPacket::write_request(
                CMD_LONG_TAP_SPLIT_WRITE_ANC,
                &[(p_type, vec![byte_val as u8])],
            );
            sender.send(pkt).await?;
        }

        let mut out = HashMap::new();
        out.insert(prop.to_string(), value.to_string());
        put_properties(props, group, out).await;
        Ok(())
    }
}

/// Swipe gesture handler.
pub struct SwipeGestureHandler;

fn swipe_action_name(value: i8) -> &'static str {
    match value {
        -1 => "tap_action_off",
        0 => "tap_action_change_volume",
        _ => "unknown",
    }
}

fn swipe_action_value(name: &str) -> Option<i8> {
    match name {
        "tap_action_off" => Some(-1),
        "tap_action_change_volume" => Some(0),
        _ => None,
    }
}

#[async_trait]
impl DeviceHandler for SwipeGestureHandler {
    fn handler_id(&self) -> &'static str {
        "gesture_swipe"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_SWIPE_READ, CMD_SWIPE_WRITE]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_SWIPE_READ, &[1, 2]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        if packet.command_id != CMD_SWIPE_READ {
            return Ok(());
        }

        let mut out = HashMap::new();
        let action = packet.find_param(1);
        if action.len() == 1 {
            let value = action[0] as i8;
            out.insert(
                "swipe_gesture".into(),
                swipe_action_name(value).to_string(),
            );
        }
        out.insert(
            "swipe_gesture_options".into(),
            "tap_action_off,tap_action_change_volume".to_string(),
        );

        put_properties(props, "action", out).await;
        Ok(())
    }

    async fn set_property(
        &mut self,
        sender: &PacketSender,
        props: &PropertyStore,
        group: &str,
        _prop: &str,
        value: &str,
    ) -> Result<()> {
        let byte_val =
            swipe_action_value(value).ok_or_else(|| anyhow::anyhow!("Unknown swipe action: {}", value))?;

        let pkt = HuaweiSppPacket::write_request(
            CMD_SWIPE_WRITE,
            &[
                (1, vec![byte_val as u8]),
                (2, vec![byte_val as u8]),
            ],
        );
        sender.send(pkt).await?;

        let mut out = HashMap::new();
        out.insert("swipe_gesture".into(), value.to_string());
        put_properties(props, group, out).await;
        Ok(())
    }
}
