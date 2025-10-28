use crate::message::Message;
use iced::{
    Alignment, Color, Element, Theme,
    widget::{column, container, text},
};

pub fn view<'a>(label: &'a str, value: i32, color: Color, prefix: &'a str) -> Element<'a, Message> {
    let value_text = if prefix.is_empty() {
        format!("{}", value)
    } else {
        format!("{}{}", prefix, value)
    };

    let card_content = column![
        text(label).size(16).color(Color::WHITE),
        text(value_text).size(28).color(Color::WHITE),
    ]
    .spacing(10)
    .padding(20)
    .align_x(Alignment::Center);

    container(card_content)
        .width(200)
        .height(120)
        .style(move |_theme: &Theme| container::Style {
            background: Some(color.into()),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
