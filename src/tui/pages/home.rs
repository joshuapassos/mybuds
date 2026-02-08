use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::tui::{Action, PageState};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    battery: &HashMap<String, String>,
    anc: &HashMap<String, String>,
    info: &HashMap<String, String>,
    connected: bool,
    state: &mut PageState,
) {
    if !connected {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "MyBuds",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "No device connected",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Pair your FreeBuds via Bluetooth settings.",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Home"));
        frame.render_widget(msg, area);
        state.item_count = 0;
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // device header
            Constraint::Length(8),  // battery section
            Constraint::Min(4),    // ANC section
        ])
        .split(area);

    // Device header
    let device_model = info
        .get("device_model")
        .or_else(|| info.get("field_15"))
        .map(|s| s.as_str())
        .unwrap_or("FreeBuds");
    let sw_ver = info.get("software_ver").map(|s| s.as_str()).unwrap_or("");
    let header_text = if sw_ver.is_empty() {
        device_model.to_string()
    } else {
        format!("{} ({})", device_model, sw_ver)
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Battery section
    render_battery(frame, chunks[1], battery);

    // ANC section
    render_anc(frame, chunks[2], anc, state);
}

fn render_battery(frame: &mut Frame, area: Rect, battery: &HashMap<String, String>) {
    let is_charging = battery.get("is_charging").map_or(false, |s| s == "true");
    let title = if is_charging {
        "Battery [Charging]"
    } else {
        "Battery"
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut gauges: Vec<(&str, Option<u16>)> = Vec::new();

    let left = battery.get("left").and_then(|s| s.parse().ok());
    let right = battery.get("right").and_then(|s| s.parse().ok());
    let case_val = battery.get("case").and_then(|s| s.parse().ok());
    let global = battery.get("global").and_then(|s| s.parse().ok());

    if left.is_some() || right.is_some() {
        gauges.push(("Left ", left));
        gauges.push(("Right", right));
        if case_val.is_some() {
            gauges.push(("Case ", case_val));
        }
    } else if global.is_some() {
        gauges.push(("Level", global));
    }

    if gauges.is_empty() {
        let p = Paragraph::new("No battery data");
        frame.render_widget(p, inner);
        return;
    }

    let constraints: Vec<Constraint> = gauges.iter().map(|_| Constraint::Length(1)).collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(0)
        .split(inner);

    for (i, (label, value)) in gauges.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let pct = value.unwrap_or(0);
        let color = if pct <= 15 {
            Color::Red
        } else if pct <= 30 {
            Color::Yellow
        } else {
            Color::Green
        };

        let gauge = Gauge::default()
            .label(format!("{}: {}%", label, pct))
            .ratio(pct as f64 / 100.0)
            .gauge_style(Style::default().fg(color));
        frame.render_widget(gauge, rows[i]);
    }
}

fn render_anc(
    frame: &mut Frame,
    area: Rect,
    anc: &HashMap<String, String>,
    state: &mut PageState,
) {
    let mode_options: Vec<String> = anc
        .get("mode_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    let current_mode = anc.get("mode").cloned();
    let level_options: Vec<String> = anc
        .get("level_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    let current_level = anc.get("level").cloned();

    let mut items: Vec<ListItem> = Vec::new();

    for opt in &mode_options {
        let is_selected = current_mode.as_ref() == Some(opt);
        let marker = if is_selected { ">" } else { " " };
        let display = anc_display_name(opt);
        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(format!("{} {}", marker, display)).style(style));
    }

    for opt in &level_options {
        let is_selected = current_level.as_ref() == Some(opt);
        let marker = if is_selected { ">" } else { " " };
        let display = format!("Level: {}", anc_display_name(opt));
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(format!("{} {}", marker, display)).style(style));
    }

    state.item_count = items.len();
    state.clamp();

    // Highlight the currently focused item with bg
    if !items.is_empty() && state.selected < items.len() {
        let item = items.remove(state.selected);
        let highlighted = item.bg(Color::DarkGray);
        items.insert(state.selected, highlighted);
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("ANC Mode (h/l to cycle)"));
    frame.render_widget(list, area);
}

fn anc_display_name(name: &str) -> String {
    match name {
        "normal" | "off" => "Off".into(),
        "cancellation" | "noise_cancelling" => "Noise Cancellation".into(),
        "awareness" | "transparency" => "Awareness".into(),
        "dynamic" => "Dynamic".into(),
        "ultra" => "Ultra".into(),
        "general" => "General".into(),
        "cozy" | "comfort" => "Cozy".into(),
        other => other.replace('_', " "),
    }
}

pub fn on_enter(anc: &HashMap<String, String>, state: &PageState) -> Action {
    on_cycle(anc, state, 0)
}

pub fn on_cycle(anc: &HashMap<String, String>, state: &PageState, direction: i32) -> Action {
    let mode_options: Vec<String> = anc
        .get("mode_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    let level_options: Vec<String> = anc
        .get("level_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if state.selected < mode_options.len() {
        // Cycling ANC mode
        let current = anc.get("mode");
        let new_val = cycle_option(&mode_options, current.map(|s| s.as_str()), direction);
        if let Some(val) = new_val {
            return Action::SetProperty {
                group: "anc".into(),
                prop: "mode".into(),
                value: val,
            };
        }
    } else {
        let level_idx = state.selected - mode_options.len();
        if level_idx < level_options.len() {
            let current = anc.get("level");
            let new_val = cycle_option(&level_options, current.map(|s| s.as_str()), direction);
            if let Some(val) = new_val {
                return Action::SetProperty {
                    group: "anc".into(),
                    prop: "level".into(),
                    value: val,
                };
            }
        }
    }
    Action::None
}

fn cycle_option(options: &[String], current: Option<&str>, direction: i32) -> Option<String> {
    if options.is_empty() {
        return None;
    }
    let cur_idx = current
        .and_then(|c| options.iter().position(|o| o == c))
        .unwrap_or(0);
    let new_idx = if direction == 0 {
        // Enter just selects next
        (cur_idx + 1) % options.len()
    } else if direction > 0 {
        (cur_idx + 1) % options.len()
    } else {
        if cur_idx == 0 {
            options.len() - 1
        } else {
            cur_idx - 1
        }
    };
    Some(options[new_idx].clone())
}
