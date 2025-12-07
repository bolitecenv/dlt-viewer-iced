use std::any::Any;

use iced::{Element, Task};

use crate::{message::Message, modal_window::modal_window::{ModalConfig, ModalWindowMessage, ModalWindowView}, module_view::{ModuleWidget, module_widget::ModuleWidgetWindowView}};


#[derive(Debug, Clone)]
pub enum SettingModalMessage {
    Close,
    Refresh,
    Apply,
    Custom(String, Vec<u8>),
}

impl From<SettingModalMessage> for ModalWindowMessage {
    fn from(msg: SettingModalMessage) -> Self {
        match msg {
            SettingModalMessage::Close => ModalWindowMessage::Close,
            SettingModalMessage::Refresh => ModalWindowMessage::Refresh,
            SettingModalMessage::Apply => ModalWindowMessage::Apply,
            SettingModalMessage::Custom(typ, data) => ModalWindowMessage::Custom(typ, data),
        }
    }
}

impl From<ModalWindowMessage> for SettingModalMessage {
    fn from(msg: ModalWindowMessage) -> Self {
        match msg {
            ModalWindowMessage::Close => SettingModalMessage::Close,
            ModalWindowMessage::Refresh => SettingModalMessage::Refresh,
            ModalWindowMessage::Apply => SettingModalMessage::Apply,
            ModalWindowMessage::Custom(typ, data) => SettingModalMessage::Custom(typ, data),
        }
    }
}

pub trait SettingModal : ModalWindowView {
    fn get_id(&self) -> u32;
    fn get_config(&self) -> ModalConfig;
    fn update_setting_modal(&mut self, message: SettingModalMessage, module: &mut ModuleWidget) -> Task<Message>;
    fn setting_content(&self) -> Element<'_, SettingModalMessage>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: SettingModal> ModalWindowView for T {
    fn get_id(&self) -> Option<u32> {
        Some(<Self as SettingModal>::get_id(self))
    }
    fn get_config(&self) -> ModalConfig {
        <Self as SettingModal>::get_config(self)
    }

    fn content(&self) -> Element<'_, ModalWindowMessage> {
        <Self as SettingModal>::setting_content(self).map(ModalWindowMessage::from)
    }

    fn update(&mut self, _message: ModalWindowMessage) -> Task<Message> {
        println!("ModalWindowView::update called directly, which is not supported.");
        Task::none()
    }

    fn as_any(&self) -> &dyn Any {
        <Self as SettingModal>::as_any(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        <Self as SettingModal>::as_any_mut(self)
    }
}