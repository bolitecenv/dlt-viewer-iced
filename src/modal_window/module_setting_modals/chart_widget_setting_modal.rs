use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::message::Message;
use crate::modal_window::modal_window::{ModalConfig, ModalWindowMessage, ModalWindowView, deserialize_message, serialize_message};
use crate::module_view::{ChartWidget, ModuleWidget};
use bincode::{Decode, Encode};
use iced::Task;
use iced::{
    Color, Element, Length, Theme,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input, pick_list, checkbox
    }
};
use std::str::FromStr;

#[derive(Debug, Clone, Encode, Decode)]
pub enum ChartModalMessage {
    UpdateChartTitle(String),
    UpdateXAxisLabel(String),
    UpdateYAxisLabel(String),
    UpdateRegexPattern(String),
    ToggleLegend,
    ToggleGrid,
    Confirm,
    Cancel,
    Refresh,
}

impl FromStr for ChartModalMessage {
    type Err = ();
    
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "toggle_legend" => Ok(ChartModalMessage::ToggleLegend),
            "toggle_grid" => Ok(ChartModalMessage::ToggleGrid),
            _ => Err(()),
        }
    }
}

pub struct ChartWidgetModal {
    pub title: String,
    pub regex_item: DltDataRegexItem,
    pub chart_widget: ChartWidget,
}

impl ChartWidgetModal {
    pub fn new(title: String, regex_item: DltDataRegexItem, chart_widget: ChartWidget) -> Self {
        Self {
            title,
            regex_item,
            chart_widget,
        }
    }

    fn create_custom_message(msg: ChartModalMessage) -> ModalWindowMessage {
        let data = serialize_message(&msg).unwrap();
        ModalWindowMessage::Custom("chart_modal".to_string(), data)
    }

    pub fn handle_update(&mut self, message: ModalWindowMessage) -> Task<Message> {
        match message {
            ModalWindowMessage::Apply => {
                println!("Confirmed");
            }
            ModalWindowMessage::Close => {
                println!("Cancelled");
            }
            ModalWindowMessage::Refresh => {
                println!("Refreshed");
            }
            ModalWindowMessage::Custom(msg_type, data) => {
                if msg_type == "chart_modal" {
                    if let Ok(modal_msg) = deserialize_message::<ChartModalMessage>(&data) {
                        match modal_msg {
                            ChartModalMessage::UpdateChartTitle(title) => {
                                self.chart_widget.window.title = title;
                            }
                            ChartModalMessage::UpdateXAxisLabel(label) => {
                                self.chart_widget.settings.x_label = label;
                            }
                            ChartModalMessage::UpdateYAxisLabel(label) => {
                                self.chart_widget.settings.y_label = label;
                            }
                            ChartModalMessage::UpdateRegexPattern(pattern) => {
                                self.regex_item.regex = pattern;
                            }
                            ChartModalMessage::ToggleLegend => {
                                self.chart_widget.settings.show_legend = !self.chart_widget.settings.show_legend;
                            }
                            ChartModalMessage::ToggleGrid => {
                                self.chart_widget.settings.show_grid = !self.chart_widget.settings.show_grid;
                            }
                            ChartModalMessage::Confirm => {
                                println!("Chart settings confirmed");
                            }
                            ChartModalMessage::Cancel => {
                                println!("Chart settings cancelled");
                            }
                            ChartModalMessage::Refresh => {
                                println!("Chart settings refreshed");
                            }
                        }
                    }
                }
            }

            _ => {},
        }
        Task::none()
    }
}

impl ModalWindowView for ChartWidgetModal {
    fn get_config(&self) -> ModalConfig {
        ModalConfig {
            width: 700.0,
            height: 600.0,
            show_refresh: true,
            show_apply: true,
            can_apply: true,
            can_close: true,
            title: self.title.clone(),
            ..Default::default()
        }
    }

    fn update(&mut self, message: ModalWindowMessage) -> Task<Message> {
        self.handle_update(message)
    }

    fn content(&self) -> Element<'_, ModalWindowMessage> {
        let text_color = Color::WHITE;
        let label_color = Color::from_rgb(0.7, 0.7, 0.7);

        column![
            // Chart Title
            row![
                text("Title:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter chart title", &self.chart_widget.window.title)
                    .on_input(|title| Self::create_custom_message(ChartModalMessage::UpdateChartTitle(title)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(15.0)),

            // X Axis Label
            row![
                text("X Axis Label:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter X axis label", &self.chart_widget.settings.x_label)
                    .on_input(|label| Self::create_custom_message(ChartModalMessage::UpdateXAxisLabel(label)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(15.0)),

            // Y Axis Label
            row![
                text("Y Axis Label:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter Y axis label", &self.chart_widget.settings.y_label)
                    .on_input(|label| Self::create_custom_message(ChartModalMessage::UpdateYAxisLabel(label)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(20.0)),

            text("Regex Pattern for Gantt Data Extraction").size(16).color(text_color),

            Space::new(Length::Shrink, Length::Fixed(10.0)),

            row![
                text("Regex Pattern:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter regex pattern", &self.regex_item.regex)
                    .on_input(|pattern| Self::create_custom_message(ChartModalMessage::UpdateRegexPattern(pattern)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new(Length::Shrink, Length::Fixed(10.0)),

            // Display Options Section
            text("Display Options").size(16).color(text_color),
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),

            Space::new(Length::Shrink, Length::Fixed(15.0)),
        ]
        .spacing(5)
        .padding(20)
        .into()
    }
}