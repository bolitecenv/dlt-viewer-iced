use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::module_view::module_widget::{
    MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidget, ModuleWidgetWindow, ModuleWidgetWindowView, WidgetData
};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};
use regex::Regex;
use std::any::Any;

#[derive(Clone)]
pub struct ChartSettings {
    pub show_grid: bool,
    pub show_legend: bool,
    pub line_smoothness: f32,
    pub x_label: String,
    pub y_label: String,
}

#[derive(Debug, Clone)]
pub struct ChartData {
    pub x_value: f32,
    pub y_value: f32,
}

#[derive(Clone)]
pub struct ChartWidget {
    pub window: ModuleWidgetWindow,
    pub dlt_data_regex_item: Option<DltDataRegexItem>,
    pub settings: ChartSettings,
    pub datas: Vec<ChartData>,
    pub dark_mode: bool,
}

impl ModuleWidgetWindowView for ChartWidget {
    fn get_window(&self) -> &ModuleWidgetWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow {
        &mut self.window
    }

    fn draw(&self, frame: &mut canvas::Frame) {
        // Call the draw_chart function with proper parameters
        draw_chart_impl(frame, self, self.window.position, self.window.size, None);
    }
    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView> {
        Box::new(ChartWidget {
            window: ModuleWidgetWindow {
                position: self.window.position,
                size: self.window.size,
                border_color: self.window.border_color,
                border_width: self.window.border_width,
                bg_color: self.window.bg_color,
                title: self.window.title.clone(),
                subtitle: self.window.subtitle.clone(),
            },
            dlt_data_regex_item: None,
            settings: ChartSettings {
                show_grid: self.settings.show_grid,
                show_legend: self.settings.show_legend,
                line_smoothness: self.settings.line_smoothness,
                x_label: self.settings.x_label.clone(),
                y_label: self.settings.y_label.clone(),
            },
            datas: self.datas.clone(),
            dark_mode: self.dark_mode,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn add_new_data_item(&mut self, data: &WidgetData) {
        let WidgetData::Chart(chart_data) = data;
        self.datas.push(chart_data.clone());
    }
}


impl ChartWidget {
    pub fn new(dark_mode: bool, settings: ChartSettings) -> Self {
        Self {
            dark_mode,
            settings,
            datas: Vec::new(),
            window: ModuleWidgetWindow::default(),
            dlt_data_regex_item: None,
        }
    }

    fn draw_line_chart(
        &self,
        frame: &mut canvas::Frame,
        area: &Rectangle,
        chart_data: &[ChartData],
    ) {
        let padding = 40.0;
        let chart_width = area.width - 2.0 * padding;
        let chart_height = area.height - 2.0 * padding;

        // Draw axes
        let axis_color = Color::from_rgb(0.7, 0.7, 0.7);
        frame.stroke(
            &canvas::Path::line(
                Point::new(area.x + padding, area.y + padding),
                Point::new(area.x + padding, area.y + area.height - padding),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );
        frame.stroke(
            &canvas::Path::line(
                Point::new(area.x + padding, area.y + area.height - padding),
                Point::new(
                    area.x + area.width - padding,
                    area.y + area.height - padding,
                ),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );

        // Define label color (used for both axis labels and tick labels)
        let label_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };

        // Draw Y-axis label (rotated 90 degrees counter-clockwise)
        if !self.settings.y_label.is_empty() {
            frame.with_save(|frame| {
                let label_x = area.x + 5.0;  // Position it closer to the left edge
                let label_y = area.y + padding + chart_height / 2.0;

                frame.translate(iced::Vector::new(label_x, label_y));
                frame.rotate(-std::f32::consts::FRAC_PI_2);

                frame.fill_text(canvas::Text {
                    content: self.settings.y_label.clone(),
                    position: Point::new(0.0, 0.0),  // Center at rotation point
                    color: label_color,
                    size: 14.0.into(),
                    ..canvas::Text::default()
                });
            });
        }

        // Draw X-axis label (centered below the axis)
        if !self.settings.x_label.is_empty() {
            let x_label_width = self.settings.x_label.len() as f32 * 7.0; // Approximate text width
            frame.fill_text(canvas::Text {
                content: self.settings.x_label.clone(),
                position: Point::new(
                    area.x + padding + (chart_width - x_label_width) / 2.0,
                    area.y + area.height - 5.0,
                ),
                color: label_color,
                size: 14.0.into(),
                ..canvas::Text::default()
            });
        }

        if chart_data.is_empty() {
            return;
        }

        // Extract x and y values
        let x_values: Vec<f32> = chart_data.iter().map(|cd| cd.x_value).collect();
        let y_values: Vec<f32> = chart_data.iter().map(|cd| cd.y_value).collect();

        let max_y = y_values.iter().cloned().fold(0.0f32, f32::max);
        let min_y = y_values.iter().cloned().fold(f32::MAX, f32::min);
        let y_range = max_y - min_y;

        let max_x = x_values.iter().cloned().fold(0.0f32, f32::max);
        let min_x = x_values.iter().cloned().fold(f32::MAX, f32::min);
        let x_range = max_x - min_x;

        // For this version, no zoom/pan - use full range
        let y_min_visible = min_y;
        let y_max_visible = max_y;
        let x_min_visible = min_x;
        let x_max_visible = max_x;

        // Draw Y-axis tick labels
        let num_y_labels = 5;
        for i in 0..=num_y_labels {
            let ratio = i as f32 / num_y_labels as f32;
            let value = y_min_visible + ratio * (y_max_visible - y_min_visible);
            let y = area.y + area.height - padding - ratio * chart_height;

            frame.fill_text(canvas::Text {
                content: format!("{:.1}", value),
                position: Point::new(area.x + padding - 35.0, y - 6.0),
                color: label_color,
                size: 12.0.into(),
                ..canvas::Text::default()
            });
        }

        // Draw X-axis tick labels
        let num_x_labels = 5;
        for i in 0..=num_x_labels {
            let ratio = i as f32 / num_x_labels as f32;
            let value = x_min_visible + ratio * (x_max_visible - x_min_visible);
            let x = area.x + padding + ratio * chart_width;

            frame.fill_text(canvas::Text {
                content: format!("{:.1}", value),
                position: Point::new(x - 10.0, area.y + area.height - padding + 15.0),
                color: label_color,
                size: 12.0.into(),
                ..canvas::Text::default()
            });
        }

        // Build the line path
        let mut path_builder = canvas::path::Builder::new();
        let mut first_point = true;

        for data_point in chart_data.iter() {
            let x_val = data_point.x_value;
            let y_val = data_point.y_value;

            // Skip points outside visible range
            if x_val < x_min_visible || x_val > x_max_visible {
                first_point = true;
                continue;
            }

            // Calculate x position based on visible range
            let x_ratio = if x_max_visible - x_min_visible > 0.0 {
                ((x_val - x_min_visible) / (x_max_visible - x_min_visible)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let x = area.x + padding + x_ratio * chart_width;

            // Calculate y position based on visible range
            let y_ratio = if y_max_visible - y_min_visible > 0.0 {
                ((y_val - y_min_visible) / (y_max_visible - y_min_visible)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let y = area.y + area.height - padding - y_ratio * chart_height;

            if first_point {
                path_builder.move_to(Point::new(x, y));
                first_point = false;
            } else {
                path_builder.line_to(Point::new(x, y));
            }
        }

        let line_path = path_builder.build();
        frame.stroke(
            &line_path,
            canvas::Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.6, 0.9))
                .with_width(2.0),
        );
    }
}

// Standalone function for drawing chart
fn draw_chart_impl(
    frame: &mut canvas::Frame,
    chart_widget: &ChartWidget,
    position: Point,
    size: Size,
    cursor_position: Option<Point>,
) {
    // Ensure chart stays within bounds
    let safe_width = size.width.max(MIN_CHART_WIDTH);
    let safe_height = size.height.max(MIN_CHART_HEIGHT);

    let corner_radius = 8.0;
    let elevation_offset = 2.0;

    // Draw elevation shadow
    let shadow_color = if chart_widget.dark_mode {
        Color::from_rgba(0.0, 0.0, 0.0, 0.5)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.2)
    };

    let shadow_path = canvas::Path::rounded_rectangle(
        Point::new(
            position.x + elevation_offset,
            position.y + elevation_offset,
        ),
        Size::new(safe_width, safe_height),
        corner_radius.into(),
    );

    frame.fill(&shadow_path, shadow_color);

    // Draw card background
    let chart_bg = if chart_widget.dark_mode {
        Color::from_rgba(0.2, 0.2, 0.25, 0.95)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.95)
    };

    let bg_path = canvas::Path::rounded_rectangle(
        position,
        Size::new(safe_width, safe_height),
        corner_radius.into(),
    );

    frame.fill(&bg_path, chart_bg);

    // Draw border
    let border = canvas::Path::rounded_rectangle(
        position,
        Size::new(safe_width, safe_height),
        corner_radius.into(),
    );

    frame.stroke(
        &border,
        canvas::Stroke::default()
            .with_color(Color::from_rgba(0.8, 0.8, 0.8, 1.0))
            .with_width(1.0),
    );

    // Draw title
    let title_color = if chart_widget.dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    };

    frame.fill_text(canvas::Text {
        content: chart_widget.window.title.clone(),
        position: Point::new(
            position.x + 15.0,
            position.y + 15.0,
        ),
        color: title_color,
        size: 20.0.into(),
        ..canvas::Text::default()
    });

    // Draw subtitle
    let subtitle_color = if chart_widget.dark_mode {
        Color::from_rgb(0.8, 0.8, 0.8)
    } else {
        Color::from_rgb(0.2, 0.2, 0.2)
    };

    frame.fill_text(canvas::Text {
        content: chart_widget.window.subtitle.clone(),
        position: Point::new(
            position.x + 15.0,
            position.y + 42.0,
        ),
        color: subtitle_color,
        size: 14.0.into(),
        ..canvas::Text::default()
    });

    // Calculate available space for chart
    let header_height = 65.0;
    let bottom_padding = 15.0;
    let available_height = safe_height - header_height - bottom_padding;

    // Draw the line chart
    let chart_area = Rectangle {
        x: position.x + 15.0,
        y: position.y + header_height,
        width: safe_width - 30.0,
        height: available_height,
    };

    chart_widget.draw_line_chart(frame, &chart_area, &chart_widget.datas);
}