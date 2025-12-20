use crate::components::tcp_handler::EcuUpdateInfo;
use crate::modal_window::modal_window::ModalWindowMessage;
use crate::module_view::ModuleCanvas;
use crate::module_view::canvas::{ModuleCanvasMessage};
use crate::module_view::setting_modals::setting_modal_window::SettingModalMessage;
use crate::plugin::PluginMessage;
use crate::{
    pages::table::DltMessageRow,
};
use iced::Point;
use iced::widget::scrollable::Viewport;
use tokio::net::TcpStream;
use std::sync::{Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected(String, Arc<Mutex<TcpStream>>),
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTheme,
    NavigateTo(Page),
    Tick,
    TcpIpChanged(String),
    TcpPortChanged(String),
    ConnectTcp,
    ConnectionEvent(ConnectionEvent),
    ClearMessages,
    OpenDltSettings,
    UpdateLogLevel(String),
    UpdateTraceStatus(String),
    SaveContextSettings,
    CancelEditContext,

    ScrollChanged(Viewport),

    PluginSelected(String),
    PluginMessage(String, PluginMessage),

    EcuListUpdate(Vec<EcuUpdateInfo>),
    BatchUpdate {
        dlt_messages: Vec<DltMessageRow>,
        ecu_updates: Vec<EcuUpdateInfo>,
    },

    SelectContext(String, String, String),
    SelectApp(String, String),
    SelectEcu(String),
    InjectMessage(String, String, String, String),
    UpdateMessageType(String),
    UpdateInjectionMessage(String),
    ClearInjectionMessage,
    ECUViewEditContext(i8, i8),

    // Module Canvas Events
    ModuleCanvasMessage(ModuleCanvasMessage),

    OpenSettingsModal,
    CloseSettingsModal,
    ModalWindowMessage(ModalWindowMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Reports,
    ECUSetting,
    Settings,
    Table,
    ChartCanvas,
    PluginPage(String),
}