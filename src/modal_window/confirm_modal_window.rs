use std::str::FromStr;

use crate::modal_window::modal_window::{ModalConfig, ModalWindowMessage, ModalWindowView, deserialize_message, serialize_message};
use crate::message::Message;
use iced::Task;
use iced::widget::{center, mouse_area, opaque, text_input};
use iced::{
    Color, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, text, Space
    },
    Font,
};
use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};

// Confirmation dialog modal
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ConfirmMessage {
    Confirm,
    Cancel,
    Save,
    Setting(String),
}

pub struct ConfirmModal {
    message: String,
    trace_status_input: String,
    title_input: String,
    should_close: bool,
}

impl FromStr for ConfirmMessage {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "confirm" => Ok(ConfirmMessage::Confirm),
            "cancel" => Ok(ConfirmMessage::Cancel),
            _ => Err(()),
        }
    }
}

impl ConfirmModal {
    pub fn new(message: String) -> Self {
        Self { message, 
            trace_status_input: String::new(),
            title_input: String::new(),
            should_close: false
        }
    }

    fn create_custom_message(msg: ConfirmMessage) -> ModalWindowMessage {
        let data = serialize_message(&msg).unwrap();
        ModalWindowMessage::Custom("confirm_modal".to_string(), data)
    }
}

impl ModalWindowView for ConfirmModal {
    fn title(&self) -> String {
        "Confirm".to_string()
    }

    fn get_config(&self) -> ModalConfig {
        ModalConfig {
            width: 800.0,
            height: 600.0,
            show_refresh: false,
            show_apply: true,
            can_apply: true,
            can_close: true,
            title: self.title(),
            ..Default::default()
        }
    }

    fn content(&self) -> Element<'_, ModalWindowMessage> {
        let label_color = Color::from_rgb(0.7, 0.7, 0.7);
        
        let content = column![
            row![
                text("Trace Status:")
                    .size(14)
                    .color(label_color)
                    .width(Length::Fixed(120.0)),
                text_input("0 or 1", &self.trace_status_input)
                    .on_input(|v| {
                        Self::create_custom_message(ConfirmMessage::Setting(v))
                    })
                    .width(Length::Fill),
            ]
            .spacing(10),
        ]
        .spacing(15)
        .padding(20);
        
        container(content)
            .width(Length::Fixed(400.0))
            .into()
    }

    fn update(&mut self, message: ModalWindowMessage) -> Task<Message> {
        match message {
            ModalWindowMessage::Apply => {
                println!("Confirmed");
            }
            ModalWindowMessage::Close => {
                println!("Cancelled");
            }
            ModalWindowMessage::Refresh => {
                println!("Refreshed");
            }
            ModalWindowMessage::Custom(msg_type, data) => {
                if let Ok(msg) = deserialize_message::<ConfirmMessage>(&data) {
                    match msg {
                        ConfirmMessage::Setting(value) => {
                            println!("Setting trace status to {}", value);
                        }
                        ConfirmMessage::Confirm => {
                            println!("Confirmed via custom message");
                        }
                        ConfirmMessage::Cancel => {
                            println!("Cancelled via custom message");
                        }
                        _ => {}
                    }
                }
            }
            _ => {},
        }
        Task::none()
    }
}