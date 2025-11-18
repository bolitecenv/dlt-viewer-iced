// plugins/example.rs
use crate::plugin::{DashboardContext, Plugin, PluginMessage};
use iced::{widget::{column, text, button}, Element, Task};

pub struct ExamplePlugin {
    counter: i32,
}

#[derive(Debug, Clone)]
pub enum ExampleMessage {
    IncrementPressed,
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "Example"
    }
    
    fn nav_name(&self) -> &str {
        "EX Plugin"
    }
    
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn update(&mut self, message: PluginMessage, context: &DashboardContext) -> Task<PluginMessage> {
        if let PluginMessage::Custom(name, _) = message {
            if name == "increment" {
                self.counter += 1;
                println!("context ecu_list length: {}", context.ecu_list.len());
                println!("context dlt_buffer length: {}", context.dlt_buffer.len());
            }
        }
        Task::none()
    }

    fn view(&self, context: &DashboardContext) -> Element<PluginMessage> {
        column![
            text("Example Plugin").size(24),
            text(format!("Counter: {}", self.counter)),
            button("Count Up")
                .on_press(PluginMessage::Custom("increment".to_string(), vec![])),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}