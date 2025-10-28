use crate::message::Message;
use iced::{
    Alignment, Color, Element, Length, Theme,
    widget::{button, container, horizontal_space, row, text},
};

pub fn view(dark_mode: bool) -> Element<'static, Message> {
    let theme_color = if dark_mode {
        Color::from_rgb(0.1, 0.1, 0.1)
    } else {
        Color::from_rgb(0.95, 0.95, 0.95)
    };

    let theme_button = button(text(if dark_mode { "☀ Light" } else { "🌙 Dark" }).size(14))
        .on_press(Message::ToggleTheme)
        .padding(8);

    container(
        row![
            text("Dashboard Application").size(24).color(if dark_mode {
                Color::WHITE
            } else {
                Color::BLACK
            }),
            horizontal_space(),
            theme_button,
        ]
        .spacing(20)
        .padding(15)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(move |_theme: &Theme| container::Style {
        background: Some(theme_color.into()),
        border: iced::Border {
            width: 1.0,
            color: if dark_mode {
                Color::from_rgb(0.2, 0.2, 0.2)
            } else {
                Color::from_rgb(0.85, 0.85, 0.85)
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
