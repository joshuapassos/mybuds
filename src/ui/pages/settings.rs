use std::collections::HashMap;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use relm4::ComponentSender;

use crate::ui::{Message, MyBudsApp};

pub fn build(
    container: &gtk4::Box,
    config: &HashMap<String, String>,
    sender: &ComponentSender<MyBudsApp>,
) {
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

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

    // Auto-pause
    let auto_pause = config
        .get("auto_pause")
        .map(|s| s == "true")
        .unwrap_or(false);

    let group = adw::PreferencesGroup::new();
    let row = adw::ActionRow::builder()
        .title("Auto-pause on ear removal")
        .build();
    let switch = gtk4::Switch::builder()
        .active(auto_pause)
        .valign(gtk4::Align::Center)
        .build();
    let s = sender.clone();
    switch.connect_state_set(move |_, active| {
        s.input(Message::SetAutoPause(active));
        gtk4::glib::Propagation::Proceed
    });
    row.add_suffix(&switch);
    row.set_activatable_widget(Some(&switch));
    group.add(&row);
    content.append(&group.upcast::<gtk4::Widget>());

    // About
    let about_group = adw::PreferencesGroup::builder()
        .title("About")
        .build();
    let about_row = adw::ActionRow::builder()
        .title("MyBuds")
        .subtitle("v0.1.0 — Manages Huawei FreeBuds and Apple AirPods headphones.\nBased on the OpenFreebuds project.")
        .build();
    about_group.add(&about_row);
    content.append(&about_group.upcast::<gtk4::Widget>());

    clamp.set_child(Some(&content));
    container.append(&clamp);
}
