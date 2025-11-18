use crate::app::Dashboard;
use crate::message::Message;
use iced::{
    Color, Element,
    widget::{button, column, row, text, vertical_space},
};

pub fn view(dashboard: &Dashboard) -> Element<Message> {
    let page_title = text("Overview").size(28).color(if dashboard.dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    });

    let status = text("Status: System Running | Uptime: 99.9% | Last Updated: Now")
        .size(14)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    column![
        page_title,
        vertical_space().height(20),
        vertical_space().height(20),
        vertical_space().height(20),
        status,
    ]
    .into()
}
