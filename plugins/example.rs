// plugins/example.rs
use crate::{plugin::{DashboardContext, Plugin, PluginMessage}, utility::util::deserialize_message};
use bincode::{Decode, Encode, config, encode_to_vec};
use iced::{widget::{column, text, button}, Element, Task};

pub struct ExamplePlugin {
    counter: i32,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum ExampleMessage {
    IncrementPressed,
}

impl ExampleMessage {
    fn create_custom_message(msg: ExampleMessage) -> PluginMessage {
        let data = encode_to_vec(&msg, config::standard()).unwrap();
        PluginMessage::Custom("example_plugin".to_string(), data)
    }
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

    fn update(&mut self, message: PluginMessage, _context: &DashboardContext) -> Task<PluginMessage> {
        match message {
            PluginMessage::Custom(_name, data) => {
                if let Ok(msg) = deserialize_message::<ExampleMessage>(&data) {
                    match msg {
                        ExampleMessage::IncrementPressed => {
                            self.counter += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self, _context: &DashboardContext) -> Element<'_, PluginMessage> {
        column![
            text("Example Plugin").size(24),
            text(format!("Counter: {}", self.counter)),
            button("Count Up")
                .on_press(ExampleMessage::create_custom_message(ExampleMessage::IncrementPressed)),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}