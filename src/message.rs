use crate::components::tcp_handler::EcuUpdateInfo;
use crate::modal_window::modal_window::ModalWindowMessage;
use crate::module_view::canvas::{ModuleCanvasMessage};
use crate::pages::overview::ConnectionType;
use crate::plugin::PluginMessage;
use crate::{
    pages::table::DltMessageRow,
};
use iced::widget::scrollable::Viewport;
use tokio::net::TcpStream;
use std::sync::{Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected(String, Arc<Mutex<TcpStream>>),
    Disconnected(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum SerialConnectionEvent {
    Connected(String),
    Disconnected(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTheme,
    NavigateTo(Page),
    Tick,
    TcpIpChanged(String),
    TcpPortChanged(String),
    TcpClientNameChanged(String),
    ConnectTcp,
    ConnectionEvent(ConnectionEvent),
    ClearMessages,
    OpenDltSettings,
    UpdateLogLevel(String),
    UpdateTraceStatus(String),
    SaveContextSettings,
    CancelEditContext,
    ConnectionTypeSelected(ConnectionType),
    SerialPortChanged(String),
    BaudRateChanged(String),
    ConnectSerial,
    SerialConnectionEvent(SerialConnectionEvent),

    ScrollChanged(Viewport),

    PluginMessage(String, PluginMessage),

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

    CloseSettingsModal,
    ModalWindowMessage(ModalWindowMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Overview,
    ECUSetting,
    Settings,
    Table,
    ChartCanvas,
    PluginPage(String),
}