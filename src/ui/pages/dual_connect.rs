use std::collections::HashMap;

use iced::widget::{column, container, horizontal_rule, row, text, toggler};
use iced::{Element, Length};

use crate::ui::Message;

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

        // Parse devices JSON (simplified display)
        let devices_json = dc.get("devices").cloned().unwrap_or_default();
        if !devices_json.is_empty() && devices_json != "{}" {
            content = content.push(text("Connected Devices").size(16));
            content = content.push(text(devices_json).size(12));
        } else {
            content = content.push(text("No devices paired").size(14));
        }
    }

    container(content).padding(20).width(Length::Fill).into()
}
