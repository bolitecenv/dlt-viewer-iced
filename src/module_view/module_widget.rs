use iced::{Color, Point, Size};

use crate::components::dlt_data_manager::DltDataRegexItem;

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
pub struct ChartWidget{
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
    pub fn new(id: usize, 
                position: Point, 
                size: Size, 
                settings: ModuleWidgetCommonSettings, 
                widget_type: WidgetTpye) -> Self {
        Self {
            id,
            position,
            size,
            settings,
            widget_type,
            dlt_data_regex_item : None,
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
            dlt_data_regex_item : Some(dlt_data_regex_item),
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
            dlt_data_regex_item : None,
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
        self.dlt_data_regex_item.as_mut().map(|item| &mut item.regex)
    }
}