use gtk4::prelude::*;

fn battery_css_class(percent: u8) -> &'static str {
    if percent > 60 {
        "battery-green"
    } else if percent > 30 {
        "battery-amber"
    } else if percent > 10 {
        "battery-orange"
    } else {
        "battery-red"
    }
}

fn battery_card(label: &str, emoji: &str, percent: u8) -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    card.add_css_class("battery-card");
    card.set_halign(gtk4::Align::Fill);
    card.set_hexpand(true);

    let label_widget = gtk4::Label::builder()
        .label(&format!("{} {}", emoji, label))
        .css_classes(["dim-label", "caption"])
        .build();

    let pct_label = gtk4::Label::builder()
        .label(&format!("{}%", percent))
        .css_classes(["title-1"])
        .build();

    let bar = gtk4::ProgressBar::new();
    bar.set_fraction(percent as f64 / 100.0);
    bar.add_css_class(battery_css_class(percent));

    card.append(&label_widget);
    card.append(&pct_label);
    card.append(&bar);

    card
}

pub fn battery_display(
    left: Option<u8>,
    right: Option<u8>,
    case: Option<u8>,
    global: Option<u8>,
    is_charging: bool,
) -> gtk4::Box {
    let wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 6);

    let cards_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    cards_row.set_homogeneous(true);

    if let (Some(l), Some(r)) = (left, right) {
        cards_row.append(&battery_card("Left", "L", l));
        cards_row.append(&battery_card("Right", "R", r));
        if let Some(c) = case {
            if c > 0 {
                cards_row.append(&battery_card("Case", "C", c));
            }
        }
    } else if let Some(g) = global {
        cards_row.append(&battery_card("Battery", "~", g));
    }

    wrapper.append(&cards_row);

    if is_charging {
        let charging = gtk4::Label::builder()
            .label("Charging...")
            .css_classes(["dim-label"])
            .halign(gtk4::Align::Center)
            .build();
        wrapper.append(&charging);
    }

    wrapper
}
