use std::collections::HashMap;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn build(container: &gtk4::Box, info: &HashMap<String, String>) {
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

    let group = adw::PreferencesGroup::builder()
        .title("Device Info")
        .build();

    let fields = [
        ("device_model", "Model"),
        ("device_submodel", "Submodel"),
        ("hardware_ver", "Hardware Version"),
        ("software_ver", "Firmware Version"),
        ("serial_number", "Serial Number"),
        ("left_serial_number", "Left S/N"),
        ("right_serial_number", "Right S/N"),
    ];

    for (key, label) in &fields {
        if let Some(value) = info.get(*key) {
            let row = adw::ActionRow::builder()
                .title(*label)
                .subtitle(value)
                .build();
            group.add(&row);
        }
    }

    // Extra fields
    for (key, value) in info {
        if !fields.iter().any(|(k, _)| k == key) {
            let row = adw::ActionRow::builder()
                .title(key.as_str())
                .subtitle(value)
                .build();
            group.add(&row);
        }
    }

    clamp.set_child(Some(&group));
    container.append(&clamp);
}
