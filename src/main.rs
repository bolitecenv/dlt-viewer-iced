mod app;
mod components;
mod message;
mod pages;
mod types;
mod module_view;

use app::Dashboard;

pub fn main() -> iced::Result {
    iced::application("Dashboard App", Dashboard::update, Dashboard::view)
        .theme(Dashboard::theme)
        .subscription(Dashboard::subscription)
        .font(include_bytes!("fonts/icons7.otf").as_slice())
        .run()
}
