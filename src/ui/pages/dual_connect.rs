use std::collections::HashMap;
use std::ffi::CStr;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use relm4::ComponentSender;
use serde_json::Value;

use crate::ui::{Message, MyBudsApp};

fn get_hostname() -> String {
    unsafe {
        let mut buf = [0u8; 256];
        if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) == 0 {
            CStr::from_ptr(buf.as_ptr() as *const libc::c_char)
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        }
    }
}

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

    let hostname = get_hostname().to_lowercase();

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
                playing: obj
                    .get("playing")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                auto_connect: obj
                    .get("auto_connect")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            };
            (mac, device)
        })
        .collect();

    devices.sort_by(|a, b| {
        let a_is_this_pc = !hostname.is_empty() && a.1.name.to_lowercase().contains(&hostname);
        let b_is_this_pc = !hostname.is_empty() && b.1.name.to_lowercase().contains(&hostname);
        match (a_is_this_pc, b_is_this_pc) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        }
    });

    devices.into_iter().map(|(_, device)| device).collect()
}

pub fn build(
    container: &gtk4::Box,
    dc: &HashMap<String, String>,
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

    let enabled = dc.get("enabled").map(|s| s == "true").unwrap_or(false);

    // Toggle
    let toggle_group = adw::PreferencesGroup::new();
    let toggle_row = adw::ActionRow::builder()
        .title("Dual Connect")
        .build();
    let switch = gtk4::Switch::builder()
        .active(enabled)
        .valign(gtk4::Align::Center)
        .build();
    let s = sender.clone();
    switch.connect_state_set(move |_, active| {
        s.input(Message::SetDualConnect(active));
        gtk4::glib::Propagation::Proceed
    });
    toggle_row.add_suffix(&switch);
    toggle_row.set_activatable_widget(Some(&switch));
    toggle_group.add(&toggle_row);
    content.append(&toggle_group.upcast::<gtk4::Widget>());

    if enabled {
        let devices_json = dc.get("devices").cloned().unwrap_or_default();
        let devices = parse_devices(&devices_json);

        let devices_group = adw::PreferencesGroup::builder()
            .title("Connected Devices")
            .build();

        if devices.is_empty() {
            let row = adw::ActionRow::builder()
                .title("No devices paired")
                .build();
            devices_group.add(&row);
        } else {
            let hostname = get_hostname();
            for device in devices {
                let is_this_pc = !hostname.is_empty()
                    && device
                        .name
                        .to_lowercase()
                        .contains(&hostname.to_lowercase());
                let connected = device.connected || is_this_pc;

                let device_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
                device_box.add_css_class("device-card");

                // Name row
                let name_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                name_row.set_valign(gtk4::Align::Center);

                let status_icon = if connected { "\u{25CF}" } else { "\u{25CB}" };
                let status_class = if connected {
                    "status-connected"
                } else {
                    "status-disconnected"
                };
                let icon_label = gtk4::Label::builder()
                    .label(status_icon)
                    .css_classes([status_class])
                    .build();
                name_row.append(&icon_label);

                let name_label = gtk4::Label::builder()
                    .label(&device.name)
                    .css_classes(["title-4"])
                    .build();
                name_row.append(&name_label);

                if is_this_pc {
                    let badge = gtk4::Label::builder()
                        .label("This PC")
                        .css_classes(["this-pc-badge"])
                        .build();
                    name_row.append(&badge);
                }

                device_box.append(&name_row);

                // Details row
                let details = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
                if connected {
                    details.append(
                        &gtk4::Label::builder()
                            .label("Connected")
                            .css_classes(["caption", "success"])
                            .build(),
                    );
                }
                if device.playing {
                    details.append(
                        &gtk4::Label::builder()
                            .label("Playing")
                            .css_classes(["caption", "accent"])
                            .build(),
                    );
                }
                if device.auto_connect {
                    details.append(
                        &gtk4::Label::builder()
                            .label("Auto-connect")
                            .css_classes(["caption", "dim-label"])
                            .build(),
                    );
                }
                device_box.append(&details);
                devices_group.add(&device_box);
            }
        }

        content.append(&devices_group.upcast::<gtk4::Widget>());
    }

    clamp.set_child(Some(&content));
    container.append(&clamp);
}
