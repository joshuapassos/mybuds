use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::tui::{Action, PageState};

struct SoundItem {
    label: String,
    value: String,
    options: Vec<String>,
    group: &'static str,
    prop: &'static str,
}

fn build_items(sound: &HashMap<String, String>, config: &HashMap<String, String>) -> Vec<SoundItem> {
    let mut items = Vec::new();

    // EQ preset
    let eq_options: Vec<String> = sound
        .get("equalizer_preset_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    if !eq_options.is_empty() {
        let current = sound.get("equalizer_preset").cloned().unwrap_or_default();
        items.push(SoundItem {
            label: "EQ Preset".into(),
            value: eq_display_name(&current),
            options: eq_options,
            group: "config_eq",
            prop: "equalizer_preset",
        });
    }

    // Sound quality preference
    let quality_options: Vec<String> = sound
        .get("quality_preference_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    if !quality_options.is_empty() {
        let current = sound.get("quality_preference").cloned().unwrap_or_default();
        items.push(SoundItem {
            label: "Sound Quality".into(),
            value: quality_display_name(&current),
            options: quality_options,
            group: "config_sound_quality",
            prop: "quality_preference",
        });
    }

    // Low latency toggle
    let low_latency = config.get("low_latency").map(|s| s == "true").unwrap_or(false);
    items.push(SoundItem {
        label: "Low Latency".into(),
        value: if low_latency { "ON".into() } else { "OFF".into() },
        options: vec!["true".into(), "false".into()],
        group: "low_latency",
        prop: "low_latency",
    });

    items
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    sound: &HashMap<String, String>,
    config: &HashMap<String, String>,
    state: &mut PageState,
) {
    let items = build_items(sound, config);
    state.item_count = items.len();
    state.clamp();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_focused = i == state.selected;
            let line = format!("{}: {}", item.label, item.value);
            let style = if is_focused {
                Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Sound Settings (h/l to cycle)"));
    frame.render_widget(list, area);
}

pub fn on_enter(
    sound: &HashMap<String, String>,
    config: &HashMap<String, String>,
    state: &PageState,
) -> Action {
    on_cycle(sound, config, state, 1)
}

pub fn on_cycle(
    sound: &HashMap<String, String>,
    config: &HashMap<String, String>,
    state: &PageState,
    direction: i32,
) -> Action {
    let items = build_items(sound, config);
    if state.selected >= items.len() {
        return Action::None;
    }

    let item = &items[state.selected];

    if item.prop == "low_latency" {
        // Toggle
        let current = config.get("low_latency").map(|s| s == "true").unwrap_or(false);
        let new_val = if current { "false" } else { "true" };
        return Action::SetProperty {
            group: item.group.into(),
            prop: item.prop.into(),
            value: new_val.into(),
        };
    }

    // Cycle through options
    let current = if item.prop == "equalizer_preset" {
        sound.get(item.prop)
    } else {
        sound.get(item.prop)
    };
    let new_val = cycle_option(&item.options, current.map(|s| s.as_str()), direction);
    if let Some(val) = new_val {
        Action::SetProperty {
            group: item.group.into(),
            prop: item.prop.into(),
            value: val,
        }
    } else {
        Action::None
    }
}

fn cycle_option(options: &[String], current: Option<&str>, direction: i32) -> Option<String> {
    if options.is_empty() {
        return None;
    }
    let cur_idx = current
        .and_then(|c| options.iter().position(|o| o == c))
        .unwrap_or(0);
    let new_idx = if direction >= 0 {
        (cur_idx + 1) % options.len()
    } else {
        if cur_idx == 0 { options.len() - 1 } else { cur_idx - 1 }
    };
    Some(options[new_idx].clone())
}

fn eq_display_name(key: &str) -> String {
    match key {
        "equalizer_preset_default" => "Default".into(),
        "equalizer_preset_hardbass" => "Bass Boost".into(),
        "equalizer_preset_treble" => "Treble Boost".into(),
        "equalizer_preset_voices" | "equalizer_preset_voice" => "Voice".into(),
        other => other.replace("equalizer_preset_", "").replace('_', " "),
    }
}

fn quality_display_name(key: &str) -> String {
    match key {
        "sqp_connectivity" => "Connectivity Priority".into(),
        "sqp_quality" => "Sound Quality Priority".into(),
        other => other.to_string(),
    }
}
