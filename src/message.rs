#[derive(Debug, Clone)]
pub enum Message {
    IncrementMetric1,
    DecrementMetric1,
    IncrementMetric2,
    RefreshData,
    ToggleTheme,
    NavigateTo(Page),
    Tick,
    TcpIpChanged(String),
    TcpPortChanged(String),
    ConnectTcp,
    TcpConnectionResult(Result<String, String>),
    GanttAddTask,
    GanttRemoveTask(usize),
    GanttTaskNameChanged(String),
    GanttShowStartPicker,
    GanttShowEndPicker,
    GanttStartDateSelected(iced_aw::date_picker::Date),
    GanttEndDateSelected(iced_aw::date_picker::Date),
    GanttCancelStartPicker,
    GanttCancelEndPicker,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Overview,
    Analytics,
    Reports,
    Settings,
    GanttChart,
}
