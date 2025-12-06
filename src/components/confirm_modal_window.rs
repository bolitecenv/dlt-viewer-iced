use std::str::FromStr;

use crate::components::modal_window::{ ModalWindowView, ModalConfig };
use crate::message::Message;
use iced::widget::{center, mouse_area, opaque};
use iced::{
    Color, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, text, Space
    },
    Font,
};

// Confirmation dialog modal
#[derive(Debug, Clone)]
pub enum ConfirmMessage {
    Confirm,
    Cancel,
}

pub struct ConfirmModal {
    message: String,
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
        Self { message }
    }

    pub fn handle_update(&self, message: ConfirmMessage) -> Option<Message> {
        match message {
            ConfirmMessage::Confirm => {
                println!("Confirmed action.");
                None
            }
            ConfirmMessage::Cancel => {
                Some(Message::CloseSettingsModal)
            }
        }
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

    fn content(&self) -> Element<'_, Message> {
        column![
            text(&self.message).size(16),
        ]
        .padding(20)
        .into()
    }

    fn close_message(&self) -> Message {
        Message::MessageModalWindow("cancel".to_string())
    }

    fn refresh_message(&self) -> Option<Message> {
        None
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::MessageModalWindow("confirm".to_string()))
    }

    fn update(&self, message: String) -> Option<Message> {
        self.handle_update(message.parse().ok()?)
    }
}