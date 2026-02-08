use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::tui::{Action, PageState};

struct GestureItem {
    label: String,
    value: String,
    options: Vec<String>,
    prop_name: &'static str,
    group: &'static str,
}

fn build_items(actions: &HashMap<String, String>) -> Vec<GestureItem> {
    let mut items = Vec::new();

    // Double tap
    let dt_opts = parse_options(actions.get("double_tap_options"));
    if !dt_opts.is_empty() {
        items.push(GestureItem {
            label: "Double Tap Left".into(),
            value: gesture_display(actions.get("double_tap_left")),
            options: dt_opts.clone(),
            prop_name: "double_tap_left",
            group: "gesture_double",
        });
        items.push(GestureItem {
            label: "Double Tap Right".into(),
            value: gesture_display(actions.get("double_tap_right")),
            options: dt_opts,
            prop_name: "double_tap_right",
            group: "gesture_double",
        });
    }

    // Triple tap
    let tt_opts = parse_options(actions.get("triple_tap_options"));
    if !tt_opts.is_empty() {
        items.push(GestureItem {
            label: "Triple Tap Left".into(),
            value: gesture_display(actions.get("triple_tap_left")),
            options: tt_opts.clone(),
            prop_name: "triple_tap_left",
            group: "gesture_triple",
        });
        items.push(GestureItem {
            label: "Triple Tap Right".into(),
            value: gesture_display(actions.get("triple_tap_right")),
            options: tt_opts,
            prop_name: "triple_tap_right",
            group: "gesture_triple",
        });
    }

    // Long tap
    let lt_opts = parse_options(actions.get("long_tap_options"));
    if !lt_opts.is_empty() {
        items.push(GestureItem {
            label: "Long Tap Left".into(),
            value: gesture_display(actions.get("long_tap_left")),
            options: lt_opts.clone(),
            prop_name: "long_tap_left",
            group: "gesture_long_split",
        });
        if actions.contains_key("long_tap_right") {
            items.push(GestureItem {
                label: "Long Tap Right".into(),
                value: gesture_display(actions.get("long_tap_right")),
                options: lt_opts,
                prop_name: "long_tap_right",
                group: "gesture_long_split",
            });
        }
    }

    // Noise control cycle
    let nc_opts = parse_options(actions.get("noise_control_options"));
    if !nc_opts.is_empty() {
        items.push(GestureItem {
            label: "ANC Cycle Left".into(),
            value: gesture_display(actions.get("noise_control_left")),
            options: nc_opts.clone(),
            prop_name: "noise_control_left",
            group: "gesture_long_split",
        });
        if actions.contains_key("noise_control_right") {
            items.push(GestureItem {
                label: "ANC Cycle Right".into(),
                value: gesture_display(actions.get("noise_control_right")),
                options: nc_opts,
                prop_name: "noise_control_right",
                group: "gesture_long_split",
            });
        }
    }

    // Swipe
    let sw_opts = parse_options(actions.get("swipe_gesture_options"));
    if !sw_opts.is_empty() {
        items.push(GestureItem {
            label: "Swipe Gesture".into(),
            value: gesture_display(actions.get("swipe_gesture")),
            options: sw_opts,
            prop_name: "swipe_gesture",
            group: "gesture_swipe",
        });
    }

    items
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    actions: &HashMap<String, String>,
    state: &mut PageState,
) {
    let items = build_items(actions);
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
        .block(Block::default().borders(Borders::ALL).title("Gesture Settings (h/l to cycle)"));
    frame.render_widget(list, area);
}

pub fn on_cycle(
    actions: &HashMap<String, String>,
    state: &PageState,
    direction: i32,
) -> Action {
    let items = build_items(actions);
    if state.selected >= items.len() {
        return Action::None;
    }

    let item = &items[state.selected];
    let current = actions.get(item.prop_name);
    let new_val = cycle_option(&item.options, current.map(|s| s.as_str()), direction);
    if let Some(val) = new_val {
        Action::SetProperty {
            group: item.group.into(),
            prop: item.prop_name.into(),
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

fn parse_options(raw: Option<&String>) -> Vec<String> {
    raw.map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default()
}

fn gesture_display(val: Option<&String>) -> String {
    val.map(|s| gesture_display_name(s)).unwrap_or_else(|| "—".into())
}

fn gesture_display_name(name: &str) -> String {
    match name {
        "tap_action_off" => "Disabled".into(),
        "tap_action_pause" => "Play/Pause".into(),
        "tap_action_next" => "Next Track".into(),
        "tap_action_prev" => "Previous Track".into(),
        "tap_action_assistant" => "Voice Assistant".into(),
        "tap_action_answer" => "Answer Call".into(),
        "tap_action_switch_anc" => "Switch ANC".into(),
        "tap_action_change_volume" => "Volume Control".into(),
        "noise_control_off_on" => "Off / NC".into(),
        "noise_control_off_on_aw" => "Off / NC / Awareness".into(),
        "noise_control_on_aw" => "NC / Awareness".into(),
        "noise_control_off_aw" => "Off / Awareness".into(),
        other => other.replace('_', " "),
    }
}
