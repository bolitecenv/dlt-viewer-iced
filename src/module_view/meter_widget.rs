use crate::module_view::module_widget::{
    MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidget, ModuleWidgetWindow, ModuleWidgetWindowView, WidgetData,
};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};
use std::any::Any;

#[derive(Clone)]
pub struct MeterSettings {
    pub min_value: f32,
    pub max_value: f32,
    pub unit: String,
    pub warning_threshold: Option<f32>,  // Red zone starts here
    pub danger_threshold: Option<f32>,   // Critical red zone
    pub show_digital_readout: bool,
    pub label: String,
}

#[derive(Clone)]
pub struct MeterWidget {
    pub window: ModuleWidgetWindow,
    pub settings: MeterSettings,
    pub current_value: f32,
    pub dark_mode: bool,
}

impl ModuleWidgetWindowView for MeterWidget {
    fn get_window(&self) -> &ModuleWidgetWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut ModuleWidgetWindow {
        &mut self.window
    }

    fn draw(&self, frame: &mut canvas::Frame) {
        draw_meter_impl(frame, self, self.window.position, self.window.size);
    }

    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView> {
        Box::new(MeterWidget {
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
            settings: MeterSettings {
                min_value: self.settings.min_value,
                max_value: self.settings.max_value,
                unit: self.settings.unit.clone(),
                warning_threshold: self.settings.warning_threshold,
                danger_threshold: self.settings.danger_threshold,
                show_digital_readout: self.settings.show_digital_readout,
                label: self.settings.label.clone(),
            },
            current_value: self.current_value,
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
        // if let WidgetData::Meter(value) = data {
        //     self.set_value(*value);
        // }
    }
}

impl MeterWidget {
    pub fn new(dark_mode: bool, settings: MeterSettings) -> Self {
        Self {
            dark_mode,
            settings,
            current_value: 0.0,
            window: ModuleWidgetWindow::default(),
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.current_value = value.clamp(self.settings.min_value, self.settings.max_value);
    }

    fn draw_gauge(
        &self,
        frame: &mut canvas::Frame,
        center: Point,
        radius: f32,
    ) {
        let start_angle = std::f32::consts::PI * 0.75; // Start at 135 degrees
        let end_angle = std::f32::consts::PI * 2.25;   // End at 405 degrees (270 degree arc)
        let total_arc = end_angle - start_angle;

        // Draw outer ring
        let ring_color = if self.dark_mode {
            Color::from_rgba(0.3, 0.3, 0.35, 1.0)
        } else {
            Color::from_rgba(0.85, 0.85, 0.85, 1.0)
        };

        let mut outer_ring = canvas::path::Builder::new();
        let num_segments = 100;
        for i in 0..=num_segments {
            let angle = start_angle + (i as f32 / num_segments as f32) * total_arc;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            
            if i == 0 {
                outer_ring.move_to(Point::new(x, y));
            } else {
                outer_ring.line_to(Point::new(x, y));
            }
        }

        frame.stroke(
            &outer_ring.build(),
            canvas::Stroke::default()
                .with_color(ring_color)
                .with_width(8.0),
        );

        // Draw colored arc segments
        let range = self.settings.max_value - self.settings.min_value;
        let warning_start = self.settings.warning_threshold.unwrap_or(self.settings.max_value);
        let danger_start = self.settings.danger_threshold.unwrap_or(self.settings.max_value);

        // Green zone
        self.draw_arc_segment(
            frame,
            center,
            radius,
            start_angle,
            start_angle + ((warning_start - self.settings.min_value) / range) * total_arc,
            Color::from_rgb(0.2, 0.8, 0.3),
            6.0,
        );

        // Yellow/warning zone
        if self.settings.warning_threshold.is_some() {
            let warning_end = if self.settings.danger_threshold.is_some() {
                danger_start
            } else {
                self.settings.max_value
            };
            
            self.draw_arc_segment(
                frame,
                center,
                radius,
                start_angle + ((warning_start - self.settings.min_value) / range) * total_arc,
                start_angle + ((warning_end - self.settings.min_value) / range) * total_arc,
                Color::from_rgb(1.0, 0.8, 0.0),
                6.0,
            );
        }

        // Red/danger zone
        if self.settings.danger_threshold.is_some() {
            self.draw_arc_segment(
                frame,
                center,
                radius,
                start_angle + ((danger_start - self.settings.min_value) / range) * total_arc,
                end_angle,
                Color::from_rgb(0.9, 0.2, 0.2),
                6.0,
            );
        }

        // Draw tick marks and labels
        let num_major_ticks = 11;
        let num_minor_ticks = 50;

        let tick_color = if self.dark_mode {
            Color::from_rgb(0.7, 0.7, 0.7)
        } else {
            Color::from_rgb(0.4, 0.4, 0.4)
        };

        // Minor ticks
        for i in 0..=num_minor_ticks {
            let angle = start_angle + (i as f32 / num_minor_ticks as f32) * total_arc;
            let inner_radius = radius - 8.0;
            let outer_radius = radius - 12.0;
            
            let x1 = center.x + inner_radius * angle.cos();
            let y1 = center.y + inner_radius * angle.sin();
            let x2 = center.x + outer_radius * angle.cos();
            let y2 = center.y + outer_radius * angle.sin();

            frame.stroke(
                &canvas::Path::line(Point::new(x1, y1), Point::new(x2, y2)),
                canvas::Stroke::default()
                    .with_color(tick_color)
                    .with_width(1.0),
            );
        }

        // Major ticks with labels
        for i in 0..=num_major_ticks {
            let angle = start_angle + (i as f32 / num_major_ticks as f32) * total_arc;
            let value = self.settings.min_value + (i as f32 / num_major_ticks as f32) * range;
            
            let inner_radius = radius - 8.0;
            let outer_radius = radius - 18.0;
            let label_radius = radius - 35.0;
            
            let x1 = center.x + inner_radius * angle.cos();
            let y1 = center.y + inner_radius * angle.sin();
            let x2 = center.x + outer_radius * angle.cos();
            let y2 = center.y + outer_radius * angle.sin();

            frame.stroke(
                &canvas::Path::line(Point::new(x1, y1), Point::new(x2, y2)),
                canvas::Stroke::default()
                    .with_color(tick_color)
                    .with_width(2.5),
            );

            // Draw value labels
            let label_x = center.x + label_radius * angle.cos();
            let label_y = center.y + label_radius * angle.sin();

            frame.fill_text(canvas::Text {
                content: format!("{:.0}", value),
                position: Point::new(label_x - 10.0, label_y - 6.0),
                color: tick_color,
                size: 12.0.into(),
                ..canvas::Text::default()
            });
        }

        // Draw needle
        let value_ratio = (self.current_value - self.settings.min_value) / range;
        let needle_angle = start_angle + value_ratio * total_arc;
        
        let needle_length = radius - 25.0;
        let needle_tip_x = center.x + needle_length * needle_angle.cos();
        let needle_tip_y = center.y + needle_length * needle_angle.sin();

        // Needle base (wider part)
        let base_width = 8.0;
        let perpendicular = needle_angle + std::f32::consts::FRAC_PI_2;
        
        let base1_x = center.x + base_width * perpendicular.cos();
        let base1_y = center.y + base_width * perpendicular.sin();
        let base2_x = center.x - base_width * perpendicular.cos();
        let base2_y = center.y - base_width * perpendicular.sin();

        let needle_color = if self.current_value >= danger_start {
            Color::from_rgb(0.9, 0.2, 0.2)
        } else if self.current_value >= warning_start {
            Color::from_rgb(1.0, 0.8, 0.0)
        } else {
            Color::from_rgb(0.9, 0.9, 0.9)
        };

        let mut needle_path = canvas::path::Builder::new();
        needle_path.move_to(Point::new(base1_x, base1_y));
        needle_path.line_to(Point::new(needle_tip_x, needle_tip_y));
        needle_path.line_to(Point::new(base2_x, base2_y));
        needle_path.close();

        frame.fill(&needle_path.build(), needle_color);

        // Draw center cap
        let cap_radius = 12.0;
        let cap_path = canvas::Path::circle(center, cap_radius);
        
        frame.fill(&cap_path, Color::from_rgb(0.3, 0.3, 0.3));
        frame.stroke(
            &cap_path,
            canvas::Stroke::default()
                .with_color(needle_color)
                .with_width(2.0),
        );

        // Draw digital readout
        if self.settings.show_digital_readout {
            let readout_y = center.y + radius * 0.4;
            
            let readout_bg = if self.dark_mode {
                Color::from_rgba(0.1, 0.1, 0.15, 0.8)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.7)
            };

            let readout_rect = canvas::Path::rounded_rectangle(
                Point::new(center.x - 50.0, readout_y - 15.0),
                Size::new(100.0, 30.0),
                4.0.into(),
            );

            frame.fill(&readout_rect, readout_bg);

            let digital_color = if self.current_value >= danger_start {
                Color::from_rgb(1.0, 0.3, 0.3)
            } else if self.current_value >= warning_start {
                Color::from_rgb(1.0, 0.9, 0.0)
            } else {
                Color::from_rgb(0.2, 1.0, 0.3)
            };

            frame.fill_text(canvas::Text {
                content: format!("{:.0}", self.current_value),
                position: Point::new(center.x - 25.0, readout_y - 10.0),
                color: digital_color,
                size: 24.0.into(),
                ..canvas::Text::default()
            });
        }

        // Draw unit label
        if !self.settings.unit.is_empty() {
            let unit_color = if self.dark_mode {
                Color::from_rgb(0.6, 0.6, 0.6)
            } else {
                Color::from_rgb(0.5, 0.5, 0.5)
            };

            frame.fill_text(canvas::Text {
                content: self.settings.unit.clone(),
                position: Point::new(center.x - 15.0, center.y + radius * 0.6),
                color: unit_color,
                size: 14.0.into(),
                ..canvas::Text::default()
            });
        }

        // Draw meter label
        if !self.settings.label.is_empty() {
            let label_color = if self.dark_mode {
                Color::from_rgb(0.8, 0.8, 0.8)
            } else {
                Color::from_rgb(0.3, 0.3, 0.3)
            };

            frame.fill_text(canvas::Text {
                content: self.settings.label.clone(),
                position: Point::new(center.x - 30.0, center.y - radius * 0.3),
                color: label_color,
                size: 16.0.into(),
                ..canvas::Text::default()
            });
        }
    }

    fn draw_arc_segment(
        &self,
        frame: &mut canvas::Frame,
        center: Point,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: Color,
        width: f32,
    ) {
        let mut arc = canvas::path::Builder::new();
        let segments = 50;
        let angle_range = end_angle - start_angle;
        
        for i in 0..=segments {
            let angle = start_angle + (i as f32 / segments as f32) * angle_range;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            
            if i == 0 {
                arc.move_to(Point::new(x, y));
            } else {
                arc.line_to(Point::new(x, y));
            }
        }

        frame.stroke(
            &arc.build(),
            canvas::Stroke::default()
                .with_color(color)
                .with_width(width),
        );
    }
}

fn draw_meter_impl(
    frame: &mut canvas::Frame,
    meter_widget: &MeterWidget,
    position: Point,
    size: Size,
) {
    let safe_width = size.width.max(MIN_CHART_WIDTH);
    let safe_height = size.height.max(MIN_CHART_HEIGHT);

    let corner_radius = 8.0;
    let elevation_offset = 2.0;

    // Draw elevation shadow
    let shadow_color = if meter_widget.dark_mode {
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
    let meter_bg = if meter_widget.dark_mode {
        Color::from_rgba(0.15, 0.15, 0.2, 0.95)
    } else {
        Color::from_rgba(0.95, 0.95, 0.95, 0.95)
    };

    let bg_path = canvas::Path::rounded_rectangle(
        position,
        Size::new(safe_width, safe_height),
        corner_radius.into(),
    );

    frame.fill(&bg_path, meter_bg);

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
    let title_color = if meter_widget.dark_mode {
        Color::WHITE
    } else {
        Color::BLACK
    };

    frame.fill_text(canvas::Text {
        content: meter_widget.window.title.clone(),
        position: Point::new(position.x + 15.0, position.y + 15.0),
        color: title_color,
        size: 20.0.into(),
        ..canvas::Text::default()
    });

    // Draw subtitle
    let subtitle_color = if meter_widget.dark_mode {
        Color::from_rgb(0.8, 0.8, 0.8)
    } else {
        Color::from_rgb(0.2, 0.2, 0.2)
    };

    frame.fill_text(canvas::Text {
        content: meter_widget.window.subtitle.clone(),
        position: Point::new(position.x + 15.0, position.y + 42.0),
        color: subtitle_color,
        size: 14.0.into(),
        ..canvas::Text::default()
    });

    // Calculate gauge dimensions
    let header_height = 65.0;
    let available_height = safe_height - header_height - 20.0;
    let gauge_radius = (available_height.min(safe_width - 30.0) * 0.45).min(150.0);
    
    let center = Point::new(
        position.x + safe_width / 2.0,
        position.y + header_height + available_height / 2.0,
    );

    meter_widget.draw_gauge(frame, center, gauge_radius);
}