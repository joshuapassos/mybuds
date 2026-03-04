use std::collections::HashMap;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use relm4::ComponentSender;

use crate::ui::{Message, MyBudsApp};

pub fn build(
    container: &gtk4::Box,
    sound: &HashMap<String, String>,
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

    // EQ Presets
    let eq_options: Vec<String> = sound
        .get("equalizer_preset_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if !eq_options.is_empty() {
        let current_eq = sound.get("equalizer_preset").cloned();
        let group = adw::PreferencesGroup::builder()
            .title("Equalizer")
            .build();

        let combo = adw::ComboRow::builder()
            .title("Preset")
            .build();

        let labels: Vec<String> = eq_options.iter().map(|s| eq_display_name(s)).collect();
        let string_list = gtk4::StringList::new(&labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        combo.set_model(Some(&string_list));

        // Set active item
        if let Some(ref current) = current_eq {
            if let Some(pos) = eq_options.iter().position(|o| o == current) {
                combo.set_selected(pos as u32);
            }
        }

        let s = sender.clone();
        let opts = eq_options.clone();
        combo.connect_selected_notify(move |c| {
            let idx = c.selected() as usize;
            if idx < opts.len() {
                s.input(Message::SetEqPreset(opts[idx].clone()));
            }
        });

        group.add(&combo);
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Sound Quality
    let quality_options: Vec<String> = sound
        .get("quality_preference_options")
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    if !quality_options.is_empty() {
        let current_quality = sound.get("quality_preference").cloned();
        let group = adw::PreferencesGroup::builder()
            .title("Sound Quality")
            .build();

        let combo = adw::ComboRow::builder()
            .title("Preference")
            .build();

        let labels: Vec<String> = quality_options
            .iter()
            .map(|s| match s.as_str() {
                "sqp_connectivity" => "Connectivity Priority".to_string(),
                "sqp_quality" => "Sound Quality Priority".to_string(),
                other => other.to_string(),
            })
            .collect();
        let string_list = gtk4::StringList::new(&labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        combo.set_model(Some(&string_list));

        if let Some(ref current) = current_quality {
            if let Some(pos) = quality_options.iter().position(|o| o == current) {
                combo.set_selected(pos as u32);
            }
        }

        let s = sender.clone();
        let opts = quality_options.clone();
        combo.connect_selected_notify(move |c| {
            let idx = c.selected() as usize;
            if idx < opts.len() {
                s.input(Message::SetSoundQuality(opts[idx].clone()));
            }
        });

        group.add(&combo);
        content.append(&group.upcast::<gtk4::Widget>());
    }

    // Low Latency
    {
        let low_latency = config
            .get("low_latency")
            .map(|s| s == "true")
            .unwrap_or(false);

        let group = adw::PreferencesGroup::new();
        let row = adw::ActionRow::builder()
            .title("Low Latency Mode")
            .build();
        let switch = gtk4::Switch::builder()
            .active(low_latency)
            .valign(gtk4::Align::Center)
            .build();
        let s = sender.clone();
        switch.connect_state_set(move |_, active| {
            s.input(Message::SetLowLatency(active));
            gtk4::glib::Propagation::Proceed
        });
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));
        group.add(&row);
        content.append(&group.upcast::<gtk4::Widget>());
    }

    clamp.set_child(Some(&content));
    container.append(&clamp);
}

fn eq_display_name(key: &str) -> String {
    match key {
        "equalizer_preset_default" => "Default".to_string(),
        "equalizer_preset_hardbass" => "Bass Boost".to_string(),
        "equalizer_preset_treble" => "Treble Boost".to_string(),
        "equalizer_preset_voices" | "equalizer_preset_voice" => "Voice".to_string(),
        other => other.replace("equalizer_preset_", "").replace('_', " "),
    }
}
