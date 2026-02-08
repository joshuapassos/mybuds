use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    info: &HashMap<String, String>,
) {
    let known_fields = [
        ("device_model", "Model"),
        ("device_submodel", "Submodel"),
        ("hardware_ver", "Hardware Version"),
        ("software_ver", "Firmware Version"),
        ("serial_number", "Serial Number"),
        ("left_serial_number", "Left S/N"),
        ("right_serial_number", "Right S/N"),
    ];

    let mut rows: Vec<Row> = Vec::new();

    for (key, label) in &known_fields {
        if let Some(value) = info.get(*key) {
            rows.push(Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::DarkGray)),
                Cell::from(value.as_str()),
            ]));
        }
    }

    // Extra unknown fields
    let mut extra: Vec<(&String, &String)> = info
        .iter()
        .filter(|(k, _)| !known_fields.iter().any(|(kf, _)| *kf == k.as_str()))
        .collect();
    extra.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (key, value) in extra {
        rows.push(Row::new(vec![
            Cell::from(key.as_str()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(value.as_str()),
        ]));
    }

    if rows.is_empty() {
        rows.push(Row::new(vec![Cell::from("No device info available")]));
    }

    let widths = [Constraint::Length(20), Constraint::Min(10)];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::ALL).title("Device Info"))
        .column_spacing(2);
    frame.render_widget(table, area);
}
