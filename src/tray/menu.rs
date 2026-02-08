use std::collections::HashMap;
use std::sync::atomic::Ordering;

use ksni::menu::*;

/// Build the tray context menu from device state.
pub fn build_menu(
    device_name: Option<&str>,
    battery: &HashMap<String, String>,
    anc_mode: Option<&str>,
    anc_options: &[&str],
    connected: bool,
) -> Vec<MenuItem<super::MyBudsTray>> {
    let mut items: Vec<MenuItem<super::MyBudsTray>> = Vec::new();

    // Device name header
    if let Some(name) = device_name {
        items.push(
            StandardItem {
                label: name.to_string(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
    }

    if connected {
        // Battery info
        let mut battery_parts = Vec::new();
        if let Some(left) = battery.get("left") {
            battery_parts.push(format!("L: {}%", left));
        }
        if let Some(right) = battery.get("right") {
            battery_parts.push(format!("R: {}%", right));
        }
        if let Some(case) = battery.get("case") {
            if case != "0" {
                battery_parts.push(format!("Case: {}%", case));
            }
        }
        if battery_parts.is_empty() {
            if let Some(global) = battery.get("global") {
                battery_parts.push(format!("Battery: {}%", global));
            }
        }
        if !battery_parts.is_empty() {
            items.push(
                StandardItem {
                    label: battery_parts.join("  "),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);

        // ANC controls as RadioGroup
        if !anc_options.is_empty() {
            let selected_idx = anc_options
                .iter()
                .position(|&opt| Some(opt) == anc_mode)
                .unwrap_or(0);

            let options: Vec<RadioItem> = anc_options
                .iter()
                .map(|&opt| {
                    let label = match opt {
                        "normal" => "Off",
                        "cancellation" => "Noise Cancelling",
                        "awareness" => "Awareness",
                        _ => opt,
                    };
                    RadioItem {
                        label: label.to_string(),
                        enabled: true,
                        ..Default::default()
                    }
                })
                .collect();

            let anc_opts: Vec<String> = anc_options.iter().map(|s| s.to_string()).collect();
            items.push(
                RadioGroup {
                    selected: selected_idx,
                    select: Box::new(move |tray: &mut super::MyBudsTray, idx| {
                        if let Some(mode) = anc_opts.get(idx) {
                            tray.pending_anc_mode = Some(mode.clone());
                        }
                    }),
                    options,
                }
                .into(),
            );
            items.push(MenuItem::Separator);
        }
    } else {
        items.push(
            StandardItem {
                label: "Not connected".to_string(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
        items.push(MenuItem::Separator);
    }

    // Show window
    items.push(
        StandardItem {
            label: "Show Window".to_string(),
            activate: Box::new(|tray: &mut super::MyBudsTray| {
                tray.flags.show_window.store(true, Ordering::Relaxed);
            }),
            ..Default::default()
        }
        .into(),
    );

    items.push(MenuItem::Separator);

    // Quit
    items.push(
        StandardItem {
            label: "Quit".to_string(),
            activate: Box::new(|tray: &mut super::MyBudsTray| {
                tray.flags.quit_app.store(true, Ordering::Relaxed);
            }),
            ..Default::default()
        }
        .into(),
    );

    items
}
