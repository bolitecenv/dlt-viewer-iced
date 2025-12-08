use std::any::Any;

use chrono::offset;
use iced::{Color, Point, Size, advanced::svg::Data, widget::canvas};

use crate::{components::dlt_data_manager::DltDataRegexItem, message::Message, module_view::{ChartWidget, ModuleCanvas, canvas::{GRID_SIZE, SNAP_THRESHOLD}, chart_widget::ChartData}};

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

#[derive(Debug, Clone)]
pub enum WidgetData {
    Chart(ChartData),
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

    pub fn add_new_data(&mut self, data: &String) {
        if let Some(dlt_regex_item) = &self.dlt_data_regex_item {
            let regex = regex::Regex::new(&dlt_regex_item.regex).unwrap();
            if regex.is_match(data) {
                // Here you can handle the matched message as needed
                println!("Module ID {}: Message matched regex {}: {}", self.id, dlt_regex_item.regex, data);
                if let Some(widget_data) = self.process_data_for_widget(data) {
                    // Handle the processed widget data
                    self.module_widget.add_new_data_item(&widget_data);
                }
            }
        }
    }

    fn process_data_for_widget(&mut self, data: &String) -> Option<WidgetData> {
        if self.module_widget.as_any().is::<ChartWidget>() {
            if let Some(dlt_regex_item) = &self.dlt_data_regex_item {
                let regex = regex::Regex::new(&dlt_regex_item.regex).unwrap();
                if regex.is_match(data) {
                    let captures = regex.captures(data).unwrap();
                    let x_value: f32 = captures.name("X").unwrap().as_str().parse().unwrap_or(0.0);
                    let y_value: f32 = captures.name("Y").unwrap().as_str().parse().unwrap_or(0.0);
                    let chart_data = ChartData {
                        x_value: x_value,
                        y_value: y_value,
                    };
                    return Some(WidgetData::Chart(chart_data));
                }
            }
        }
        None
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

#[derive(Debug, Clone)]
pub struct ModuleWidgetWindow {
    pub position: Point,
    pub initial_position: Point,
    pub size: Size,
    pub border_color: Color,
    pub border_width: f32,
    pub bg_color: Color,
    pub title: String,
    pub subtitle: String,
}

// Default window
impl ModuleWidgetWindow {
    pub fn default() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
            initial_position: Point::new(0.0, 0.0),
            size: Size::new(200.0, 200.0),
            border_color: Color::from_rgb(0.0, 0.0, 0.0),
            border_width: 1.0,
            bg_color: Color::from_rgb(1.0, 1.0, 1.0),
            title: "Untitled".to_string(),
            subtitle: "".to_string(),
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

    fn move_window(&mut self, initial_mouse_position: Point, current_mouse_position: Point) {
        let window = self.get_window_mut();
        let offset_x = initial_mouse_position.x - window.initial_position.x;
        let offset_y = initial_mouse_position.y - window.initial_position.y;

        let delta_x = current_mouse_position.x - initial_mouse_position.x;
        let delta_y = current_mouse_position.y - initial_mouse_position.y;

        // Apply the delta to the current window position
        window.position.x = initial_mouse_position.x + delta_x - offset_x;
        window.position.y = initial_mouse_position.y + delta_y - offset_y;
    }

    fn set_window_initial_position(&mut self, position: Point) {
        let window = self.get_window_mut();
        window.initial_position = position;
    }

    fn get_window_position(&self) -> Point {
        let window = self.get_window();
        window.position
    }

    fn resize_window(&mut self, resize_type: ResizeType, position: Point) {
        let window = self.get_window_mut();
        
        match resize_type {
            ResizeType::Right => {
                window.size.width = (position.x - window.position.x).max(MIN_CHART_WIDTH);
            }
            ResizeType::Bottom => {
                window.size.height = (position.y - window.position.y).max(MIN_CHART_HEIGHT);
            }
            ResizeType::Corner => {
                window.size.width = (position.x - window.position.x).max(MIN_CHART_WIDTH);
                window.size.height = (position.y - window.position.y).max(MIN_CHART_HEIGHT);
            }
            _ => {}
        }
        window.size = sticky_snap_to_grid_size(window.size);
    }

    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn add_new_data_item(&mut self, data: &WidgetData);
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


fn sticky_snap_to_grid(value: f32, grid_size: f32, threshold: f32) -> f32 {
    let remainder = value % grid_size;
    if remainder < threshold {
        value - remainder
    } else if remainder > grid_size - threshold {
        value + (grid_size - remainder)
    } else {
        value
    }
}
fn sticky_snap_to_grid_size(size: Size) -> Size {
    let snapped_width = sticky_snap_to_grid(size.width, GRID_SIZE, SNAP_THRESHOLD);
    let snapped_height = sticky_snap_to_grid(size.height, GRID_SIZE, SNAP_THRESHOLD);
    Size::new(snapped_width, snapped_height)
}