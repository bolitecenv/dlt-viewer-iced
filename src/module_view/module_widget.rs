use iced::{Color, Point, Size, widget::canvas};

use crate::{components::dlt_data_manager::DltDataRegexItem, message::Message};

// Constants
pub const RESIZE_HANDLE_SIZE: f32 = 10.0;
pub const MIN_CHART_WIDTH: f32 = 200.0;
pub const MIN_CHART_HEIGHT: f32 = 200.0;

#[derive(Debug, Clone)]
pub struct ChartData {
    pub x_value: f32,
    pub y_value: f32,
}

#[derive(Debug, Clone)]
pub struct ChartSettings {
    pub show_grid: bool,
    pub show_legend: bool,
    pub line_smoothness: f32,
    pub x_label: String,
    pub y_label: String,
}

#[derive(Debug, Clone)]
pub struct ChartWidget {
    pub chart_data: Vec<ChartData>,
    pub settings: ChartSettings,
}

#[derive(Debug, Clone)]
pub struct GanttChartSettings {
    pub time_scale: f32,
    pub show_dependencies: bool,
}

#[derive(Debug, Clone)]
pub struct GanttChartDataPoint {
    pub y_label: String,
    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Debug, Clone)]
pub struct GanttChartData {
    pub data_points: Vec<GanttChartDataPoint>,
}

#[derive(Debug, Clone)]
pub struct GanttChartWidget {
    pub chart_data: GanttChartData,
    pub settings: GanttChartSettings,
}

#[derive(Debug, Clone)]
pub enum WidgetTpye {
    LineChart(ChartWidget),
    BarChart(ChartWidget),
    GanttChart(GanttChartWidget),
}

#[derive(Debug, Clone)]
pub struct ModuleWidgetCommonSettings {
    pub title: String,
    pub show_title: bool,
    pub background_color: Color,
    pub color: Color,
    pub x_zoom: f32,
    pub y_zoom: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(Debug, Clone)]
pub struct ModuleWidget {
    pub id: usize,
    pub position: Point,
    pub size: Size,
    pub settings: ModuleWidgetCommonSettings,
    pub widget_type: WidgetTpye,
    pub dlt_data_regex_item: Option<DltDataRegexItem>,
}

impl ModuleWidget {
    pub fn new(
        id: usize,
        position: Point,
        size: Size,
        settings: ModuleWidgetCommonSettings,
        widget_type: WidgetTpye,
    ) -> Self {
        Self {
            id,
            position,
            size,
            settings,
            widget_type,
            dlt_data_regex_item: None,
        }
    }

    pub fn default_gantt_chart_widget(id: usize, position: Point, size: Size) -> Self {
        let common_settings = ModuleWidgetCommonSettings {
            title: "Gantt Chart".to_string(),
            show_title: true,
            background_color: Color::from_rgb(1.0, 1.0, 1.0),
            color: Color::from_rgb(0.0, 0.0, 0.0),
            x_zoom: 1.0,
            y_zoom: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
        };

        let gantt_chart_settings = GanttChartSettings {
            time_scale: 1.0,
            show_dependencies: false,
        };

        let gantt_chart_data = GanttChartData {
            data_points: Vec::new(),
        };

        let gantt_chart_widget = GanttChartWidget {
            chart_data: gantt_chart_data,
            settings: gantt_chart_settings,
        };

        let dlt_data_regex_item = DltDataRegexItem {
            id,
            regex: r"([^>]+),([D]),(\d+)".to_string(),
            description: "Gantt Chart Data Extractor".to_string(),
        };

        Self {
            id,
            position,
            size,
            settings: common_settings,
            widget_type: WidgetTpye::GanttChart(gantt_chart_widget),
            dlt_data_regex_item: Some(dlt_data_regex_item),
        }
    }

    pub fn default_chart_widget(id: usize, position: Point, size: Size) -> Self {
        let common_settings = ModuleWidgetCommonSettings {
            title: "Chart".to_string(),
            show_title: true,
            background_color: Color::from_rgb(1.0, 1.0, 1.0),
            color: Color::from_rgb(0.0, 0.0, 0.0),
            x_zoom: 1.0,
            y_zoom: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
        };

        let chart_settings = ChartSettings {
            show_grid: true,
            show_legend: true,
            line_smoothness: 0.5,
            x_label: "X-Axis".to_string(),
            y_label: "Y-Axis".to_string(),
        };

        let chart_data = ChartData {
            x_value: 0.0,
            y_value: 0.0,
        };

        let chart_widget = ChartWidget {
            chart_data: vec![chart_data],
            settings: chart_settings,
        };

        Self {
            id,
            position,
            size,
            settings: common_settings,
            widget_type: WidgetTpye::LineChart(chart_widget),
            dlt_data_regex_item: None,
        }
    }
}

impl WidgetTpye {
    // Immutable access (what you have)
    pub fn get_chart_settings(&self) -> Option<&ChartSettings> {
        match self {
            WidgetTpye::LineChart(chart_widget) => Some(&chart_widget.settings),
            WidgetTpye::BarChart(chart_widget) => Some(&chart_widget.settings),
            WidgetTpye::GanttChart(_) => None,
        }
    }

    // Mutable access (what you need)
    pub fn get_chart_settings_mut(&mut self) -> Option<&mut ChartSettings> {
        match self {
            WidgetTpye::LineChart(chart_widget) => Some(&mut chart_widget.settings),
            WidgetTpye::BarChart(chart_widget) => Some(&mut chart_widget.settings),
            WidgetTpye::GanttChart(_) => None,
        }
    }
}

impl ModuleWidget {
    pub fn get_dlt_data_regex_item(&self) -> Option<&DltDataRegexItem> {
        match &self.dlt_data_regex_item {
            Some(item) => Some(item),
            None => None,
        }
    }

    pub fn get_dlt_regex_pattern_mut(&mut self) -> Option<&mut String> {
        self.dlt_data_regex_item
            .as_mut()
            .map(|item| &mut item.regex)
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
