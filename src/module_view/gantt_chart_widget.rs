use crate::module_view::module_widget::{
    MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidgetWindow, ModuleWidgetWindowView, WidgetData
};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};
use std::any::Any;

#[derive(Clone)]
pub struct GanttSettings {
    pub show_grid: bool,
    pub show_labels: bool,
    pub row_height: f32,
    pub bar_height: f32,
    pub x_label: String,
}

#[derive(Debug, Clone)]
pub struct GanttDataPoint {
    pub constructed: bool,
    pub label: String,
    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Debug, Clone)]
pub struct GanttStartData {
    pub label: String,
    pub start_time: f32,
}

#[derive(Debug, Clone)]
pub struct GanttEndData {
    pub label: String,
    pub end_time: f32,
}

#[derive(Clone)]
pub struct GanttChartWidget {
    pub window: ModuleWidgetWindow,
    pub settings: GanttSettings,
    pub datas: Vec<GanttDataPoint>,
    pub dark_mode: bool,
    pub pan_offset_x: f32,
    pub pan_offset_y: f32,
    pub zoom_x: f32,
    pub zoom_y: f32,
}

impl Default for GanttSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_labels: true,
            row_height: 40.0,
            bar_height: 24.0,
            x_label: String::from("Time Scale"),
        }
    }
}

impl ModuleWidgetWindowView for GanttChartWidget {
    fn get_window(&self) -> &ModuleWidgetWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow {
        &mut self.window
    }

