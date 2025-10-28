use crate::chart::line_chart;
use crate::message::Message;
use iced::{
    Color, Element,
    widget::{column, text, vertical_space},
};

pub fn view(dark_mode: bool, chart_data: &[f32]) -> Element<'static, Message> {
    let page_title = text("Analytics").size(28).color(if dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    });

    // Use the passed chart data instead of generating new random data
    let chart = line_chart::view(chart_data.to_vec());

    column![
        page_title,
        vertical_space().height(20),
        text("Sales Performance").size(18).color(if dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.2, 0.2, 0.2)
        }),
        vertical_space().height(10),
        chart,
    ]
    .into()
}
