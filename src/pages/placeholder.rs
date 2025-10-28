use crate::message::Message;
use iced::{
    Alignment, Color, Element,
    widget::{column, text, vertical_space},
};

pub fn view<'a>(title: &'a str, icon: &'a str, dark_mode: bool) -> Element<'a, Message> {
    let page_title = text(title).size(28).color(if dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    });

    let icon_text = text(icon).size(64);

    let desc = text(format!("{} page - Coming soon!", title))
        .size(16)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    column![
        page_title,
        vertical_space().height(60),
        icon_text,
        vertical_space().height(20),
        desc,
    ]
    .align_x(Alignment::Center)
    .into()
}
