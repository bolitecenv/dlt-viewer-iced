use std::any::Any;

use iced::{Color, Point, Size, widget::canvas};

use crate::{components::dlt_data_manager::DltDataRegexItem, message::Message};

// Constants
pub const RESIZE_HANDLE_SIZE: f32 = 10.0;
pub const MIN_CHART_WIDTH: f32 = 200.0;
pub const MIN_CHART_HEIGHT: f32 = 200.0;

pub const RESIZE_HANDLE_MARGIN: f32 = 5.0;
pub const HORIZONTAL_RESIZE_MARGIN: f32 = 5.0;
pub const VERTICAL_RESIZE_MARGIN: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeType {
    Left,
    Right,
    Top,
    Bottom,
    Corner,
}

pub struct ModuleWidget {
    pub id: usize,
    pub module_widget: Box<dyn ModuleWidgetWindowView>,
    pub dlt_data_regex_item: Option<DltDataRegexItem>,
}

impl ModuleWidget {
    pub fn new(id: usize, module_widget: Box<dyn ModuleWidgetWindowView>, dlt_data_regex_item: Option<DltDataRegexItem>) -> Self {
        Self {
            id,
            module_widget,
            dlt_data_regex_item,
        }
    }
}

impl Clone for ModuleWidget {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            module_widget: self.module_widget.clone_box(),
            dlt_data_regex_item: self.dlt_data_regex_item.clone(),
        }
    }
}

pub struct ModuleWidgetWindow {
    pub position: Point,
    pub size: Size,
    pub border_color: Color,
    pub border_width: f32,
    pub bg_color: Color,
    pub title: String,
}

// Default window
impl ModuleWidgetWindow {
    pub fn default() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
            size: Size::new(200.0, 200.0),
            border_color: Color::from_rgb(0.0, 0.0, 0.0),
            border_width: 1.0,
            bg_color: Color::from_rgb(1.0, 1.0, 1.0),
            title: "Untitled".to_string(),
        }
    }
}
pub struct ModuleScreen<T: ModuleWidgetWindowView> {
    pub widget_components: Vec<T>,
}

// New trait for module widget window view and move, resize
pub trait ModuleWidgetWindowView: Send + Sync {
    fn get_window(&self) -> &ModuleWidgetWindow;
    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow;
    fn get_window_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x + HORIZONTAL_RESIZE_MARGIN
            && point.x <= window.position.x + window.size.width - HORIZONTAL_RESIZE_MARGIN
            && point.y >= window.position.y + VERTICAL_RESIZE_MARGIN
            && point.y <= window.position.y + window.size.height - VERTICAL_RESIZE_MARGIN
    }
    fn get_window_some_resize_contains_point(&self, point: Point) -> bool {
        self.get_window_left_resize_contains_point(point)
            || self.get_window_right_resize_contains_point(point)
            || self.get_window_top_resize_contains_point(point)
            || self.get_window_bottom_resize_contains_point(point)
            || self.get_window_resize_handle_contains_point(point)
    }
    fn get_window_resize_type_contains_point(&self, point: Point) -> Option<ResizeType> {
        if self.get_window_left_resize_contains_point(point) {
            Some(ResizeType::Left)
        } else if self.get_window_right_resize_contains_point(point) {
            Some(ResizeType::Right)
        } else if self.get_window_top_resize_contains_point(point) {
            Some(ResizeType::Top)
        } else if self.get_window_bottom_resize_contains_point(point) {
            Some(ResizeType::Bottom)
        } else if self.get_window_resize_handle_contains_point(point) {
            Some(ResizeType::Corner)
        } else {
            None
        }
    }
    fn get_window_resize_handle_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x + window.size.width - RESIZE_HANDLE_SIZE
            && point.x <= window.position.x + window.size.width + RESIZE_HANDLE_SIZE
            && point.y >= window.position.y + window.size.height - RESIZE_HANDLE_SIZE
            && point.y <= window.position.y + window.size.height + RESIZE_HANDLE_SIZE
    }
    fn get_window_right_resize_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x + window.size.width - HORIZONTAL_RESIZE_MARGIN
            && point.x <= window.position.x + window.size.width + HORIZONTAL_RESIZE_MARGIN
            && point.y >= window.position.y
            && point.y <= window.position.y + window.size.height - VERTICAL_RESIZE_MARGIN
    }
    fn get_window_left_resize_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x - HORIZONTAL_RESIZE_MARGIN
            && point.x <= window.position.x + HORIZONTAL_RESIZE_MARGIN
            && point.y >= window.position.y
            && point.y <= window.position.y + window.size.height
    }
    fn get_window_top_resize_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x
            && point.x <= window.position.x + window.size.width
            && point.y >= window.position.y - VERTICAL_RESIZE_MARGIN
            && point.y <= window.position.y + VERTICAL_RESIZE_MARGIN
    }
    fn get_window_bottom_resize_contains_point(&self, point: Point) -> bool {
        let window = self.get_window();
        point.x >= window.position.x
            && point.x <= window.position.x + window.size.width - HORIZONTAL_RESIZE_MARGIN
            && point.y >= window.position.y + window.size.height - VERTICAL_RESIZE_MARGIN
            && point.y <= window.position.y + window.size.height + VERTICAL_RESIZE_MARGIN
    }
    fn draw(&self, frame: &mut canvas::Frame);
    fn window_draw(&self, frame: &mut canvas::Frame) {
        // Draw window border and bg
        let window = self.get_window();
        frame.fill_rectangle(window.position, window.size, window.bg_color);

        self.draw(frame);
    }
    fn move_window(&mut self, new_position: Point) {
        let window = self.get_window_mut();
        window.position = new_position;
    }

    fn resize_window(&mut self, new_size: Size) {
        let window = self.get_window_mut();
        window.size = new_size;
    }

    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ModuleScreen<T>
where
    T: ModuleWidgetWindowView,
{
    pub fn new() -> Self {
        Self {
            widget_components: Vec::new(),
        }
    }

    pub fn draw(&self, frame: &mut canvas::Frame) {
        for widget in self.widget_components.iter() {
            widget.window_draw(frame);
        }
    }

    pub fn add_widget(&mut self, widget: T) {
        self.widget_components.push(widget);
    }
}
