use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::tui::{Action, PageState};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    config: &HashMap<String, String>,
    state: &mut PageState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // toggles
            Constraint::Min(0),   // about
        ])
        .split(area);

    // Settings items
    let auto_pause = config.get("auto_pause").map(|s| s == "true").unwrap_or(false);

    state.item_count = 1;
    state.clamp();

    let items = vec![
        ListItem::new(format!(
            "Auto-pause on ear removal: {}",
            if auto_pause { "ON" } else { "OFF" }
        ))
        .style(if state.selected == 0 {
            Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings (Enter to toggle)"));
    frame.render_widget(list, chunks[0]);

    // About section
    let about = Paragraph::new(vec![
        Line::from(Span::styled(
            "About",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("FreeBuds Manager v0.1.0"),
        Line::from("Manages Huawei FreeBuds headphones via SPP protocol"),
        Line::from("Based on the OpenFreebuds project"),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(about, chunks[1]);
}

pub fn on_enter(config: &HashMap<String, String>, state: &PageState) -> Action {
    on_cycle(config, state, 0)
}

pub fn on_cycle(config: &HashMap<String, String>, state: &PageState, _direction: i32) -> Action {
    if state.selected == 0 {
        let auto_pause = config.get("auto_pause").map(|s| s == "true").unwrap_or(false);
        Action::SetProperty {
            group: "tws_auto_pause".into(),
            prop: "auto_pause".into(),
            value: if auto_pause { "false" } else { "true" }.into(),
        }
    } else {
        Action::None
    }
}
