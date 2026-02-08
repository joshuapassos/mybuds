use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length};

/// Color for battery level.
fn battery_color(percent: u8) -> iced::Color {
    if percent > 60 {
        iced::Color::from_rgb(0.18, 0.72, 0.38) // Green
    } else if percent > 30 {
        iced::Color::from_rgb(0.95, 0.68, 0.0) // Amber
    } else if percent > 10 {
        iced::Color::from_rgb(0.95, 0.45, 0.0) // Orange
    } else {
        iced::Color::from_rgb(0.90, 0.22, 0.20) // Red
    }
}

/// A single battery card with icon label, percentage, and progress bar.
fn battery_card<'a, M: 'a>(label: &str, emoji: &str, percent: u8) -> Element<'a, M> {
    let color = battery_color(percent);
    let bar_fraction = percent as f32 / 100.0;

    let percentage_text = text(format!("{}%", percent))
        .size(26);

    let label_text = text(format!("{} {}", emoji, label))
        .size(13)
        .color(iced::Color::from_rgb(0.45, 0.45, 0.45));

    // Progress bar: colored fill inside a gray track
    let bar_width = 100.0;
    let fill_width = (bar_fraction * bar_width).max(2.0);

    let bar_track = container(
        container(text(""))
            .width(Length::Fixed(fill_width))
            .height(Length::Fixed(6.0))
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
    .width(Length::Fixed(bar_width))
    .height(Length::Fixed(6.0))
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.90, 0.90, 0.90,
        ))),
        border: iced::Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Card container
    let card_content = column![label_text, percentage_text, bar_track,]
        .spacing(4)
        .align_x(Alignment::Center);

    container(card_content)
        .padding([12, 16])
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.97, 0.97, 0.97,
            ))),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgb(0.90, 0.90, 0.90),
            },
            ..Default::default()
        })
        .into()
}

/// Full battery display with cards for left, right, case (or global).
pub fn battery_display<'a, M: 'a>(
    left: Option<u8>,
    right: Option<u8>,
    case: Option<u8>,
    global: Option<u8>,
    is_charging: bool,
) -> Element<'a, M> {
    let mut cards: Vec<Element<'a, M>> = Vec::new();

    if let (Some(l), Some(r)) = (left, right) {
        cards.push(battery_card("Left", "L", l));
        cards.push(battery_card("Right", "R", r));
        if let Some(c) = case {
            if c > 0 {
                cards.push(battery_card("Case", "C", c));
            }
        }
    } else if let Some(g) = global {
        cards.push(battery_card("Battery", "~", g));
    }

    let battery_row = row(cards).spacing(10);

    let mut content = column![battery_row].spacing(6);

    if is_charging {
        content = content.push(
            container(
                text("Charging...")
                    .size(12)
                    .color(iced::Color::from_rgb(0.18, 0.72, 0.38)),
            )
            .center_x(Length::Fill),
        );
    }

    content.into()
}
