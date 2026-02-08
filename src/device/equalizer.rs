use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::handler::{put_properties, DeviceHandler, PacketSender, PropertyStore};
use crate::protocol::commands::*;
use crate::protocol::HuaweiSppPacket;

/// Known built-in EQ presets.
fn builtin_preset_name(id: u8) -> Option<&'static str> {
    match id {
        1 => Some("equalizer_preset_default"),
        2 => Some("equalizer_preset_hardbass"),
        3 => Some("equalizer_preset_treble"),
        9 => Some("equalizer_preset_voices"),
        _ => None,
    }
}

/// EQ preset entry: (id, label).
#[derive(Debug, Clone)]
struct PresetEntry {
    id: i16,
    label: String,
    data: Option<Vec<u8>>,
}

/// Equalizer preset handler.
pub struct EqualizerHandler {
    presets: Vec<(u8, &'static str)>,
    with_custom: bool,
    custom_rows: usize,
    custom_max_count: usize,
    preset_data: Vec<PresetEntry>,
}

impl EqualizerHandler {
    pub fn new(
        presets: Vec<(u8, &'static str)>,
        with_custom: bool,
    ) -> Self {
        let preset_data: Vec<PresetEntry> = presets
            .iter()
            .map(|&(id, name)| PresetEntry {
                id: id as i16,
                label: format!("equalizer_preset_{}", name),
                data: None,
            })
            .collect();

        Self {
            presets,
            with_custom,
            custom_rows: 10,
            custom_max_count: 3,
            preset_data,
        }
    }

    pub fn with_presets(presets: Vec<(u8, &'static str)>) -> Self {
        Self::new(presets, false)
    }
}

#[async_trait]
impl DeviceHandler for EqualizerHandler {
    fn handler_id(&self) -> &'static str {
        "config_eq"
    }

    fn commands(&self) -> &[CommandId] {
        &[CMD_EQUALIZER_READ]
    }

    fn ignore_commands(&self) -> &[CommandId] {
        &[CMD_EQUALIZER_WRITE]
    }

    async fn on_init(&mut self, sender: &PacketSender, _props: &PropertyStore) -> Result<()> {
        let pkt = HuaweiSppPacket::read_request(CMD_EQUALIZER_READ, &[1, 2, 3, 4, 5, 6, 7, 8]);
        sender.send(pkt).await?;
        Ok(())
    }

    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()> {
        let mut out = HashMap::new();

        // Param 3: available built-in preset IDs
        let available = packet.find_param(3);
        if !available.is_empty() {
            // Rebuild preset list from device response
            let mut new_presets = Vec::new();
            for &id in available {
                let name = builtin_preset_name(id)
                    .unwrap_or("unknown")
                    .to_string();
                new_presets.push(PresetEntry {
                    id: id as i16,
                    label: name,
                    data: None,
                });
            }
            // Keep custom presets from before
            new_presets.extend(
                self.preset_data
                    .iter()
                    .filter(|p| p.data.is_some())
                    .cloned(),
            );
            self.preset_data = new_presets;
        }

        // Param 8: custom presets (36 bytes each)
        let custom_modes = packet.find_param(8);
        if self.with_custom && !custom_modes.is_empty() {
            let mut offset = 0;
            while offset + 36 <= custom_modes.len() {
                let chunk = &custom_modes[offset..offset + 36];
                let mode_id = chunk[0] as i16;
                let count_lines = chunk[1] as usize;
                let data = chunk[2..2 + count_lines].to_vec();
                let label_bytes = &chunk[2 + count_lines..];
                let label = label_bytes
                    .split(|&b| b == 0)
                    .next()
                    .and_then(|s| String::from_utf8(s.to_vec()).ok())
                    .unwrap_or_else(|| format!("custom_{}", mode_id));

                self.preset_data.push(PresetEntry {
                    id: mode_id,
                    label,
                    data: Some(data),
                });
                offset += 36;
            }
        }

        // Build options list
        let options: Vec<String> = self.preset_data.iter().map(|p| p.label.clone()).collect();
        out.insert("equalizer_preset_options".into(), options.join(","));
        out.insert(
            "equalizer_max_custom_modes".into(),
            if self.with_custom {
                self.custom_max_count.to_string()
            } else {
                "0".into()
            },
        );

        // Param 2: current mode ID
        let current = packet.find_param(2);
        if current.len() == 1 {
            let current_id = current[0] as i8 as i16;
            let mut found_label = format!("unknown_{}", current_id);
            for preset in &self.preset_data {
                if preset.id == current_id {
                    found_label = preset.label.clone();
                    if let Some(ref data) = preset.data {
                        let rows: Vec<String> =
                            data.iter().map(|&b| (b as i8).to_string()).collect();
                        out.insert("equalizer_rows".into(), format!("[{}]", rows.join(",")));
                    }
                    break;
                }
            }
            out.insert("equalizer_preset".into(), found_label);
        }

        put_properties(props, "sound", out).await;
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
        if prop == "equalizer_preset" {
            // Find preset by label
            let preset = self
                .preset_data
                .iter()
                .find(|p| p.label == value);

            if let Some(preset) = preset {
                let mode_id = preset.id;

                if preset.data.is_some() {
                    // Custom mode: send full payload
                    let data = preset.data.as_ref().unwrap();
                    let pkt = HuaweiSppPacket::write_request(
                        CMD_EQUALIZER_WRITE,
                        &[
                            (1, vec![mode_id as u8]),
                            (2, vec![data.len() as u8]),
                            (3, data.clone()),
                            (4, value.as_bytes().to_vec()),
                            (5, vec![1]),
                        ],
                    );
                    sender.send(pkt).await?;
                } else {
                    // Built-in mode: just send ID
                    let pkt = HuaweiSppPacket::write_request(
                        CMD_EQUALIZER_WRITE,
                        &[(1, vec![mode_id as u8])],
                    );
                    sender.send(pkt).await?;
                }
            }

            // Re-read state
            let pkt =
                HuaweiSppPacket::read_request(CMD_EQUALIZER_READ, &[1, 2, 3, 4, 5, 6, 7, 8]);
            sender.send(pkt).await?;
        }
        Ok(())
    }
}
