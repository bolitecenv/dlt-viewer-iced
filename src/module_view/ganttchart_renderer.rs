use crate::module_view::module_widget::{
    GanttChartData, MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidget, RESIZE_HANDLE_SIZE, WidgetTpye,
};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

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

pub struct GanttChartRenderer {
    pub dark_mode: bool,
}

impl GanttChartRenderer {
    pub fn new(dark_mode: bool) -> Self {
        Self { dark_mode }
    }

    pub fn is_on_resize_handle(&self, module_widget: &ModuleWidget, position: Point) -> bool {
        let handle_x = module_widget.position.x + module_widget.size.width - RESIZE_HANDLE_SIZE;
        let handle_y = module_widget.position.y + module_widget.size.height - RESIZE_HANDLE_SIZE;

        position.x >= handle_x
            && position.x <= handle_x + RESIZE_HANDLE_SIZE
            && position.y >= handle_y
            && position.y <= handle_y + RESIZE_HANDLE_SIZE
    }

    pub fn draw_chart(
        &self,
        frame: &mut canvas::Frame,
        module_widget: &ModuleWidget,
        cursor_position: Option<Point>,
    ) {
        // Extract Gantt chart widget if it's a GanttChart
        let gantt_widget = match &module_widget.widget_type {
            WidgetTpye::GanttChart(gantt) => gantt,
            _ => return, // Skip if not a Gantt chart
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
        frame.fill_rectangle(
            module_widget.position,
            Size::new(safe_width, safe_height),
            chart_bg,
        );

        // Draw border with chart color
        let border =
            canvas::Path::rectangle(module_widget.position, Size::new(safe_width, safe_height));
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
            "Project Timeline"
        };

        frame.fill_text(canvas::Text {
            content: title_text.to_string(),
            position: Point::new(
                module_widget.position.x + 15.0,
                module_widget.position.y + 15.0,
            ),
            color: title_color,
            size: 20.0.into(),
            ..canvas::Text::default()
        });

        // Draw subtitle
        let subtitle_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.2, 0.2, 0.2)
        };

        frame.fill_text(canvas::Text {
            content: "Task Schedule".to_string(),
            position: Point::new(
                module_widget.position.x + 15.0,
                module_widget.position.y + 42.0,
            ),
            color: subtitle_color,
            size: 14.0.into(),
            ..canvas::Text::default()
        });

        // Calculate available space for chart, leaving room for title and padding
        let header_height = 65.0; // Space for title and subtitle
        let bottom_padding = 15.0; // Padding at the bottom
        let available_height = safe_height - header_height - bottom_padding;

        // Draw the Gantt chart
        let chart_area = Rectangle {
            x: module_widget.position.x + 15.0,
            y: module_widget.position.y + header_height,
            width: safe_width - 30.0,
            height: available_height,
        };

        self.draw_gantt_chart(frame, &chart_area, &gantt_widget.chart_data, module_widget);

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

    pub fn draw_gantt_chart(
        &self,
        frame: &mut canvas::Frame,
        area: &Rectangle,
        chart_data: &GanttChartData,
        module_widget: &ModuleWidget,
    ) {
        if chart_data.data_points.is_empty() {
            return;
        }

        let task_name_width = 150.0;
        let padding = 20.0;
        let row_height = 35.0;
        let bar_height = 20.0;

        // Ensure chart stays within bounds
        let safe_width = module_widget.size.width.max(MIN_CHART_WIDTH);
        let safe_height = module_widget.size.height.max(MIN_CHART_HEIGHT);

        // Draw Gantt chart inner background (within the area parameter, not full widget)
        let chart_bg = if self.dark_mode {
            Color::from_rgba(0.2, 0.2, 0.25, 0.95)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.95)
        };
        frame.fill_rectangle(
            module_widget.position,
            Size::new(safe_width, safe_height),
            chart_bg,
        );

        let chart_width = area.width - task_name_width - 2.0 * padding;
        let chart_x_start = area.x + task_name_width + padding;

        // Find time range across all tasks
        let min_start = chart_data
            .data_points
            .iter()
            .map(|t| t.start_time)
            .fold(f32::MAX, f32::min);
        let max_end = chart_data
            .data_points
            .iter()
            .map(|t| t.end_time)
            .fold(0.0f32, f32::max);
        let time_range = max_end - min_start;

        // Apply X-axis zoom and pan (for time axis)
        let time_range_zoomed = time_range / module_widget.settings.x_zoom;
        let time_center =
            min_start + time_range * 0.5 + module_widget.settings.x_offset * time_range;
        let time_min_visible = time_center - time_range_zoomed * 0.5;
        let time_max_visible = time_center + time_range_zoomed * 0.5;

        // Calculate total content height based on number of tasks and y_zoom
        let base_row_height = row_height / module_widget.settings.y_zoom;
        let total_tasks = chart_data.data_points.len();
        let total_content_height = total_tasks as f32 * base_row_height;
        let visible_height = area.height - padding * 2.0 - 30.0; // Available height for tasks

        // Calculate y_offset for scrolling (clamped to valid range)
        let max_y_offset = if total_content_height > visible_height {
            (total_content_height - visible_height) / total_content_height
        } else {
            0.0
        };
        let y_scroll_offset = (module_widget.settings.y_offset * total_content_height)
            .clamp(0.0, max_y_offset * total_content_height);

        // Draw time axis
        let axis_color = Color::from_rgb(0.7, 0.7, 0.7);
        let axis_y = area.y + padding;
        frame.stroke(
            &canvas::Path::line(
                Point::new(chart_x_start, axis_y),
                Point::new(chart_x_start + chart_width, axis_y),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );

        // Draw time labels
        let label_color = if self.dark_mode {
            Color::from_rgb(0.8, 0.8, 0.8)
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };

        let num_time_labels = 6;
        for i in 0..=num_time_labels {
            let ratio = i as f32 / num_time_labels as f32;
            let time_value = time_min_visible + ratio * (time_max_visible - time_min_visible);
            let x = chart_x_start + ratio * chart_width;

            frame.fill_text(canvas::Text {
                content: format!("{:.1}", time_value),
                position: Point::new(x - 15.0, axis_y - 18.0),
                color: label_color,
                size: 12.0.into(),
                ..canvas::Text::default()
            });

            // Draw vertical grid line
            let grid_color = if self.dark_mode {
                Color::from_rgba(0.4, 0.4, 0.4, 0.3)
            } else {
                Color::from_rgba(0.7, 0.7, 0.7, 0.3)
            };

            frame.stroke(
                &canvas::Path::line(
                    Point::new(x, axis_y),
                    Point::new(x, area.y + area.height - padding),
                ),
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(1.0),
            );
        }

        // Draw tasks with y_zoom and y_scroll applied
        for (idx, task) in chart_data.data_points.iter().enumerate() {
            // Apply y_zoom to row height and adjust for scroll offset
            let y_pos = area.y + padding + 30.0 + (idx as f32 * base_row_height) - y_scroll_offset;

            // Skip if task row is outside visible area (with some margin for partial visibility)
            if y_pos + bar_height < area.y + padding + 30.0
                || y_pos > area.y + area.height - padding
            {
                continue;
            }

            // Draw task name
            frame.fill_text(canvas::Text {
                content: chart_data.data_points[idx].y_label.clone(),
                position: Point::new(area.x + padding, y_pos),
                color: label_color,
                size: 13.0.into(),
                ..canvas::Text::default()
            });

            // Calculate bar position based on time
            let start_ratio = if time_max_visible - time_min_visible > 0.0 {
                ((task.start_time - time_min_visible) / (time_max_visible - time_min_visible))
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };

            let end_ratio = if time_max_visible - time_min_visible > 0.0 {
                ((task.end_time - time_min_visible) / (time_max_visible - time_min_visible))
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };

            // Skip if task is completely outside visible range
            if end_ratio < 0.0 || start_ratio > 1.0 {
                continue;
            }

            // Ensure bar stays within chart boundaries
            let bar_x_start = (chart_x_start + start_ratio * chart_width).max(chart_x_start);
            let bar_x_end =
                (chart_x_start + end_ratio * chart_width).min(chart_x_start + chart_width);
            let bar_width = (bar_x_end - bar_x_start).max(0.0);

            // Adjust bar height based on y_zoom (scale down when zoomed out)
            let scaled_bar_height = (bar_height / module_widget.settings.y_zoom).max(2.0);

            // Only draw if there's visible width
            if bar_width > 0.0 {
                let bar_color = Color::from_rgb(0.2, 0.6, 0.8);
                let bar_position = Point::new(bar_x_start, y_pos - 2.0);

                frame.fill_rectangle(
                    bar_position,
                    Size::new(bar_width.max(2.0), scaled_bar_height),
                    bar_color,
                );

                // Draw bar border
                let bar_rect = canvas::Path::rectangle(
                    bar_position,
                    Size::new(bar_width.max(2.0), scaled_bar_height),
                );
                frame.stroke(
                    &bar_rect,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.3))
                        .with_width(1.0),
                );
            }
        }
    }
}
