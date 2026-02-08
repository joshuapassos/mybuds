use std::collections::HashMap;

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::ui::Message;

pub fn view(info: &HashMap<String, String>) -> Element<'_, Message> {
    let mut content = column![text("Device Info").size(18)].spacing(8);

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
            let value = value.clone();
            content = content.push(
                row![
                    text(format!("{}:", label))
                        .size(14)
                        .width(Length::Fixed(150.0)),
                    text(value).size(14),
                ]
                .spacing(8),
            );
        }
    }

    // Show any extra fields
    for (key, value) in info {
        if !fields.iter().any(|(k, _)| k == key) {
            let key = key.clone();
            let value = value.clone();
            content = content.push(
                row![
                    text(format!("{}:", key))
                        .size(14)
                        .width(Length::Fixed(150.0)),
                    text(value).size(14),
                ]
                .spacing(8),
            );
        }
    }

    container(content).padding(20).width(Length::Fill).into()
}
