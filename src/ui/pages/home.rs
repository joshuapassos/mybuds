use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, row, text, toggler, Space};
use iced::{Alignment, Element, Length};

use crate::ui::widgets::anc_selector::{anc_level_selector, anc_mode_selector};
use crate::ui::widgets::battery_indicator::battery_display;
use crate::ui::Message;

pub fn view<'a>(
    battery: &'a HashMap<String, String>,
    anc: &'a HashMap<String, String>,
    info: &'a HashMap<String, String>,
    ear_detection: &'a HashMap<String, String>,
    conversation_awareness: &'a HashMap<String, String>,
    personalized_volume: &'a HashMap<String, String>,
    connected: bool,
) -> Element<'a, Message> {
    if !connected {
        return container(
            column![
                Space::with_height(40),
                text("MyBuds").size(24),
                Space::with_height(8),
                text("No device connected")
                    .size(16)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                Space::with_height(8),
                text("Pair your headphones via Bluetooth settings.")
                    .size(13)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .align_x(Alignment::Center),
        )
        .padding(40)
        .center_x(Length::Fill)
        .into();
    }

    // Device name from info store
    let device_model = info
        .get("device_name")
        .or_else(|| info.get("device_model"))
        .or_else(|| info.get("field_15"))
        .cloned()
        .unwrap_or_else(|| "Headphones".into());
    let sw_version = info
        .get("software_ver")
        .or_else(|| info.get("firmware_ver_1"))
        .cloned();

    // Header with device name
    let mut header = column![text(device_model.clone()).size(22),].align_x(Alignment::Center);

    if let Some(ver) = sw_version {
        header = header.push(
            text(ver.clone())
                .size(12)
                .color(iced::Color::from_rgb(0.55, 0.55, 0.55)),
        );
    }

    let header_section = container(header).center_x(Length::Fill).padding(16);

    // Battery
    let left = battery.get("left").and_then(|s| s.parse().ok());
    let right = battery.get("right").and_then(|s| s.parse().ok());
    let case = battery.get("case").and_then(|s| s.parse().ok());
    let global = battery.get("global").and_then(|s| s.parse().ok());
    let is_charging = battery.get("is_charging").map_or(false, |s| s == "true");

    let battery_section = column![
        section_title("Battery"),
        battery_display(left, right, case, global, is_charging),
    ]
    .spacing(8);

    // ANC
    let anc_mode = anc.get("mode").cloned();
    let anc_options: Vec<String> = anc
        .get("mode_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    let anc_level = anc.get("level").cloned();
    let anc_level_options: Vec<String> = anc
        .get("level_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    let mut content = column![header_section, divider(), battery_section,]
        .spacing(12)
        .padding(20);

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

        content = content.push(divider());
        content = content.push(
            column![
                section_title("Ear Detection"),
                row![
                    text(format!("L: {}", ear_label(&primary))).size(14),
                    Space::with_width(20),
                    text(format!("R: {}", ear_label(&secondary))).size(14),
                ]
                .spacing(8),
            ]
            .spacing(8),
        );
    }

    // ANC modes
    if !anc_options.is_empty() {
        content = content.push(divider());
        content = content.push(anc_mode_selector(
            anc_mode.as_deref(),
            &anc_options,
            |mode| Message::SetAncMode(mode),
        ));

        if !anc_level_options.is_empty() {
            content = content.push(anc_level_selector(
                anc_level.as_deref(),
                &anc_level_options,
                |level| Message::SetAncLevel(level),
            ));
        }
    }

    // Conversational Awareness (AirPods)
    if !conversation_awareness.is_empty() {
        let ca_enabled = conversation_awareness
            .get("enabled")
            .map(|s| s == "true")
            .unwrap_or(false);

        content = content.push(divider());
        content = content.push(
            row![
                text("Conversational Awareness").size(14),
                toggler(ca_enabled).on_toggle(|v| Message::SetConversationAwareness(v)),
            ]
            .spacing(12),
        );
    }

    // Personalized Volume (AirPods)
    if !personalized_volume.is_empty() {
        let pv_enabled = personalized_volume
            .get("enabled")
            .map(|s| s == "true")
            .unwrap_or(false);

        content = content.push(
            row![
                text("Personalized Volume").size(14),
                toggler(pv_enabled).on_toggle(|v| Message::SetPersonalizedVolume(v)),
            ]
            .spacing(12),
        );
    }

    content.into()
}

fn section_title<'a, M: 'a>(title: &'a str) -> Element<'a, M> {
    text(title)
        .size(14)
        .color(iced::Color::from_rgb(0.4, 0.4, 0.4))
        .into()
}

fn divider<'a, M: 'a>() -> Element<'a, M> {
    container(horizontal_rule(1)).padding(4).into()
}

fn ear_label(state: &str) -> &str {
    match state {
        "in_ear" => "In Ear",
        "out" => "Out",
        "in_case" => "In Case",
        _ => "Unknown",
    }
}
