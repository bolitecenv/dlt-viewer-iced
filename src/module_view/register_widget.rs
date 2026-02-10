use iced::{
    widget::canvas::{self, Frame, Path, Stroke, Text},
    Color, Point, Size, Theme,
};
use std::any::Any;
use std::collections::HashMap;

use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::module_view::{
    module_widget::{ModuleWidgetWindow, ModuleWidgetWindowView, WidgetData},
    setting_modals::setting_modal_window::SettingModal,
};

const MAX_REGISTERS: usize = 50;
const REGISTER_HEIGHT: f32 = 25.0;
const PADDING: f32 = 10.0;

#[derive(Debug, Clone)]
pub struct RegisterData {
    pub name: String,
    pub value: String,
    pub timestamp: std::time::Instant,
}

pub struct RegisterWidget {
    pub window: ModuleWidgetWindow,
    pub registers: HashMap<String, RegisterData>,
    pub register_order: Vec<String>, // To maintain display order
    pub max_registers: usize,
}

impl RegisterWidget {
    pub fn new(position: Point, size: Size, title: String) -> Self {
        Self {
            window: ModuleWidgetWindow {
                position,
                initial_position: position,
                size,
                border_color: Color::from_rgb(0.4, 0.6, 0.8),
                border_width: 2.0,
                bg_color: Color::from_rgb(0.15, 0.15, 0.15),
                title,
                subtitle: String::new(),
            },
            registers: HashMap::new(),
            register_order: Vec::new(),
            max_registers: MAX_REGISTERS,
        }
    }

    pub fn add_register(&mut self, name: String, value: String) {
        let data = RegisterData {
            name: name.clone(),
            value,
            timestamp: std::time::Instant::now(),
        };

        if !self.registers.contains_key(&name) {
            // New register - add to order
            self.register_order.push(name.clone());
            
            // Remove oldest if we exceed max
            if self.register_order.len() > self.max_registers {
                let oldest = self.register_order.remove(0);
                self.registers.remove(&oldest);
            }
        }

        self.registers.insert(name, data);
    }

    pub fn clear_registers(&mut self) {
        self.registers.clear();
        self.register_order.clear();
    }
}

impl ModuleWidgetWindowView for RegisterWidget {
    fn get_window(&self) -> &ModuleWidgetWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow {
        &mut self.window
    }

    fn add_new_data_item(&mut self, data: &WidgetData) {
        match data {
            WidgetData::Register(reg_data) => {
                self.add_register(reg_data.name.clone(), reg_data.value.clone());
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, dark_mode: bool) {
        let bounds = iced::Rectangle {
            x: self.window.position.x,
            y: self.window.position.y,
            width: self.window.size.width,
            height: self.window.size.height,
        };

        // Draw background
        let background = Path::rectangle(
            Point::new(bounds.x, bounds.y),
            Size::new(bounds.width, bounds.height),
        );
        frame.fill(&background, Color::from_rgb(0.15, 0.15, 0.15));

        // Draw border
        frame.stroke(
            &background,
            Stroke::default()
                .with_color(Color::from_rgb(0.4, 0.6, 0.8))
                .with_width(2.0),
        );

        // Draw title
        let title_text = Text {
            content: self.window.title.clone(),
            position: Point::new(bounds.x + PADDING, bounds.y + PADDING),
            color: Color::WHITE,
            size: 16.0.into(),
            ..Text::default()
        };
        frame.fill_text(title_text);

        // Draw column headers
        let header_y = bounds.y + PADDING + 25.0;
        let name_header = Text {
            content: "Register".to_string(),
            position: Point::new(bounds.x + PADDING, header_y),
            color: Color::from_rgb(0.7, 0.7, 0.7),
            size: 12.0.into(),
            ..Text::default()
        };
        frame.fill_text(name_header);

        let value_header = Text {
            content: "Value".to_string(),
            position: Point::new(bounds.x + bounds.width * 0.5, header_y),
            color: Color::from_rgb(0.7, 0.7, 0.7),
            size: 12.0.into(),
            ..Text::default()
        };
        frame.fill_text(value_header);

        // Draw separator line
        let separator_y = header_y + 20.0;
        let separator = Path::line(
            Point::new(bounds.x + PADDING, separator_y),
            Point::new(bounds.x + bounds.width - PADDING, separator_y),
        );
        frame.stroke(
            &separator,
            Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.3, 0.3))
                .with_width(1.0),
        );

        // Draw registers
        let mut current_y = separator_y + 10.0;
        let visible_height = bounds.height - (current_y - bounds.y) - PADDING;
        let max_visible = (visible_height / REGISTER_HEIGHT) as usize;

        for (idx, reg_name) in self.register_order.iter().rev().enumerate() {
            if idx >= max_visible {
                break;
            }

            if let Some(reg_data) = self.registers.get(reg_name) {
                // Alternate row background
                if idx % 2 == 0 {
                    let row_bg = Path::rectangle(
                        Point::new(bounds.x, current_y - 2.0),
                        Size::new(bounds.width, REGISTER_HEIGHT),
                    );
                    frame.fill(&row_bg, Color::from_rgba(1.0, 1.0, 1.0, 0.05));
                }

                // Register name
                let name_text = Text {
                    content: reg_data.name.clone(),
                    position: Point::new(bounds.x + PADDING, current_y),
                    color: Color::from_rgb(0.9, 0.9, 0.9),
                    size: 12.0.into(),
                    ..Text::default()
                };
                frame.fill_text(name_text);

                // Register value
                let value_color = if reg_data.value.starts_with("0x") {
                    Color::from_rgb(0.5, 0.8, 1.0) // Blue for hex
                } else {
                    Color::from_rgb(0.5, 1.0, 0.5) // Green for decimal
                };

                let value_text = Text {
                    content: reg_data.value.clone(),
                    position: Point::new(bounds.x + bounds.width * 0.5, current_y),
                    color: value_color,
                    size: 12.0.into(),
                    ..Text::default()
                };
                frame.fill_text(value_text);

                current_y += REGISTER_HEIGHT;
            }
        }

        // Draw scroll indicator if needed
        if self.register_order.len() > max_visible {
            let scroll_text = Text {
                content: format!("({} more...)", self.register_order.len() - max_visible),
                position: Point::new(bounds.x + PADDING, bounds.y + bounds.height - 20.0),
                color: Color::from_rgb(0.5, 0.5, 0.5),
                size: 10.0.into(),
                ..Text::default()
            };
            frame.fill_text(scroll_text);
        }
    }

    fn zoom(&mut self, _delta: f32, _shift: bool, _ctrl: bool) {
        // Register widgets don't zoom
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView> {
        Box::new(Self {
            window: self.window.clone(),
            registers: self.registers.clone(),
            register_order: self.register_order.clone(),
            max_registers: self.max_registers,
        })
    }

    fn open_settings_modal(
        &self,
        _id: u32,
        _regex_item: DltDataRegexItem,
    ) -> Option<Box<dyn SettingModal>> {
        // TODO: Implement settings modal for register widget
        None
    }
}

#[derive(Debug, Clone)]
pub struct RegisterWidgetData {
    pub name: String,
    pub value: String,
}
