use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::message::Message;
use crate::modal_window::modal_window::{ModalConfig, ModalWindowView};
use crate::module_view::{ChartWidget, ModuleWidget};
use iced::{
    Color, Element, Length, Theme,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input, pick_list, checkbox
    }
};
use std::str::FromStr;

#[derive(Debug, Clone)]
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
    pub regex_item: DltDataRegexItem,
    pub chart_widget: ChartWidget,
}

impl ChartWidgetModal {
    pub fn new(regex_item: DltDataRegexItem, chart_widget: ChartWidget) -> Self {
        Self {
            regex_item,
            chart_widget,
        }
    }

    pub fn handle_update(&self, message: ChartModalMessage) -> Task<Message> {
        match message {
            ChartModalMessage::UpdateChartTitle(title) => {
                None
            }
            ChartModalMessage::UpdateXAxisLabel(label) => {
                None
            }
            ChartModalMessage::UpdateYAxisLabel(label) => {
                None
            }
            ChartModalMessage::UpdateRegexPattern(pattern) => {
                None
            }
            ChartModalMessage::ToggleLegend => {
                // Implement legend toggle logic
                None
            }
            ChartModalMessage::ToggleGrid => {
                // Implement grid toggle logic
                None
            }

            _ => None,
        }
    }
}

impl ModalWindowView for ChartWidgetModal {
    fn title(&self) -> String {
        "Chart Settings".to_string()
    }

    fn get_config(&self) -> ModalConfig {
        ModalConfig {
            width: 700.0,
            height: 600.0,
            show_refresh: true,
            show_apply: true,
            can_apply: true,
            can_close: true,
            title: self.title(),
            ..Default::default()
        }
    }

    fn close_message(&self) -> Message {
        Message::MessageModalWindow("cancel".to_string())
    }

    fn refresh_message(&self) -> Option<Message> {
        Some(Message::MessageModalWindow("refresh".to_string()))
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::MessageModalWindow("confirm".to_string()))
    }

    fn update(&self, message: String) -> Option<Message> {
        self.handle_update(message.parse().ok()?)
    }

    fn content(&self) -> Element<'_, Message> {
        let text_color = Color::WHITE;
        let label_color = Color::from_rgb(0.7, 0.7, 0.7);

        column![
            // Chart Title
            row![
                text("Title:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter chart title", &self.chart_widget.window.title)
                    .on_input(|title| Message::MessageModalWindow("cancel".to_string()))
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
                    .on_input(|label| Message::MessageModalWindow("cancel".to_string()))
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
                    .on_input(|label| Message::MessageModalWindow("cancel".to_string()))
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
                    .on_input(|pattern| Message::MessageModalWindow("cancel".to_string()))
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