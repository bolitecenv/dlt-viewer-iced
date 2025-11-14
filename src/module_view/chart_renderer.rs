use crate::module_view::module_widget::{ChartData, ChartWidget, MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidget, RESIZE_HANDLE_SIZE};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

pub struct ChartRenderer {
    pub dark_mode: bool,
}

impl ChartRenderer {
    pub fn new(dark_mode: bool) -> Self {
        Self { dark_mode }
    }

    pub fn is_on_resize_handle(&self, module_widget: &ModuleWidget, position: Point) -> bool {
        let handle_x = module_widget.position.x + module_widget.size.width - RESIZE_HANDLE_SIZE;
        let handle_y = module_widget.position.y + module_widget.size.height - RESIZE_HANDLE_SIZE;
        
        position.x >= handle_x && position.x <= handle_x + RESIZE_HANDLE_SIZE &&
        position.y >= handle_y && position.y <= handle_y + RESIZE_HANDLE_SIZE
    }

    pub fn draw_chart(
        &self,
        frame: &mut canvas::Frame,
        module_widget: &ModuleWidget,
        cursor_position: Option<Point>,
    ) {
        // Extract chart widget if it's a LineChart
        let chart_widget = match &module_widget.widget_type {
            crate::module_view::module_widget::WidgetTpye::LineChart(chart) => chart,
            _ => return, // Skip if not a line chart
        };

        // Ensure chart stays within bounds
        let safe_width = module_widget.size.width.max(MIN_CHART_WIDTH);
        let safe_height = module_widget.size.height.max(MIN_CHART_HEIGHT);
        
        // Draw chart background
        let chart_bg = if self.dark_mode {
            Color::from_rgba(0.2, 0.2, 0.25, 0.95)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.95)
        };
        frame.fill_rectangle(module_widget.position, Size::new(safe_width, safe_height), chart_bg);

        // Draw border with chart color
        let border = canvas::Path::rectangle(module_widget.position, Size::new(safe_width, safe_height));
        frame.stroke(
            &border,
            canvas::Stroke::default()
                .with_color(module_widget.settings.color)
                .with_width(2.0),
        );

        // Draw title (from settings or default)
        let title_color = if self.dark_mode {
            Color::WHITE
        } else {
            Color::BLACK
        };
        
        let title_text = if module_widget.settings.show_title {
            &module_widget.settings.title
        } else {
            "Analytics"
        };
        
        frame.fill_text(canvas::Text {
            content: title_text.to_string(),
            position: Point::new(module_widget.position.x + 15.0, module_widget.position.y + 15.0),
            color: title_color,
            size: 20.0.into(),
            ..canvas::Text::default()
        });

        // Draw Sales Performance subtitle
        let subtitle_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.2, 0.2, 0.2)
        };
        
        frame.fill_text(canvas::Text {
            content: "Sales Performance".to_string(),
            position: Point::new(module_widget.position.x + 15.0, module_widget.position.y + 42.0),
            color: subtitle_color,
            size: 14.0.into(),
            ..canvas::Text::default()
        });

        // Calculate available space for chart, leaving room for title and padding
        let header_height = 65.0; // Space for title and subtitle
        let bottom_padding = 15.0; // Padding at the bottom
        let available_height = safe_height - header_height - bottom_padding;
        
        // Draw the line chart
        let chart_area = Rectangle {
            x: module_widget.position.x + 15.0,
            y: module_widget.position.y + header_height,
            width: safe_width - 30.0,
            height: available_height,
        };
        
        self.draw_line_chart(frame, &chart_area, &chart_widget.chart_data, module_widget, chart_widget);

        // Draw resize handle in bottom-right corner
        let handle_x = module_widget.position.x + safe_width - RESIZE_HANDLE_SIZE;
        let handle_y = module_widget.position.y + safe_height - RESIZE_HANDLE_SIZE;
        let handle_position = Point::new(handle_x, handle_y);
        
        // Determine handle color based on hover state
        let is_hovering = cursor_position
            .map(|pos| self.is_on_resize_handle(module_widget, pos))
            .unwrap_or(false);
        
        let handle_color = if is_hovering {
            module_widget.settings.color
        } else {
            Color::from_rgba(0.5, 0.5, 0.5, 0.6)
        };
        
        frame.fill_rectangle(
            handle_position,
            Size::new(RESIZE_HANDLE_SIZE, RESIZE_HANDLE_SIZE),
            handle_color,
        );
    }

    pub fn draw_line_chart(
        &self,
        frame: &mut canvas::Frame,
        area: &Rectangle,
        chart_data: &[ChartData],
        module_widget: &ModuleWidget,
        chart_widget: &ChartWidget,
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
                Point::new(area.x + area.width - padding, area.y + area.height - padding),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );

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

        // Apply Y-axis zoom and pan (from ModuleWidgetCommonSettings)
        let y_range_zoomed = y_range / module_widget.settings.y_zoom;
        let y_center = min_y + y_range * 0.5 + module_widget.settings.y_offset * y_range;
        let y_min_visible = y_center - y_range_zoomed * 0.5;
        let y_max_visible = y_center + y_range_zoomed * 0.5;

        // Draw Y-axis labels
        let label_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };
        
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

        // Draw Y-axis label (rotated 90 degrees counter-clockwise)
        if !chart_widget.settings.y_label.is_empty() {
            // Save the current transformation state
            frame.with_save(|frame| {
                // Move to the position where we want the rotated text
                // Position it to the left of the Y-axis tick labels (which are at padding - 35.0)
                let label_x = area.x - 15.0;
                let label_y = area.y + padding + chart_height / 2.0;
                
                // Translate to the rotation point
                frame.translate(iced::Vector::new(label_x, label_y));
                
                // Rotate -90 degrees (counter-clockwise)
                frame.rotate(-std::f32::consts::FRAC_PI_2);
                
                // Draw the text at the origin (which is now rotated)
                // Offset slightly to center it better
                frame.fill_text(canvas::Text {
                    content: chart_widget.settings.y_label.clone(),
                    position: Point::new(-20.0, 0.0),
                    color: label_color,
                    size: 14.0.into(),
                    ..canvas::Text::default()
                });
            });
        }

        // Apply X-axis zoom and pan (from ModuleWidgetCommonSettings)
        let x_range_zoomed = x_range / module_widget.settings.x_zoom;
        let x_center = min_x + x_range * 0.5 + module_widget.settings.x_offset * x_range;
        let x_min_visible = x_center - x_range_zoomed * 0.5;
        let x_max_visible = x_center + x_range_zoomed * 0.5;

        // Draw X-axis labels
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

        // Draw X-axis label (centered below the axis)
        if !chart_widget.settings.x_label.is_empty() {
            frame.fill_text(canvas::Text {
                content: chart_widget.settings.x_label.clone(),
                position: Point::new(area.x + padding + chart_width / 2.0 - 20.0, area.y + area.height - 5.0),
                color: label_color,
                size: 14.0.into(),
                ..canvas::Text::default()
            });
        }

        // Build the line path with zoom and pan applied
        let mut path_builder = canvas::path::Builder::new();
        let mut first_point = true;
        
        for data_point in chart_data.iter() {
            let x_val = data_point.x_value;
            let y_val = data_point.y_value;
            
            // Skip points outside visible range
            if x_val < x_min_visible || x_val > x_max_visible {
                first_point = true; // Reset for next visible segment
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