    fn draw(&self, frame: &mut canvas::Frame) {
        // Call the standalone draw implementation
        draw_gantt_impl(frame, self, self.window.position, self.window.size);
    }

    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView> {
        Box::new(GanttChartWidget {
            window: ModuleWidgetWindow {
                position: self.window.position,
                initial_position: self.window.initial_position,
                size: self.window.size,
                border_color: self.window.border_color,
                border_width: self.window.border_width,
                bg_color: self.window.bg_color,
                title: self.window.title.clone(),
                subtitle: self.window.subtitle.clone(),
            },
            settings: self.settings.clone(),
            datas: self.datas.clone(),
            dark_mode: self.dark_mode,
            pan_offset_x: self.pan_offset_x,
            pan_offset_y: self.pan_offset_y,
            zoom_x: self.zoom_x,
            zoom_y: self.zoom_y,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn add_new_data_item(&mut self, data: &WidgetData) {
        match data {
            WidgetData::GanttStart(start_data) => {
                // Look for an existing incomplete entry with the same label
                if let Some(existing) = self.datas.iter_mut()
                    .find(|d| !d.constructed && d.label == start_data.label) {
                    // Update existing entry with start time
                    existing.start_time = start_data.start_time;
                    if existing.end_time != 0.0 {
                        existing.constructed = true;
                    }
                } else {
                    // Create new entry with just start time
                    let gantt_data = GanttDataPoint {
                        constructed: false,
                        start_time: start_data.start_time,
                        end_time: 0.0,
                        label: start_data.label.clone(),
                    };
                    self.datas.push(gantt_data);
                }
            },
            WidgetData::GanttEnd(end_data) => {
                // Look for an existing incomplete entry with the same label
                if let Some(existing) = self.datas.iter_mut()
                    .find(|d| !d.constructed && d.label == end_data.label) {
                    // Update existing entry with end time
                    existing.end_time = end_data.end_time;
                    if existing.start_time != 0.0 {
                        existing.constructed = true;
                    }
                } else {
                    // Create new entry with just end time
                    let gantt_data = GanttDataPoint {
                        constructed: false,
                        start_time: 0.0,
                        end_time: end_data.end_time,
                        label: end_data.label.clone(),
                    };
                    self.datas.push(gantt_data);
                }
            },
            _ => {} // Ignore non-Gantt data
        }
    }
}

impl GanttChartWidget {
    pub fn new(dark_mode: bool, settings: GanttSettings) -> Self {
        Self {
            dark_mode,
            settings,
            datas: Vec::new(),
            window: ModuleWidgetWindow::default(),
            pan_offset_x: 0.0,
            pan_offset_y: 0.0,
            zoom_x: 1.0,
            zoom_y: 1.0,
        }
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        self.pan_offset_x += delta_x;
        self.pan_offset_y += delta_y;
    }

    pub fn zoom(&mut self, delta: f32, zoom_x: bool, zoom_y: bool) {
        let zoom_factor = if delta > 0.0 { 1.1 } else { 0.9 };

        if zoom_x {
            self.zoom_x *= zoom_factor;
            self.zoom_x = self.zoom_x.clamp(0.1, 20.0);
        }
        if zoom_y {
            self.zoom_y *= zoom_factor;
            self.zoom_y = self.zoom_y.clamp(0.1, 5.0);
        }
    }

    pub fn reset_view(&mut self) {
        self.pan_offset_x = 0.0;
        self.pan_offset_y = 0.0;
        self.zoom_x = 1.0;
        self.zoom_y = 1.0;
    }

    fn draw_gantt_chart(
        &self,
        frame: &mut canvas::Frame,
        area: &Rectangle,
        data: &[GanttDataPoint],
    ) {
        if data.is_empty() {
            return;
        }

        let filtered_data: Vec<&GanttDataPoint> = data
        .iter()
        .filter(|d| d.constructed)
        .collect();

        if filtered_data.is_empty() {
            return;
        }

        let label_width = if self.settings.show_labels { 120.0 } else { 0.0 };
        let padding = 20.0;
        
        let chart_width = area.width - label_width - 2.0 * padding;
        let chart_x_start = area.x + label_width + padding;
        
        // 1. Calculate Time Range (X-Axis)
        let min_start = filtered_data.iter().map(|d| d.start_time).fold(f32::MAX, f32::min);
        let max_end = filtered_data.iter().map(|d| d.end_time).fold(0.0f32, f32::max);
        
        let raw_range = (max_end - min_start).max(1.0);
        let x_range = raw_range * 1.1; 
        let visible_range = x_range / self.zoom_x;
        
        let center_time = min_start + (raw_range / 2.0) - self.pan_offset_x;
        let min_visible_time = center_time - visible_range / 2.0;
        let max_visible_time = center_time + visible_range / 2.0;

        // 2. Define Colors
        let axis_color = Color::from_rgb(0.7, 0.7, 0.7);
        let grid_color = if self.dark_mode {
            Color::from_rgba(0.4, 0.4, 0.4, 0.3)
        } else {
            Color::from_rgba(0.7, 0.7, 0.7, 0.3)
        };
        let text_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };
        let bar_color = Color::from_rgb(0.2, 0.6, 0.8);
        let border_color = Color::from_rgba(0.0, 0.0, 0.0, 0.3);

        // 3. Draw X-Axis (Time)
        let axis_y = area.y + padding;
        
        frame.stroke(
            &canvas::Path::line(
                Point::new(chart_x_start, axis_y),
                Point::new(chart_x_start + chart_width, axis_y),
            ),
            canvas::Stroke::default().with_color(axis_color).with_width(2.0),
        );

        let num_labels = 5;
        for i in 0..=num_labels {
            let ratio = i as f32 / num_labels as f32;
            let time_val = min_visible_time + ratio * (max_visible_time - min_visible_time);
            let x_pos = chart_x_start + ratio * chart_width;

            frame.fill_text(canvas::Text {
                content: format!("{:.1}", time_val),
                position: Point::new(x_pos - 10.0, axis_y - 15.0),
                color: text_color,
                size: 12.0.into(),
                ..canvas::Text::default()
            });

            if self.settings.show_grid {
                frame.stroke(
                    &canvas::Path::line(
                        Point::new(x_pos, axis_y),
                        Point::new(x_pos, area.y + area.height - padding),
                    ),
                    canvas::Stroke::default().with_color(grid_color).with_width(1.0),
                );
            }
        }

        // 4. Draw Tasks (Bars)
        let current_row_height = self.settings.row_height * self.zoom_y;
        let current_bar_height = (self.settings.bar_height * self.zoom_y).max(2.0);
        
        // Define the area where bars are allowed to be drawn (The Clipping Rectangle)
        let view_rect = Rectangle {
            x: chart_x_start, // Start after the labels
            y: axis_y + 1.0,  // Start below the axis
            width: chart_width,
            height: area.height - padding * 2.0,
        };

        for (i, task) in filtered_data.iter().enumerate() {
            let y_base = axis_y + 20.0;
            let y_pos = y_base + (i as f32 * current_row_height) + self.pan_offset_y;

            // Draw Label (Left side) - Labels are drawn outside the chart clip area
            // Only draw label if the row is roughly vertically visible
            if self.settings.show_labels 
                && y_pos + current_row_height > area.y 
                && y_pos < area.y + area.height 
            {
                frame.fill_text(canvas::Text {
                    content: task.label.clone(),
                    position: Point::new(area.x + padding, y_pos + (current_row_height - 12.0) / 2.0),
                    color: text_color,
                    size: 13.0.into(),
                    ..canvas::Text::default()
                });
            }

            // Calculate Bar Geometry
            let range_len = max_visible_time - min_visible_time;
            if range_len <= 0.0 { continue; }

            let start_ratio = (task.start_time - min_visible_time) / range_len;
            let end_ratio = (task.end_time - min_visible_time) / range_len;

            // Skip if completely out of horizontal view
            if end_ratio < 0.0 || start_ratio > 1.0 {
                continue;
            }

            let bar_x = chart_x_start + start_ratio * chart_width;
            let bar_w = (end_ratio - start_ratio) * chart_width;
            let bar_y = y_pos + (current_row_height - current_bar_height) / 2.0;

            // Create the ideal bar rectangle
            let bar_rect = Rectangle::new(
                Point::new(bar_x, bar_y), 
                Size::new(bar_w, current_bar_height)
            );

            // FIX: Use intersection to simulate clipping
            // This returns None if there is no overlap, or the visible sub-rectangle if there is.
            if let Some(visible_bar) = bar_rect.intersection(&view_rect) {
                // Fill the visible part
                frame.fill_rectangle(
                    visible_bar.position(), 
                    visible_bar.size(), 
                    bar_color
                );
                
                // Stroke the visible part
                let bar_path = canvas::Path::rectangle(
                    visible_bar.position(), 
                    visible_bar.size()
                );
                frame.stroke(
                    &bar_path, 
                    canvas::Stroke::default()
                        .with_color(border_color)
                        .with_width(1.0)
                );
            }
        }
    }
}

// Standalone function for drawing chart (Matches ChartWidget pattern)
fn draw_gantt_impl(
    frame: &mut canvas::Frame,
    gantt_widget: &GanttChartWidget,
    position: Point,
    size: Size
) {
    // Ensure chart stays within bounds
    let safe_width = size.width.max(MIN_CHART_WIDTH);
    let safe_height = size.height.max(MIN_CHART_HEIGHT);

    let corner_radius = 8.0;
    let elevation_offset = 2.0;

    // Draw elevation shadow
    let shadow_color = if gantt_widget.dark_mode {
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
    let chart_bg = if gantt_widget.dark_mode {
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
    let title_color = if gantt_widget.dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    };

    frame.fill_text(canvas::Text {
        content: gantt_widget.window.title.clone(),
        position: Point::new(
            position.x + 15.0,
            position.y + 15.0,
        ),
        color: title_color,
        size: 20.0.into(),
        ..canvas::Text::default()
    });

    // Draw subtitle
    let subtitle_color = if gantt_widget.dark_mode {
        Color::from_rgb(0.8, 0.8, 0.8)
    } else {
        Color::from_rgb(0.2, 0.2, 0.2)
    };

    frame.fill_text(canvas::Text {
        content: gantt_widget.window.subtitle.clone(),
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

    // Define the inner chart area
    let chart_area = Rectangle {
        x: position.x + 15.0,
        y: position.y + header_height,
        width: safe_width - 30.0,
        height: available_height,
    };

    // Delegate to the specific Gantt drawing logic
    gantt_widget.draw_gantt_chart(frame, &chart_area, &gantt_widget.datas);
}