use iced::{Element, Task};
use crate::{message::Message, 
    modal_window::modal_window::{ModalConfig, ModalWindowMessage, ModalWindowView}, 
    module_view::{ModuleWidget}};


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
    fn setting_content(&self) -> Element<'_, SettingModalMessage>;
    fn update(&mut self, message: SettingModalMessage, module: Option<&mut ModuleWidget>) -> Task<Message>;
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

    fn update(&mut self, _message: ModalWindowMessage, _module: Option<&mut ModuleWidget>) -> Task<Message> {
        <Self as SettingModal>::update(self, SettingModalMessage::from(_message), _module)
    }
}