mod app;
mod components;
mod message;
mod pages;
mod types;
mod module_view;
mod plugin;
mod plugins;
mod plugin_registry;
mod modal_window;
mod utility;

use app::Dashboard;


pub fn main() -> iced::Result {
    let initial_size = (1500.0, 900.0);
    
    iced::application(Dashboard::new, Dashboard::update, Dashboard::view)
        .theme(Dashboard::theme)
        .subscription(Dashboard::subscription)
        .font(include_bytes!("fonts/icons7.otf").as_slice())
        .window_size(initial_size)
        .run()
}
