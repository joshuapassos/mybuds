use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, row, text, toggler, Space};
use iced::{Border, Element, Length, Theme};
use serde_json::Value;

use crate::ui::Message;

#[derive(Debug)]
struct Device {
    name: String,
    connected: bool,
    playing: bool,
    auto_connect: bool,
}

fn parse_devices(json_str: &str) -> Vec<Device> {
    let Ok(parsed) = serde_json::from_str::<HashMap<String, Value>>(json_str) else {
        return Vec::new();
    };

    let mut devices: Vec<(String, Device)> = parsed
        .into_iter()
        .map(|(mac, obj)| {
            let device = Device {
                name: obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Device")
                    .to_string(),
                connected: obj
                    .get("connected")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                playing: obj.get("playing").and_then(|v| v.as_bool()).unwrap_or(false),
                auto_connect: obj
                    .get("auto_connect")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            };
            (mac, device)
        })
        .collect();

    // Sort by MAC address to keep consistent order
    devices.sort_by(|a, b| a.0.cmp(&b.0));

    devices.into_iter().map(|(_, device)| device).collect()
}

fn device_card(name: String, connected: bool, playing: bool, auto_connect: bool) -> Element<'static, Message> {
    let status_icon = if connected { "●" } else { "○" };
    let status_color = if connected {
        iced::Color::from_rgb(0.0, 0.8, 0.0) // Green
    } else {
        iced::Color::from_rgb(0.5, 0.5, 0.5) // Gray
    };

    let name_row = row![
        text(status_icon).size(20).color(status_color),
        text(name).size(16),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut details = row![].spacing(16);

    if connected {
        details = details.push(text("Connected").size(12).color(iced::Color::from_rgb(0.0, 0.6, 0.0)));
    }

    if playing {
        details = details.push(text("Playing").size(12).color(iced::Color::from_rgb(0.3, 0.5, 0.9)));
    }

    if auto_connect {
        details = details.push(text("Auto-connect").size(12).color(iced::Color::from_rgb(0.5, 0.5, 0.5)));
    }

    let card_content = column![name_row, details].spacing(6);

    container(card_content)
        .padding(12)
        .style(|theme: &Theme| {
            let base_color = theme.palette().background;
            let border_color = iced::Color {
                r: base_color.r * 0.8,
                g: base_color.g * 0.8,
                b: base_color.b * 0.8,
                a: 1.0,
            };
            let bg_color = iced::Color {
                r: base_color.r * 0.95,
                g: base_color.g * 0.95,
                b: base_color.b * 0.95,
                a: 1.0,
            };

            container::Style {
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                background: Some(bg_color.into()),
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
}

pub fn view(dc: &HashMap<String, String>) -> Element<'_, Message> {
    let mut content = column![text("Dual Connect").size(18)].spacing(12);

    let enabled = dc.get("enabled").map(|s| s == "true").unwrap_or(false);

    content = content.push(
        row![
            text("Dual Connect").size(14),
            toggler(enabled).on_toggle(|v| Message::SetDualConnect(v)),
        ]
        .spacing(12),
    );

    if enabled {
        content = content.push(horizontal_rule(1));
        content = content.push(Space::with_height(8));
        content = content.push(text("Connected Devices").size(16));

        let devices_json = dc.get("devices").cloned().unwrap_or_default();
        let devices = parse_devices(&devices_json);

        if devices.is_empty() {
            content = content.push(
                container(text("No devices paired").size(14).color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
                    .padding(12)
            );
        } else {
            for device in devices {
                content = content.push(device_card(
                    device.name,
                    device.connected,
                    device.playing,
                    device.auto_connect,
                ));
            }
        }
    }

    container(content).padding(20).width(Length::Fill).into()
}
