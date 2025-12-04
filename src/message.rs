use crate::components::confirm_modal_window::ConfirmMessage;
use crate::components::tcp_handler::EcuUpdateInfo;
use crate::module_view::ModuleCanvas;
use crate::module_view::canvas::{ModuleCanvasMessage};
use crate::plugin::PluginMessage;
use crate::{
    module_view::{ModuleWidget, canvas::ContextMenuAction},
    pages::table::DltMessageRow,
};
use iced::Point;

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
    DltMessageReceived(Vec<DltMessageRow>),
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
    RefreshDltItems,
    ApplyDltSettings,
    SelectDltEcu(String),
    SelectDltApp(String, String),
    SelectDltContext(String, String, String),
    CloseDltSettings,
    OpenDltSettings,
    UpdateLogLevel(String),
    UpdateTraceStatus(String),
    SaveContextSettings,
    CancelEditContext,
    EditContext(i8, i8),


    // Module View Messages
    MouseWheel(usize, f32),
    ShiftKeyChanged(bool),

    ToggleGrid,
    ToggleLegend,

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

    Confirm(ConfirmMessage),
    OpenSettingsModal,
    CloseSettingsModal,
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
