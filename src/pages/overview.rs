use crate::app::Dashboard;
use crate::components::card;
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

    let cards_row = row![
        card::view(
            "Revenue",
            dashboard.metric1,
            Color::from_rgb(0.2, 0.6, 0.9),
            "$"
        ),
        card::view(
            "Orders",
            dashboard.metric2,
            Color::from_rgb(0.3, 0.7, 0.5),
            ""
        ),
        card::view(
            "Total Users",
            dashboard.total_users as i32,
            Color::from_rgb(0.8, 0.4, 0.2),
            ""
        ),
        card::view(
            "Active Sessions",
            dashboard.active_sessions as i32,
            Color::from_rgb(0.6, 0.3, 0.7),
            ""
        ),
    ]
    .spacing(20)
    .padding([20, 0]);

    let controls = row![
        button(text("+ Revenue").center())
            .on_press(Message::IncrementMetric1)
            .padding(10),
        button(text("- Revenue").center())
            .on_press(Message::DecrementMetric1)
            .padding(10),
        button(text("+ Orders").center())
            .on_press(Message::IncrementMetric2)
            .padding(10),
        button(text("Refresh Data").center())
            .on_press(Message::RefreshData)
            .padding(10),
    ]
    .spacing(10)
    .padding([20, 0]);

    let status = text("Status: System Running | Uptime: 99.9% | Last Updated: Now")
        .size(14)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    column![
        page_title,
        vertical_space().height(20),
        cards_row,
        vertical_space().height(20),
        controls,
        vertical_space().height(20),
        status,
    ]
    .into()
}
