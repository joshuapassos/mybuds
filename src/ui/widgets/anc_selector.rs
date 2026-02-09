use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

fn anc_mode_label(mode: &str) -> String {
    match mode {
        "normal" => "Off".into(),
        "cancellation" => "Noise Cancelling".into(),
        "awareness" => "Awareness".into(),
        other => other.to_string(),
    }
}

fn anc_level_label(level: &str) -> String {
    match level {
        "comfort" => "Comfort".into(),
        "normal" => "Normal".into(),
        "ultra" => "Ultra".into(),
        "dynamic" => "Dynamic".into(),
        "voice_boost" => "Voice Boost".into(),
        other => other.to_string(),
    }
}

/// Render ANC mode as styled toggle buttons.
pub fn anc_mode_selector<'a, M: Clone + 'a>(
    current_mode: Option<&str>,
    options: &[String],
    on_change: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    let section_label = text("Noise Control".to_string())
        .size(16)
        .color(iced::Color::from_rgb(0.3, 0.3, 0.3));

    let mut buttons: Vec<Element<'a, M>> = Vec::new();

    for opt in options.iter() {
        let label = anc_mode_label(opt);
        let is_active = current_mode == Some(opt.as_str());
        let opt_clone = opt.clone();

        let style = if is_active {
            button::primary
        } else {
            button::secondary
        };

        let btn = button(
            container(text(label).size(13))
                .center_x(Length::Fill)
                .padding(4),
        )
        .on_press(on_change(opt_clone))
        .style(style)
        .width(Length::Fill);

        buttons.push(btn.into());
    }

    let button_row = row(buttons).spacing(6);

    column![section_label, button_row]
        .spacing(8)
        .into()
}

/// Render ANC level as styled toggle buttons.
pub fn anc_level_selector<'a, M: Clone + 'a>(
    current_level: Option<&str>,
    options: &[String],
    on_change: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    let section_label = text("Level".to_string())
        .size(14)
        .color(iced::Color::from_rgb(0.4, 0.4, 0.4));

    let mut buttons: Vec<Element<'a, M>> = Vec::new();

    for opt in options.iter() {
        let label = anc_level_label(opt);
        let is_active = current_level == Some(opt.as_str());
        let opt_clone = opt.clone();

        let style = if is_active {
            button::primary
        } else {
            button::secondary
        };

        let btn = button(
            container(text(label).size(12))
                .center_x(Length::Fill)
                .padding(2),
        )
        .on_press(on_change(opt_clone))
        .style(style)
        .width(Length::Fill);

        buttons.push(btn.into());
    }

    let button_row = row(buttons).spacing(4);

    column![section_label, button_row]
        .spacing(6)
        .into()
}
