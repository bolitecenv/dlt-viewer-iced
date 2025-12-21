use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::message::Message;
use crate::modal_window::modal_window::ModalConfig;
use crate::module_view::setting_modals::setting_modal_window::{SettingModal, SettingModalMessage};
use crate::module_view::{GanttChartWidget, ModuleWidget};
use crate::utility::util::{serialize_message, deserialize_message};
use iced::Task;
use iced::{
    Color, Element, Length,
    widget::{
        column, row, text, Space, text_input, checkbox
    }
};
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum GanttModalMessage {
    UpdateGanttTitle(String),
    UpdateXAxisLabel(String),
    UpdateRegexPattern(String),
    UpdateRowHeight(String),
    UpdateBarHeight(String),
    ToggleGrid,
    ToggleLabels,
    Confirm,
    Cancel,
    Refresh,
}

pub struct GanttWidgetModal {
    pub id: u32,
    pub title: String,
    pub regex_item: DltDataRegexItem,
    pub gantt_widget: GanttChartWidget,
}

impl GanttWidgetModal {
    pub fn new(id: u32, title: String, regex_item: DltDataRegexItem, gantt_widget: GanttChartWidget) -> Self {
        Self {
            id,
            title,
            regex_item,
            gantt_widget,
        }
    }

    fn create_custom_message(msg: GanttModalMessage) -> SettingModalMessage {
        let data = serialize_message(&msg).unwrap();
        SettingModalMessage::Custom("gantt_modal".to_string(), data)
    }
}

impl SettingModal for GanttWidgetModal {
    fn get_id(&self) -> u32 {
        self.id
    }

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

    fn setting_content(&self) -> Element<'_, SettingModalMessage> {
        let text_color = Color::WHITE;
        let label_color = Color::from_rgb(0.7, 0.7, 0.7);

        column![
            // Gantt Title
            row![
                text("Title:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter gantt title", &self.gantt_widget.window.title)
                    .on_input(|title| Self::create_custom_message(GanttModalMessage::UpdateGanttTitle(title)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new().width(Length::Shrink).height(Length::Fixed(15.0)),

            // X Axis Label
            row![
                text("X Axis Label:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter X axis label", &self.gantt_widget.settings.x_label)
                    .on_input(|label| Self::create_custom_message(GanttModalMessage::UpdateXAxisLabel(label)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new().width(Length::Shrink).height(Length::Fixed(20.0)),

            text("Regex Pattern for Gantt Data Extraction").size(16).color(text_color),

            Space::new().width(Length::Shrink).height(Length::Fixed(10.0)),

            row![
                text("Regex Pattern:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter regex pattern", &self.regex_item.regex)
                    .on_input(|pattern| Self::create_custom_message(GanttModalMessage::UpdateRegexPattern(pattern)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new().width(Length::Shrink).height(Length::Fixed(20.0)),

            // Display Options Section
            text("Display Options").size(16).color(text_color),

            Space::new().width(Length::Shrink).height(Length::Fixed(10.0)),

            // Row Height
            row![
                text("Row Height:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter row height", &self.gantt_widget.settings.row_height.to_string())
                    .on_input(|height| Self::create_custom_message(GanttModalMessage::UpdateRowHeight(height)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new().width(Length::Shrink).height(Length::Fixed(10.0)),

            // Bar Height
            row![
                text("Bar Height:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text_input("Enter bar height", &self.gantt_widget.settings.bar_height.to_string())
                    .on_input(|height| Self::create_custom_message(GanttModalMessage::UpdateBarHeight(height)))
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),

            Space::new().width(Length::Shrink).height(Length::Fixed(10.0)),

            // Toggle Grid
            row![
                text("Show Grid").size(14).color(label_color).width(Length::Fixed(120.0)),
                checkbox(self.gantt_widget.settings.show_grid)
                    .on_toggle(|_| Self::create_custom_message(GanttModalMessage::ToggleGrid)),
            ]
            .spacing(10),

            Space::new().width(Length::Shrink).height(Length::Fixed(10.0)),

            // Toggle Labels
            row![
                text("Show Labels").size(14).color(label_color).width(Length::Fixed(120.0)),
                checkbox(self.gantt_widget.settings.show_labels)
                    .on_toggle(|_| Self::create_custom_message(GanttModalMessage::ToggleLabels)),
            ]
            .spacing(10),

            Space::new().width(Length::Shrink).height(Length::Fixed(15.0)),
        ]
        .spacing(5)
        .padding(20)
        .into()
    }

    fn update(&mut self, message: SettingModalMessage, module: Option<&mut ModuleWidget>) -> Task<Message> {
        match message {
            SettingModalMessage::Apply => {
                println!("Confirmed");
                println!("Module ID: {}", module.as_ref().map_or(0, |m| m.id));

                if let Some(m) = module {
                    if let Some(gantt_widget) = m.module_widget.as_any_mut().downcast_mut::<GanttChartWidget>() {
                        *gantt_widget = self.gantt_widget.clone();
                    }
                    if let Some(dlt_data_regex_item) = m.dlt_data_regex_item.as_mut() {
                        *dlt_data_regex_item = self.regex_item.clone();
                    }
                }
                return Task::done(Message::CloseSettingsModal);
            }
            SettingModalMessage::Close => {
                return Task::done(Message::CloseSettingsModal);
            }
            SettingModalMessage::Refresh => {
                println!("Refreshed");
            }
            SettingModalMessage::Custom(msg_type, data) => {
                if msg_type == "gantt_modal" {
                    if let Ok(modal_msg) = deserialize_message::<GanttModalMessage>(&data) {
                        match modal_msg {
                            GanttModalMessage::UpdateGanttTitle(title) => {
                                self.gantt_widget.window.title = title;
                            }
                            GanttModalMessage::UpdateXAxisLabel(label) => {
                                self.gantt_widget.settings.x_label = label;
                            }
                            GanttModalMessage::UpdateRegexPattern(pattern) => {
                                self.regex_item.regex = pattern;
                            }
                            GanttModalMessage::UpdateRowHeight(height_str) => {
                                if let Ok(height) = height_str.parse::<f32>() {
                                    self.gantt_widget.settings.row_height = height;
                                }
                            }
                            GanttModalMessage::UpdateBarHeight(height_str) => {
                                if let Ok(height) = height_str.parse::<f32>() {
                                    self.gantt_widget.settings.bar_height = height;
                                }
                            }
                            GanttModalMessage::ToggleGrid => {
                                self.gantt_widget.settings.show_grid = !self.gantt_widget.settings.show_grid;
                            }
                            GanttModalMessage::ToggleLabels => {
                                self.gantt_widget.settings.show_labels = !self.gantt_widget.settings.show_labels;
                            }
                            GanttModalMessage::Confirm => {
                                println!("Gantt settings confirmed");
                            }
                            GanttModalMessage::Cancel => {
                                println!("Gantt settings cancelled");
                            }
                            GanttModalMessage::Refresh => {
                                println!("Gantt settings refreshed");
                            }
                        }
                    }
                }
            }
        }
        Task::none()
    }
}