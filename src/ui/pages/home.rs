use std::collections::HashMap;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use relm4::ComponentSender;

use crate::ui::widgets::anc_selector;
use crate::ui::widgets::battery_indicator;
use crate::ui::{Message, MyBudsApp};

pub fn build(
    container: &gtk4::Box,
    battery: &HashMap<String, String>,
    anc: &HashMap<String, String>,
    info: &HashMap<String, String>,
    ear_detection: &HashMap<String, String>,
    conversation_awareness: &HashMap<String, String>,
    personalized_volume: &HashMap<String, String>,
    connected: bool,
    sender: &ComponentSender<MyBudsApp>,
) {
    // Clear previous content
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if !connected {
        let status = adw::StatusPage::builder()
            .title("MyBuds")
            .description("No device connected.\nPair your headphones via Bluetooth settings.")
            .icon_name("audio-headphones-symbolic")
            .build();
        container.append(&status);
        return;
    }

    let clamp = adw::Clamp::builder()
        .maximum_size(500)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

    // Header: device name + firmware
    let device_model = info
        .get("device_name")
        .or_else(|| info.get("device_model"))
        .or_else(|| info.get("field_15"))
        .cloned()
        .unwrap_or_else(|| "Headphones".into());

    let header = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    header.set_halign(gtk4::Align::Center);

    let name_label = gtk4::Label::builder()
        .label(&device_model)
        .css_classes(["title-2"])
        .build();
    header.append(&name_label);

    if let Some(ver) = info
        .get("software_ver")
        .or_else(|| info.get("firmware_ver_1"))
    {
        let ver_label = gtk4::Label::builder()
            .label(ver)
            .css_classes(["dim-label", "caption"])
            .build();
        header.append(&ver_label);
    }
    content.append(&header);

    // Battery
    let left = battery.get("left").and_then(|s| s.parse().ok());
    let right = battery.get("right").and_then(|s| s.parse().ok());
    let case = battery.get("case").and_then(|s| s.parse().ok());
    let global = battery.get("global").and_then(|s| s.parse().ok());
    let is_charging = battery.get("is_charging").map_or(false, |s| s == "true");

    let battery_group = adw::PreferencesGroup::builder()
        .title("Battery")
        .build();
    let battery_widget = battery_indicator::battery_display(left, right, case, global, is_charging);
    battery_group.add(&battery_widget);
    content.append(&battery_group.upcast::<gtk4::Widget>());

    // Ear detection (AirPods)
    if !ear_detection.is_empty() {
        let primary = ear_detection
            .get("primary")
            .cloned()
            .unwrap_or_else(|| "unknown".into());
        let secondary = ear_detection
            .get("secondary")
            .cloned()
            .unwrap_or_else(|| "unknown".into());

        let ear_group = adw::PreferencesGroup::builder()
            .title("Ear Detection")
            .build();

        let ear_row = adw::ActionRow::builder()
            .title(&format!("L: {}  /  R: {}", ear_label(&primary), ear_label(&secondary)))
            .build();
        ear_group.add(&ear_row);
        content.append(&ear_group.upcast::<gtk4::Widget>());
    }

    // ANC
    let anc_options: Vec<String> = anc
        .get("mode_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if !anc_options.is_empty() {
        let anc_mode = anc.get("mode").cloned();
        let anc_group = adw::PreferencesGroup::builder()
            .title("Noise Control")
            .build();

        let mode_widget = anc_selector::anc_mode_buttons(anc_mode.as_deref(), &anc_options, sender);
        anc_group.add(&mode_widget);

        // ANC level
        let anc_level_options: Vec<String> = anc
            .get("level_options")
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_default();

        if !anc_level_options.is_empty() {
            let anc_level = anc.get("level").cloned();
            let level_widget =
                anc_selector::anc_level_buttons(anc_level.as_deref(), &anc_level_options, sender);
            anc_group.add(&level_widget);
        }

        content.append(&anc_group.upcast::<gtk4::Widget>());
    }

    // Conversational Awareness (AirPods)
    if !conversation_awareness.is_empty() {
        let ca_enabled = conversation_awareness
            .get("enabled")
            .map(|s| s == "true")
            .unwrap_or(false);

        let group = adw::PreferencesGroup::new();
        let row = adw::ActionRow::builder()
            .title("Conversational Awareness")
            .build();
        let switch = gtk4::Switch::builder()
            .active(ca_enabled)
            .valign(gtk4::Align::Center)
            .build();
        let s = sender.clone();
        switch.connect_state_set(move |_, active| {
            s.input(Message::SetConversationAwareness(active));
            gtk4::glib::Propagation::Proceed
        });
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));
        group.add(&row);
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Personalized Volume (AirPods)
    if !personalized_volume.is_empty() {
        let pv_enabled = personalized_volume
            .get("enabled")
            .map(|s| s == "true")
            .unwrap_or(false);

        let group = adw::PreferencesGroup::new();
        let row = adw::ActionRow::builder()
            .title("Personalized Volume")
            .build();
        let switch = gtk4::Switch::builder()
            .active(pv_enabled)
            .valign(gtk4::Align::Center)
            .build();
        let s = sender.clone();
        switch.connect_state_set(move |_, active| {
            s.input(Message::SetPersonalizedVolume(active));
            gtk4::glib::Propagation::Proceed
        });
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));
        group.add(&row);
        content.append(&group.upcast::<gtk4::Widget>());
    }

    clamp.set_child(Some(&content));
    container.append(&clamp);
}

fn ear_label(state: &str) -> &str {
    match state {
        "in_ear" => "In Ear",
        "out" => "Out",
        "in_case" => "In Case",
        _ => "Unknown",
    }
}
