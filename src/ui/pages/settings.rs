use std::collections::HashMap;

use iced::widget::{column, container, row, text, toggler};
use iced::{Element, Length};

use crate::ui::Message;

pub fn view(config: &HashMap<String, String>) -> Element<'_, Message> {
    let mut content = column![text("Settings").size(18)].spacing(12);

    // Auto-pause
    let auto_pause = config
        .get("auto_pause")
        .map(|s| s == "true")
        .unwrap_or(false);

    content = content.push(
        row![
            text("Auto-pause on ear removal").size(14),
            toggler(auto_pause).on_toggle(|v| Message::SetAutoPause(v)),
        ]
        .spacing(12),
    );

    content = content.push(
        column![
            text("About").size(16),
            text("FreeBuds Manager v0.1.0").size(12),
            text("Manages Huawei FreeBuds headphones via SPP protocol").size(12),
            text("Based on the OpenFreebuds project").size(12),
        ]
        .spacing(4),
    );

    container(content).padding(20).width(Length::Fill).into()
}
