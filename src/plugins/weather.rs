// plugins/weather.rs or src/plugins/weather.rs
use crate::plugin::{DashboardContext, Plugin, PluginMessage};
use iced::{Element, Task, widget::{column, text, button}};

pub struct WeatherPlugin {
    temperature: String,
}

impl Plugin for WeatherPlugin {
    fn name(&self) -> &str {
        "Weather"
    }

    fn nav_name(&self) -> &str {
        "Weather"
    }
    
    fn new() -> Self {
        Self {
            temperature: "Loading...".to_string(),
        }
    }

    fn update(&mut self, message: PluginMessage, context: &DashboardContext) -> Task<PluginMessage> {
        // Handle messages
        Task::none()
    }

    fn view(&self, context: &DashboardContext) -> Element<PluginMessage> {
        column![
            text("Weather Dashboard").size(24),
            text(&self.temperature),
            button("Refresh"),
        ].into()
    }
    
    fn description(&self) -> Option<&str> {
        Some("Shows current weather information")
    }
}