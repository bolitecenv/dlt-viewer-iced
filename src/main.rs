mod app;
mod chart;
mod components;
mod message;
mod pages;

use app::Dashboard;

pub fn main() -> iced::Result {
    iced::application("Dashboard App", Dashboard::update, Dashboard::view)
        .theme(Dashboard::theme)
        .subscription(Dashboard::subscription)
        .font(include_bytes!("fonts/icons.otf").as_slice())
        .run()
}
