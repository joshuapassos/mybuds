use std::collections::HashMap;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use relm4::ComponentSender;

use crate::ui::{Message, MyBudsApp};

pub fn build(
    container: &gtk4::Box,
    actions: &HashMap<String, String>,
    sender: &ComponentSender<MyBudsApp>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let clamp = adw::Clamp::builder()
        .maximum_size(500)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

    // Double tap
    let dt_options = parse_options(actions.get("double_tap_options"));
    if !dt_options.is_empty() {
        let group = adw::PreferencesGroup::builder()
            .title("Double Tap")
            .build();
        group.add(&gesture_combo_row(
            "Left",
            actions.get("double_tap_left").cloned(),
            &dt_options,
            "double_tap_left",
            sender,
        ));
        group.add(&gesture_combo_row(
            "Right",
            actions.get("double_tap_right").cloned(),
            &dt_options,
            "double_tap_right",
            sender,
        ));
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Triple tap
    let tt_options = parse_options(actions.get("triple_tap_options"));
    if !tt_options.is_empty() {
        let group = adw::PreferencesGroup::builder()
            .title("Triple Tap")
            .build();
        group.add(&gesture_combo_row(
            "Left",
            actions.get("triple_tap_left").cloned(),
            &tt_options,
            "triple_tap_left",
            sender,
        ));
        group.add(&gesture_combo_row(
            "Right",
            actions.get("triple_tap_right").cloned(),
            &tt_options,
            "triple_tap_right",
            sender,
        ));
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Long tap
    let lt_options = parse_options(actions.get("long_tap_options"));
    if !lt_options.is_empty() {
        let group = adw::PreferencesGroup::builder()
            .title("Long Tap")
            .build();
        group.add(&gesture_combo_row(
            "Left",
            actions.get("long_tap_left").cloned(),
            &lt_options,
            "long_tap_left",
            sender,
        ));
        if actions.contains_key("long_tap_right") {
            group.add(&gesture_combo_row(
                "Right",
                actions.get("long_tap_right").cloned(),
                &lt_options,
                "long_tap_right",
                sender,
            ));
        }
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Noise control cycle
    let nc_options = parse_options(actions.get("noise_control_options"));
    if !nc_options.is_empty() {
        let group = adw::PreferencesGroup::builder()
            .title("ANC Cycle Mode")
            .build();
        group.add(&gesture_combo_row(
            "Left",
            actions.get("noise_control_left").cloned(),
            &nc_options,
            "noise_control_left",
            sender,
        ));
        if actions.contains_key("noise_control_right") {
            group.add(&gesture_combo_row(
                "Right",
                actions.get("noise_control_right").cloned(),
                &nc_options,
                "noise_control_right",
                sender,
            ));
        }
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Swipe
    let swipe_options = parse_options(actions.get("swipe_gesture_options"));
    if !swipe_options.is_empty() {
        let group = adw::PreferencesGroup::builder()
            .title("Swipe Gesture")
            .build();
        group.add(&gesture_combo_row(
            "Action",
            actions.get("swipe_gesture").cloned(),
            &swipe_options,
            "swipe_gesture",
            sender,
        ));
        content.append(&group.upcast::<gtk4::Widget>());
    }

    clamp.set_child(Some(&content));
    container.append(&clamp);
}

fn gesture_combo_row(
    label: &str,
    current: Option<String>,
    options: &[String],
    prop_name: &'static str,
    sender: &ComponentSender<MyBudsApp>,
) -> adw::ComboRow {
    let combo = adw::ComboRow::builder().title(label).build();

    let display_labels: Vec<String> = options.iter().map(|s| gesture_display_name(s)).collect();
    let string_list =
        gtk4::StringList::new(&display_labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    combo.set_model(Some(&string_list));

    if let Some(ref current) = current {
        if let Some(pos) = options.iter().position(|o| o == current) {
            combo.set_selected(pos as u32);
        }
    }

    let s = sender.clone();
    let opts = options.to_vec();
    combo.connect_selected_notify(move |c| {
        let idx = c.selected() as usize;
        if idx < opts.len() {
            s.input(Message::SetGesture(
                prop_name.to_string(),
                opts[idx].clone(),
            ));
        }
    });

    combo
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
