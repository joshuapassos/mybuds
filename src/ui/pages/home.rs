use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::ui::widgets::anc_selector::{anc_level_selector, anc_mode_selector};
use crate::ui::widgets::battery_indicator::battery_display;
use crate::ui::Message;

pub fn view<'a>(
    battery: &'a HashMap<String, String>,
    anc: &'a HashMap<String, String>,
    info: &'a HashMap<String, String>,
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
                text("Pair your FreeBuds via Bluetooth settings.")
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
        .get("device_model")
        .or_else(|| info.get("field_15"))
        .cloned()
        .unwrap_or_else(|| "FreeBuds".into());
    let sw_version = info.get("software_ver").cloned();

    // Header with device name
    let mut header = column![
        text(device_model.clone()).size(22),
    ]
    .align_x(Alignment::Center);

    if let Some(ver) = sw_version {
        header = header.push(
            text(ver.clone())
                .size(12)
                .color(iced::Color::from_rgb(0.55, 0.55, 0.55)),
        );
    }

    let header_section = container(header)
        .center_x(Length::Fill)
        .padding(16);

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

    let mut content = column![
        header_section,
        divider(),
        battery_section,
    ]
    .spacing(12)
    .padding(20);

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

    content.into()
}

fn section_title<'a, M: 'a>(title: &'a str) -> Element<'a, M> {
    text(title)
        .size(14)
        .color(iced::Color::from_rgb(0.4, 0.4, 0.4))
        .into()
}

fn divider<'a, M: 'a>() -> Element<'a, M> {
    container(horizontal_rule(1))
        .padding(4)
        .into()
}
