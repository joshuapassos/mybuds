use gtk4::prelude::*;
use relm4::ComponentSender;

use crate::ui::{Message, MyBudsApp};

fn anc_mode_label(mode: &str) -> &str {
    match mode {
        "normal" => "Off",
        "cancellation" => "Noise Cancelling",
        "awareness" => "Awareness",
        _ => mode,
    }
}

fn anc_level_label(level: &str) -> &str {
    match level {
        "comfort" => "Comfort",
        "normal" => "Normal",
        "ultra" => "Ultra",
        "dynamic" => "Dynamic",
        "voice_boost" => "Voice Boost",
        _ => level,
    }
}

pub fn anc_mode_buttons(
    current_mode: Option<&str>,
    options: &[String],
    sender: &ComponentSender<MyBudsApp>,
) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    row.add_css_class("linked");
    row.set_halign(gtk4::Align::Center);
    row.set_margin_top(8);
    row.set_margin_bottom(8);

    let mut first_btn: Option<gtk4::ToggleButton> = None;

    for opt in options {
        let label = anc_mode_label(opt);
        let is_active = current_mode == Some(opt.as_str());

        let btn = gtk4::ToggleButton::builder()
            .label(label)
            .active(is_active)
            .build();

        if let Some(ref group) = first_btn {
            btn.set_group(Some(group));
        } else {
            first_btn = Some(btn.clone());
        }

        let s = sender.clone();
        let opt_clone = opt.clone();
        btn.connect_toggled(move |b| {
            if b.is_active() {
                s.input(Message::SetAncMode(opt_clone.clone()));
            }
        });

        row.append(&btn);
    }

    row
}

pub fn anc_level_buttons(
    current_level: Option<&str>,
    options: &[String],
    sender: &ComponentSender<MyBudsApp>,
) -> gtk4::Box {
    let wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    let label = gtk4::Label::builder()
        .label("Level")
        .css_classes(["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    wrapper.append(&label);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    row.add_css_class("linked");
    row.set_halign(gtk4::Align::Center);

    let mut first_btn: Option<gtk4::ToggleButton> = None;

    for opt in options {
        let label = anc_level_label(opt);
        let is_active = current_level == Some(opt.as_str());

        let btn = gtk4::ToggleButton::builder()
            .label(label)
            .active(is_active)
            .build();

        if let Some(ref group) = first_btn {
            btn.set_group(Some(group));
        } else {
            first_btn = Some(btn.clone());
        }

        let s = sender.clone();
        let opt_clone = opt.clone();
        btn.connect_toggled(move |b| {
            if b.is_active() {
                s.input(Message::SetAncLevel(opt_clone.clone()));
            }
        });

        row.append(&btn);
    }

    wrapper.append(&row);
    wrapper
}
