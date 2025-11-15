use iced::Point;
use crate::plugin::PluginMessage;
use crate::{components::view::{gantt_chart_setting::ModuleGanttChartWidgetSettingsMessage, module_view_settings::{ChartType, ModuleChartWidgetSettingsMessage}}, module_view::{ModuleWidget, canvas::ContextMenuAction}, pages::table::DltMessageRow};

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
    MousePressed(Point),
    MouseReleased,
    MouseMoved(Point),
    StartResize(usize, Point),
    ShowContextMenu(Point),
    ContextMenuAction(ContextMenuAction),
    RightMouseReleased(Point),


    // Modal related messages\
    UpdateModuleChartWidgetSettingsMessage(ModuleChartWidgetSettingsMessage),
    UpdateGanttChartWidgetSettingsMessage(ModuleGanttChartWidgetSettingsMessage),
    UpdateChartType(ChartType),
    UpdateChartTitle(String),
    UpdateXAxisLabel(String),
    UpdateYAxisLabel(String),
    ToggleChartLegend(bool),
    ToggleChartGrid(bool),
    UpdateLineWidth(String),
    CloseChartSettings(ModuleWidget),
    RefreshChartData,
    ApplyChartSettings,

    // Module View Messages
    MouseWheel(usize, f32),
    ShiftKeyChanged(bool),

    ToggleGrid,
    ToggleLegend,

    PluginSelected(String),
    PluginMessage(String, PluginMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Overview,
    Reports,
    Settings,
    Table,
    ChartCanvas,
    PluginPage(String),
}