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

impl ConfirmModal {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn update(&mut self, message: ConfirmMessage) -> Option<bool> {
        match message {
            ConfirmMessage::Confirm => Some(true),
            ConfirmMessage::Cancel => Some(false),
        }
    }
}

impl ModalWindowView for ConfirmModal {
    fn title(&self) -> String {
        "Confirm".to_string()
    }

    fn get_config(&self) -> ModalConfig {
        ModalConfig {
            width: 400.0,
            height: 200.0,
            show_refresh: false,
            show_apply: true,
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

    fn main_content(&self, dark_mode: bool) -> Element<'_, Message> {
        self.draw(dark_mode)
    }

    fn close_message(&self) -> Message {
        Message::Confirm(ConfirmMessage::Cancel)
    }

    fn refresh_message(&self) -> Option<Message> {
        None
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::Confirm(ConfirmMessage::Confirm))
    }
}