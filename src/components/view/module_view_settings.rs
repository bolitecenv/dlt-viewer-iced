use crate::message::Message;
use crate::components::view::modal_window::{ModalWindow, ModalContent, ModalConfig, ModalState};
use crate::module_view::ModuleWidget;
use crate::module_view::module_widget::{ChartSettings, ChartWidget, ModuleWidgetCommonSettings, WidgetTpye};
use iced::{
    Color, Element, Length, Theme,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input, pick_list, checkbox
    }
};

#[derive(Debug, Clone)]
pub enum ModuleChartWidgetSettingsMessage {
    UpdateChartType(ChartType),
    UpdateChartTitle(String),
    UpdateXAxisLabel(String),
    UpdateYAxisLabel(String),
    UpdateRegexPattern(String),
    ToggleLegend,
    ToggleGrid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Line,
    Bar,
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Line => write!(f, "Line Chart"),
            ChartType::Bar => write!(f, "Bar Chart"),
        }
    }
}

pub struct ModalWindow_ModuleChartWidgetSettingsView {
    pub modal_state: ModalState,
    pub widget: Option<ModuleWidget>,
    pub chart_type: ChartType,
}

impl ModalWindow_ModuleChartWidgetSettingsView {
    pub fn new() -> Self {
        Self {
            modal_state: ModalState::new(),
            widget: None,
            chart_type: ChartType::Line,
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
        if let Some(widget) = &mut self.widget {
            widget.settings.title = new_title;
        }
    }

    pub fn update_x_label(&mut self, new_label: String) {
        if let Some(widget) = &mut self.widget {
            if let Some(chart_settings) = widget.widget_type.get_chart_settings_mut() {
                chart_settings.x_label = new_label;
            }
        }
    }

    pub fn update_y_label(&mut self, new_label: String) {
        if let Some(widget) = &mut self.widget {
            if let Some(chart_settings) = widget.widget_type.get_chart_settings_mut() {
                chart_settings.y_label = new_label;
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

    fn build_settings_form<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        let text_color = if dark_mode { Color::WHITE } else { Color::BLACK };
        let label_color = if dark_mode { 
            Color::from_rgb(0.7, 0.7, 0.7) 
        } else { 
            Color::from_rgb(0.5, 0.5, 0.5) 
        };

        column![
            // Chart Type Selection
            row![
                text("Chart Type:").size(14).color(label_color).width(Length::Fixed(120.0)),
                pick_list(
                    vec![ChartType::Line, ChartType::Bar],
                    Some(self.chart_type.clone()),
                    Message::UpdateChartType
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(15.0)),

            // Chart Title
            row![
                text("Title:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter chart title", &self.widget.as_ref().unwrap().settings.title)
                    .on_input(|title| Message::UpdateModuleChartWidgetSettingsMessage(ModuleChartWidgetSettingsMessage::UpdateChartTitle(title)))
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

            // X Axis Label
            row![
                text("X Axis Label:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter X axis label", &self.widget.as_ref().unwrap().widget_type.get_chart_settings().unwrap().x_label)
                    .on_input(|label| Message::UpdateModuleChartWidgetSettingsMessage(ModuleChartWidgetSettingsMessage::UpdateXAxisLabel(label)))
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

            // Y Axis Label
            row![
                text("Y Axis Label:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter Y axis label", &self.widget.as_ref().unwrap().widget_type.get_chart_settings().unwrap().y_label)
                    .on_input(|label| Message::UpdateModuleChartWidgetSettingsMessage(ModuleChartWidgetSettingsMessage::UpdateYAxisLabel(label)))
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
                    .on_input(|pattern| Message::UpdateModuleChartWidgetSettingsMessage(
                        ModuleChartWidgetSettingsMessage::UpdateRegexPattern(pattern)
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



            Space::new(Length::Shrink, Length::Fixed(15.0)),
        ]
        .spacing(5)
        .into()
    }
}

impl ModalContent<Message> for ModalWindow_ModuleChartWidgetSettingsView {
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
        //Message::CloseChartSettings(self.widget.clone().unwrap())
        Message::RefreshChartData
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
            height: 600.0,
            title: "Chart Settings".to_string(),
            show_refresh: true,
            show_apply: true,
        }
    }
}