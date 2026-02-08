use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, pick_list, row, text};
use iced::{Element, Length};

use crate::ui::Message;

pub fn view(actions: &HashMap<String, String>) -> Element<'_, Message> {
    let mut content = column![text("Gesture Settings").size(18)].spacing(12);

    // Double tap
    let dt_options = parse_options(actions.get("double_tap_options"));
    if !dt_options.is_empty() {
        content = content.push(text("Double Tap").size(16));
        content = content.push(gesture_row(
            "Left:",
            actions.get("double_tap_left").cloned(),
            dt_options.clone(),
            "double_tap_left",
        ));
        content = content.push(gesture_row(
            "Right:",
            actions.get("double_tap_right").cloned(),
            dt_options,
            "double_tap_right",
        ));
        content = content.push(horizontal_rule(1));
    }

    // Triple tap
    let tt_options = parse_options(actions.get("triple_tap_options"));
    if !tt_options.is_empty() {
        content = content.push(text("Triple Tap").size(16));
        content = content.push(gesture_row(
            "Left:",
            actions.get("triple_tap_left").cloned(),
            tt_options.clone(),
            "triple_tap_left",
        ));
        content = content.push(gesture_row(
            "Right:",
            actions.get("triple_tap_right").cloned(),
            tt_options,
            "triple_tap_right",
        ));
        content = content.push(horizontal_rule(1));
    }

    // Long tap
    let lt_options = parse_options(actions.get("long_tap_options"));
    if !lt_options.is_empty() {
        content = content.push(text("Long Tap").size(16));
        content = content.push(gesture_row(
            "Left:",
            actions.get("long_tap_left").cloned(),
            lt_options.clone(),
            "long_tap_left",
        ));

        if actions.contains_key("long_tap_right") {
            content = content.push(gesture_row(
                "Right:",
                actions.get("long_tap_right").cloned(),
                lt_options,
                "long_tap_right",
            ));
        }
        content = content.push(horizontal_rule(1));
    }

    // Noise control cycle
    let nc_options = parse_options(actions.get("noise_control_options"));
    if !nc_options.is_empty() {
        content = content.push(text("ANC Cycle Mode").size(16));
        content = content.push(gesture_row(
            "Left:",
            actions.get("noise_control_left").cloned(),
            nc_options.clone(),
            "noise_control_left",
        ));
        if actions.contains_key("noise_control_right") {
            content = content.push(gesture_row(
                "Right:",
                actions.get("noise_control_right").cloned(),
                nc_options,
                "noise_control_right",
            ));
        }
        content = content.push(horizontal_rule(1));
    }

    // Swipe
    let swipe_options = parse_options(actions.get("swipe_gesture_options"));
    if !swipe_options.is_empty() {
        content = content.push(text("Swipe Gesture").size(16));
        content = content.push(gesture_row(
            "Action:",
            actions.get("swipe_gesture").cloned(),
            swipe_options,
            "swipe_gesture",
        ));
    }

    container(content).padding(20).width(Length::Fill).into()
}

fn gesture_row<'a>(
    label: &'a str,
    current: Option<String>,
    options: Vec<String>,
    prop_name: &'static str,
) -> Element<'a, Message> {
    let display_options: Vec<String> = options.iter().map(|s| gesture_display_name(s)).collect();
    let current_display = current.as_ref().map(|s| gesture_display_name(s));

    let options_clone = options.clone();
    let display_clone = display_options.clone();

    row![
        text(label).size(14).width(Length::Fixed(80.0)),
        pick_list(display_options, current_display, move |selected: String| {
            let idx = display_clone
                .iter()
                .position(|s| *s == selected)
                .unwrap_or(0);
            Message::SetGesture(prop_name.to_string(), options_clone[idx].clone())
        })
        .width(Length::Fixed(200.0)),
    ]
    .spacing(8)
    .into()
}

fn parse_options(raw: Option<&String>) -> Vec<String> {
    raw.map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default()
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
