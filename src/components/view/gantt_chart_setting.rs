use crate::message::Message;
use crate::components::view::modal_window::{ModalWindow, ModalContent, ModalConfig, ModalState};
use crate::module_view::ModuleWidget;
use crate::module_view::module_widget::{GanttChartSettings, GanttChartWidget, ModuleWidgetCommonSettings, WidgetTpye};
use iced::{
    Color, Element, Length, Theme,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input, pick_list, checkbox
    }
};

#[derive(Debug, Clone)]
pub enum ModuleGanttChartWidgetSettingsMessage {
    UpdateChartTitle(String),
    UpdateTimeScale(String),
    UpdateRegexPattern(String),
    ToggleDependencies,
}

pub struct ModalWindow_ModuleGanttChartWidgetSettingsView {
    pub modal_state: ModalState,
    pub widget: Option<ModuleWidget>,
    pub time_scale_input: String,
}

impl ModalWindow_ModuleGanttChartWidgetSettingsView {
    pub fn new() -> Self {
        Self {
            modal_state: ModalState::new(),
            widget: None,
            time_scale_input: String::from("1.0"),
        }
    }

    pub fn open(&mut self, widget: ModuleWidget) {
        self.modal_state.open();
        
        
        self.widget = Some(widget);
    }

    pub fn close(&mut self) {
        self.modal_state.close();
    }

    pub fn toggle(&mut self) {
        self.modal_state.toggle();
    }

    pub fn is_open(&self) -> bool {
        self.modal_state.is_open()
    }

    pub fn view<'a>(&self, dark_mode: bool) -> Option<Element<'a, Message>> {
        ModalWindow::view(self, dark_mode, &self.modal_state)
    }

    pub fn update_title(&mut self, new_title: String) {

    }

    pub fn update_time_scale(&mut self, new_scale: String) {
        self.time_scale_input = new_scale.clone();
        

    }

    pub fn update_regex_pattern(&mut self, new_pattern: String) {
        if let Some(widget) = &mut self.widget {
            if let Some(ref mut regex_item) = widget.dlt_data_regex_item {
                regex_item.regex = new_pattern;
            }
        }
    }

    pub fn toggle_dependencies(&mut self) {

    }

    fn build_settings_form<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        let text_color = if dark_mode { Color::WHITE } else { Color::BLACK };
        let label_color = if dark_mode { 
            Color::from_rgb(0.7, 0.7, 0.7) 
        } else { 
            Color::from_rgb(0.5, 0.5, 0.5) 
        };


    }
}

impl ModalContent<Message> for ModalWindow_ModuleGanttChartWidgetSettingsView {
    fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        let settings_form = self.build_settings_form(dark_mode);

        // Single panel layout for settings
        ModalWindow::panel_container(
            scrollable(settings_form)
                .width(Length::Fill)
                .height(Length::Fill),
            dark_mode,
            Length::Fill,
            Length::Fixed(400.0),
        )
    }

    fn close_message(&self) -> Message {
        Message::CloseChartSettings(self.widget.clone().unwrap())
    }

    fn refresh_message(&self) -> Option<Message> {
        Some(Message::RefreshChartData)
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::ApplyChartSettings)
    }

    fn config(&self) -> ModalConfig {
        ModalConfig {
            width: 700.0,
            height: 500.0,
            title: "Gantt Chart Settings".to_string(),
            show_refresh: true,
            show_apply: true,
        }
    }
}