use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, pick_list, row, text, toggler};
use iced::{Element, Length};

use crate::ui::Message;

pub fn view<'a>(
    sound: &'a HashMap<String, String>,
    config: &'a HashMap<String, String>,
) -> Element<'a, Message> {
    let mut content = column![text("Sound Settings").size(18)].spacing(12);

    // EQ Presets
    let current_eq = sound.get("equalizer_preset").cloned();
    let eq_options: Vec<String> = sound
        .get("equalizer_preset_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if !eq_options.is_empty() {
        let eq_labels: Vec<String> = eq_options.iter().map(|s| eq_display_name(s)).collect();
        let current_label = current_eq.as_ref().map(|s| eq_display_name(s));
        let eq_labels_clone = eq_labels.clone();
        let eq_options_clone = eq_options.clone();

        content = content.push(
            column![
                text("Equalizer Preset").size(14),
                pick_list(eq_labels, current_label, move |selected: String| {
                    let idx = eq_labels_clone
                        .iter()
                        .position(|s| *s == selected)
                        .unwrap_or(0);
                    Message::SetEqPreset(eq_options_clone[idx].clone())
                })
                .width(Length::Fixed(200.0)),
            ]
            .spacing(4),
        );
    }

    content = content.push(horizontal_rule(1));

    // Sound quality preference
    let quality = sound.get("quality_preference").cloned();
    let quality_options: Vec<String> = sound
        .get("quality_preference_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if !quality_options.is_empty() {
        let quality_labels: Vec<String> = quality_options
            .iter()
            .map(|s| match s.as_str() {
                "sqp_connectivity" => "Connectivity Priority".to_string(),
                "sqp_quality" => "Sound Quality Priority".to_string(),
                other => other.to_string(),
            })
            .collect();
        let current_quality_label = quality.as_ref().map(|s| match s.as_str() {
            "sqp_connectivity" => "Connectivity Priority".to_string(),
            "sqp_quality" => "Sound Quality Priority".to_string(),
            other => other.to_string(),
        });

        content = content.push(
            column![
                text("Sound Quality Preference").size(14),
                pick_list(quality_labels, current_quality_label, |selected: String| {
                    let value = if selected.contains("Connectivity") {
                        "sqp_connectivity"
                    } else {
                        "sqp_quality"
                    };
                    Message::SetSoundQuality(value.to_string())
                })
                .width(Length::Fixed(250.0)),
            ]
            .spacing(4),
        );
    }

    // Low latency
    let low_latency = config
        .get("low_latency")
        .map(|s| s == "true")
        .unwrap_or(false);

    content = content.push(
        row![
            text("Low Latency Mode").size(14),
            toggler(low_latency).on_toggle(|v| Message::SetLowLatency(v)),
        ]
        .spacing(12),
    );

    container(content).padding(20).width(Length::Fill).into()
}

fn eq_display_name(key: &str) -> String {
    match key {
        "equalizer_preset_default" => "Default".to_string(),
        "equalizer_preset_hardbass" => "Bass Boost".to_string(),
        "equalizer_preset_treble" => "Treble Boost".to_string(),
        "equalizer_preset_voices" | "equalizer_preset_voice" => "Voice".to_string(),
        other => other.replace("equalizer_preset_", "").replace('_', " "),
    }
}
