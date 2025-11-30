use iced::{Color, Point, Size, widget::canvas};

use crate::{components::dlt_data_manager::DltDataRegexItem, message::Message};

// Constants
pub const RESIZE_HANDLE_SIZE: f32 = 10.0;
pub const MIN_CHART_WIDTH: f32 = 200.0;
pub const MIN_CHART_HEIGHT: f32 = 200.0;

#[derive(Debug, Clone)]
pub struct ModuleWidget<T: ModuleWidgetWindowView> {
    pub id: usize,
    pub module_widget: T,
    pub dlt_data_regex_item: Option<DltDataRegexItem>,
}

impl<T: ModuleWidgetWindowView> ModuleWidget<T> {
    pub fn new(id: usize, module_widget: T, dlt_data_regex_item: Option<DltDataRegexItem>) -> Self {
        Self {
            id,
            module_widget,
            dlt_data_regex_item,
        }
    }
}

pub struct ModuleWidgetWindow {
    pub position: Point,
    pub size: Size,
    pub border_color: Color,
    pub border_width: f32,
    pub bg_color: Color,
}

// Default window
impl ModuleWidgetWindow {
    pub fn default() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
            size: Size::new(100.0, 100.0),
            border_color: Color::from_rgb(0.0, 0.0, 0.0),
            border_width: 1.0,
            bg_color: Color::from_rgb(1.0, 1.0, 1.0),
        }
    }
}
pub struct ModuleScreen<T: ModuleWidgetWindowView> {
    pub widget_components: Vec<T>,
}

// New trait for module widget window view and move, resize
pub trait ModuleWidgetWindowView {
    fn get_window(&self) -> &ModuleWidgetWindow;
    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow;
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

pub struct CardWidget {
    pub window: ModuleWidgetWindow,
    pub title: String,
    pub content: String,
}

impl ModuleWidgetWindowView for CardWidget {
    fn get_window(&self) -> &ModuleWidgetWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow {
        &mut self.window
    }

    fn draw(&self, frame: &mut canvas::Frame) {
        // Box
        let box_size = Size::new(self.window.size.width, self.window.size.height);
        frame.fill_rectangle(
            self.window.position,
            box_size,
            Color::from_rgb(0.9, 0.9, 0.9),
        );

        // Title
        let title_size = Size::new(self.window.size.width, 20.0);
        frame.fill_rectangle(
            self.window.position,
            title_size,
            Color::from_rgb(0.8, 0.8, 0.8),
        );
        frame.fill_text(canvas::Text {
            content: self.title.clone(),
            position: self.window.position,
            color: Color::from_rgb(0.0, 0.0, 0.0),
            size: 12.into(),
            ..canvas::Text::default()
        });

        // Content
        let content_size = Size::new(self.window.size.width, self.window.size.height - 20.0);
        frame.fill_rectangle(
            self.window.position,
            content_size,
            Color::from_rgb(0.7, 0.7, 0.7),
        );
        frame.fill_text(canvas::Text {
            content: self.content.clone(),
            position: self.window.position,
            color: Color::from_rgb(0.0, 0.0, 0.0),
            size: 12.into(),
            ..canvas::Text::default()
        });
    }
}
