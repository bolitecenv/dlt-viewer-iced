use std::any::Any;

use crate::app::ICON_FONT;
use crate::modal_window::modal_window::{ModalConfig, ModalWindowMessage, ModalWindowView, deserialize_message, serialize_message};
use crate::message::Message;
use crate::module_view::ModuleWidget;
use iced::Task;
use iced::widget::text_input;
use iced::{
    Color, Element, Length,
    widget::{
        column, container, row, text
    },
};
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode, encode_to_vec, decode_from_slice, config};
use bincode::error::{EncodeError, DecodeError};

// Confirmation dialog modal
#[derive(Debug, Clone, Encode, Decode)]
pub enum ConfirmMessage {
    Confirm,
    Cancel,
}

pub struct ConfirmModal {
    title: String,
    message: String,
}

impl ConfirmModal {
    pub fn new(title: String, message: String) -> Self {
        Self { 
            title, 
            message, 
        }
    }

    fn create_custom_message(msg: ConfirmMessage) -> ModalWindowMessage {
        let data = serialize_message(&msg).unwrap();
        ModalWindowMessage::Custom("confirm_modal".to_string(), data)
    }
}

impl ModalWindowView for ConfirmModal {
    fn get_config(&self) -> ModalConfig {
        ModalConfig {
            width: 400.0,
            height: 300.0,
            show_refresh: false,
            show_apply: true,
            can_apply: true,
            can_close: true,
            title: self.title.clone(),
            ..Default::default()
        }
    }

    fn content(&self) -> Element<'_, ModalWindowMessage> {
        let label_color = Color::from_rgb(0.1, 0.1, 0.1);
        
        let content = column![
            row![
                text(&self.message)
                    .size(16)
                    .color(label_color)
            ]
        ]
        .spacing(20)
        .padding(20);
        
        container(content)
            .width(Length::Fixed(400.0))
            .into()
    }

    fn update(&mut self, message: ModalWindowMessage, module: Option<&mut ModuleWidget>) -> Task<Message> {
        match message {
            ModalWindowMessage::Apply => {
                return Task::done(Message::CloseSettingsModal);
            }
            ModalWindowMessage::Close => {
                return Task::done(Message::CloseSettingsModal);
            }
            ModalWindowMessage::Custom(msg_type, data) => {
                if let Ok(msg) = deserialize_message::<ConfirmMessage>(&data) {
                    match msg {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}