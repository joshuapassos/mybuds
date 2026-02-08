use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::tui::{Action, PageState};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    dc: &HashMap<String, String>,
    state: &mut PageState,
) {
    let enabled = dc.get("enabled").map(|s| s == "true").unwrap_or(false);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // toggle
            Constraint::Min(0),   // device list
        ])
        .split(area);

    // Toggle item
    state.item_count = 1;
    state.clamp();

    let toggle_text = format!("Dual Connect: {}", if enabled { "ON" } else { "OFF" });
    let toggle_style = if state.selected == 0 {
        Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let toggle = List::new(vec![ListItem::new(toggle_text).style(toggle_style)])
        .block(Block::default().borders(Borders::ALL).title("Dual Connect (Enter to toggle)"));
    frame.render_widget(toggle, chunks[0]);

    // Paired devices info
    if enabled {
        let devices_json = dc.get("devices").cloned().unwrap_or_default();
        let text = if !devices_json.is_empty() && devices_json != "{}" {
            format!("Connected Devices:\n{}", devices_json)
        } else {
            "No devices paired".into()
        };
        let para = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Devices"));
        frame.render_widget(para, chunks[1]);
    }
}

pub fn on_enter(dc: &HashMap<String, String>, state: &PageState) -> Action {
    on_cycle(dc, state, 0)
}

pub fn on_cycle(dc: &HashMap<String, String>, state: &PageState, _direction: i32) -> Action {
    if state.selected == 0 {
        let enabled = dc.get("enabled").map(|s| s == "true").unwrap_or(false);
        Action::SetProperty {
            group: "dual_connect".into(),
            prop: "enabled".into(),
            value: if enabled { "false" } else { "true" }.into(),
        }
    } else {
        Action::None
    }
}
