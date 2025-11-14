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
        
        // Initialize time_scale_input from widget settings
        if let WidgetTpye::GanttChart(ref gantt_widget) = widget.widget_type {
            self.time_scale_input = gantt_widget.settings.time_scale.to_string();
        }
        
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
        if let Some(widget) = &mut self.widget {
            widget.settings.title = new_title;
        }
    }

    pub fn update_time_scale(&mut self, new_scale: String) {
        self.time_scale_input = new_scale.clone();
        
        // Try to parse and update the widget settings
        if let Ok(scale_value) = new_scale.parse::<f32>() {
            if let Some(widget) = &mut self.widget {
                if let WidgetTpye::GanttChart(ref mut gantt_widget) = widget.widget_type {
                    gantt_widget.settings.time_scale = scale_value;
                }
            }
        }
    }

    pub fn update_regex_pattern(&mut self, new_pattern: String) {
        if let Some(widget) = &mut self.widget {
            if let Some(ref mut regex_item) = widget.dlt_data_regex_item {
                regex_item.regex = new_pattern;
            }
        }
    }

    pub fn toggle_dependencies(&mut self) {
        if let Some(widget) = &mut self.widget {
            if let WidgetTpye::GanttChart(ref mut gantt_widget) = widget.widget_type {
                gantt_widget.settings.show_dependencies = !gantt_widget.settings.show_dependencies;
            }
        }
    }

    fn build_settings_form<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        let text_color = if dark_mode { Color::WHITE } else { Color::BLACK };
        let label_color = if dark_mode { 
            Color::from_rgb(0.7, 0.7, 0.7) 
        } else { 
            Color::from_rgb(0.5, 0.5, 0.5) 
        };

        // Get current settings from widget
        let (time_scale, show_dependencies) = if let Some(ref widget) = self.widget {
            if let WidgetTpye::GanttChart(ref gantt_widget) = widget.widget_type {
                (
                    gantt_widget.settings.time_scale,
                    gantt_widget.settings.show_dependencies,
                )
            } else {
                (1.0, false)
            }
        } else {
            (1.0, false)
        };

        let title = if let Some(ref widget) = self.widget {
            &widget.settings.title
        } else {
            ""
        };

        column![
            // Chart Title
            row![
                text("Title:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter chart title", title)
                    .on_input(|title| Message::UpdateGanttChartWidgetSettingsMessage(
                        ModuleGanttChartWidgetSettingsMessage::UpdateChartTitle(title)
                    ))
                    .size(14)
                    .width(Length::Fill)
                    .style(move |theme: &Theme, status| {
                        text_input::Style {
                            background: if dark_mode {
                                Color::from_rgb(0.2, 0.2, 0.2).into()
                            } else {
                                Color::WHITE.into()
                            },
                            border: iced::Border {
                                width: 1.0,
                                color: if dark_mode {
                                    Color::from_rgb(0.4, 0.4, 0.4)
                                } else {
                                    Color::from_rgb(0.7, 0.7, 0.7)
                                },
                                radius: 4.0.into(),
                            },
                            icon: text_input::default(theme, status).icon,
                            placeholder: text_input::default(theme, status).placeholder,
                            value: if dark_mode { Color::WHITE } else { Color::BLACK },
                            selection: text_input::default(theme, status).selection,
                        }
                    }),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(15.0)),

            // Time Scale
            row![
                text("Time Scale:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter time scale (e.g., 1.0)", &self.time_scale_input)
                    .on_input(|scale| Message::UpdateGanttChartWidgetSettingsMessage(
                        ModuleGanttChartWidgetSettingsMessage::UpdateTimeScale(scale)
                    ))
                    .size(14)
                    .width(Length::Fill)
                    .style(move |theme: &Theme, status| {
                        text_input::Style {
                            background: if dark_mode {
                                Color::from_rgb(0.2, 0.2, 0.2).into()
                            } else {
                                Color::WHITE.into()
                            },
                            border: iced::Border {
                                width: 1.0,
                                color: if dark_mode {
                                    Color::from_rgb(0.4, 0.4, 0.4)
                                } else {
                                    Color::from_rgb(0.7, 0.7, 0.7)
                                },
                                radius: 4.0.into(),
                            },
                            icon: text_input::default(theme, status).icon,
                            placeholder: text_input::default(theme, status).placeholder,
                            value: if dark_mode { Color::WHITE } else { Color::BLACK },
                            selection: text_input::default(theme, status).selection,
                        }
                    }),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(20.0)),

            text("Regex Pattern for Gantt Data Extraction").size(16).color(text_color),

            Space::new(Length::Shrink, Length::Fixed(10.0)),

            row![
                text("Regex Pattern:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter regex pattern", &self.widget.as_ref().unwrap().get_dlt_data_regex_item().unwrap().regex)
                    .on_input(|pattern| Message::UpdateGanttChartWidgetSettingsMessage(
                        ModuleGanttChartWidgetSettingsMessage::UpdateRegexPattern(pattern)
                    ))
                    .size(14)
                    .width(Length::Fill)
                    .style(move |theme: &Theme, status| {
                        text_input::Style {
                            background: if dark_mode {
                                Color::from_rgb(0.2, 0.2, 0.2).into()
                            } else {
                                Color::WHITE.into()
                            },
                            border: iced::Border {
                                width: 1.0,
                                color: if dark_mode {
                                    Color::from_rgb(0.4, 0.4, 0.4)
                                } else {
                                    Color::from_rgb(0.7, 0.7, 0.7)
                                },
                                radius: 4.0.into(),
                            },
                            icon: text_input::default(theme, status).icon,
                            placeholder: text_input::default(theme, status).placeholder,
                            value: if dark_mode { Color::WHITE } else { Color::BLACK },
                            selection: text_input::default(theme, status).selection,
                        }
                    }),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),


            // Display Options Section
            text("Display Options").size(16).color(text_color),
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),

            // Show Dependencies Checkbox
            checkbox("Show Dependencies", show_dependencies)
                .on_toggle(|_| Message::UpdateGanttChartWidgetSettingsMessage(
                    ModuleGanttChartWidgetSettingsMessage::ToggleDependencies
                ))
                .size(14)
                .text_size(14),

            Space::new(Length::Shrink, Length::Fixed(15.0)),
        ]
        .spacing(5)
        .into()
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