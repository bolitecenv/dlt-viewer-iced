use iced::Alignment::Center;
use iced::widget::canvas;
use iced::{Color, Point};

#[derive(Debug, Clone)]
pub struct CircularContextMenu {
    pub position: Point,
    pub items: Vec<CircularContextMenuItem>,
    pub target_module: Option<usize>,  // Which module this menu is for
}


#[derive(Debug, Clone)]
pub struct CircularContextMenuItem {
    pub label: String,
    pub angle_start: f32,
    pub angle_end: f32,
    pub action: CircularContextMenuAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircularContextMenuAction {
    AddChart,
    AddGanttChart,
    Delete,
    Duplicate,
    Settings,
}

impl CircularContextMenu {
    pub fn new(position: Point, target_module: Option<usize>) -> Self {
        let mut items = vec![
            CircularContextMenuItem {
                label: "Add Chart".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: CircularContextMenuAction::AddChart,
            },
            CircularContextMenuItem {
                label: "Add Gantt Chart".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: CircularContextMenuAction::AddGanttChart,
            },
            CircularContextMenuItem {
                label: "Delete".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: CircularContextMenuAction::Delete,
            },
            CircularContextMenuItem {
                label: "Duplicate".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: CircularContextMenuAction::Duplicate,
            },
            CircularContextMenuItem {
                label: "Settings".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: CircularContextMenuAction::Settings,
            },
        ];

        // Automatically calculate angles
        let item_count = items.len();
        let angle_per_item = 2.0 * std::f32::consts::PI / item_count as f32;

        for (i, item) in items.iter_mut().enumerate() {
            item.angle_start = i as f32 * angle_per_item;
            item.angle_end = (i + 1) as f32 * angle_per_item;
        }

        Self { position, items, target_module }
    }

    pub fn get_action_at(&self, point: Point, _radius: f32) -> Option<CircularContextMenuAction> {
        let dx = point.x - self.position.x;
        let dy = point.y - self.position.y;

        let angle = dy.atan2(dx);
        let normalized_angle = if angle < 0.0 {
            angle + 2.0 * std::f32::consts::PI
        } else {
            angle
        };

        let distance = (dx * dx + dy * dy).sqrt();
        if distance < 20.0 {
            return None;
        }

        for item in &self.items {
            if normalized_angle >= item.angle_start && normalized_angle < item.angle_end {
                return Some(item.action);
            }
        }

        None
    }
}


pub fn draw_circular_context_menu(
    frame: &mut canvas::Frame,
    menu: &CircularContextMenu,
    cursor_position: Option<Point>,
    dark_mode: bool,
) {
    let radius = 90.0;
    let inner_radius = 30.0;

    // Determine which slice is hovered
    let hovered_action = cursor_position.and_then(|pos| menu.get_action_at(pos, radius));

    // Draw subtle backdrop circle for depth
    let backdrop_color = if dark_mode {
        Color::from_rgba(0.0, 0.0, 0.0, 0.3)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.15)
    };
    frame.fill(
        &canvas::Path::circle(menu.position, radius + 5.0),
        backdrop_color,
    );

    // Draw each menu slice with radial gradient transparency
    for item in &menu.items {
        let is_hovered = hovered_action == Some(item.action);

        // Base colors without transparency (we'll add it in the gradient)
        let base_rgb = if dark_mode {
            (0.2, 0.2, 0.25)
        } else {
            (1.0, 1.0, 1.0)
        };

        let hover_rgb = if dark_mode {
            (0.35, 0.5, 0.7)
        } else {
            (0.6, 0.75, 0.95)
        };

        let (r, g, b) = if is_hovered { hover_rgb } else { base_rgb };

        // Draw multiple layers with increasing transparency to create gradient effect
        let radial_layers = 20;
        let segments = 30;

        for layer in 0..radial_layers {
            let t_inner = layer as f32 / radial_layers as f32;
            let t_outer = (layer + 1) as f32 / radial_layers as f32;

            // Calculate radius for this layer
            let layer_inner_radius = inner_radius + (radius - inner_radius) * t_inner;
            let layer_outer_radius = inner_radius + (radius - inner_radius) * t_outer;

            // Calculate opacity: 1.0 at center (inner_radius), 0.0 at edge (radius)
            // Use average opacity for this layer
            let opacity_inner = 1.0 - t_inner;
            let opacity_outer = 1.0 - t_outer;
            let layer_opacity = (opacity_inner + opacity_outer) / 2.0;

            let layer_color = Color::from_rgba(r, g, b, layer_opacity);

            // Build the path for this layer
            let mut path_builder = canvas::path::Builder::new();

            // Outer arc
            for i in 0..=segments {
                let seg_t = i as f32 / segments as f32;
                let angle = item.angle_start + (item.angle_end - item.angle_start) * seg_t;

                let x = menu.position.x + layer_outer_radius * angle.cos();
                let y = menu.position.y + layer_outer_radius * angle.sin();

                if i == 0 {
                    path_builder.move_to(Point::new(x, y));
                } else {
                    path_builder.line_to(Point::new(x, y));
                }
            }

            // Inner arc (reverse direction)
            for i in (0..=segments).rev() {
                let seg_t = i as f32 / segments as f32;
                let angle = item.angle_start + (item.angle_end - item.angle_start) * seg_t;

                let x = menu.position.x + layer_inner_radius * angle.cos();
                let y = menu.position.y + layer_inner_radius * angle.sin();

                path_builder.line_to(Point::new(x, y));
            }

            path_builder.close();
            let path = path_builder.build();

            frame.fill(&path, layer_color);
        }

        // Draw border only on the outermost edge
        let mut border_builder = canvas::path::Builder::new();

        // Outer arc for border
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = item.angle_start + (item.angle_end - item.angle_start) * t;

            let x = menu.position.x + radius * angle.cos();
            let y = menu.position.y + radius * angle.sin();

            if i == 0 {
                border_builder.move_to(Point::new(x, y));
            } else {
                border_builder.line_to(Point::new(x, y));
            }
        }

        let border_path = border_builder.build();
        frame.stroke(
            &border_path,
            canvas::Stroke::default()
                .with_color(if dark_mode {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.1)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                })
                .with_width(1.0),
        );

        // Draw icon/text label with better positioning
        let mid_angle = (item.angle_start + item.angle_end) / 2.0;
        let text_radius = (radius + inner_radius) / 2.0;
        let text_x = menu.position.x + text_radius * mid_angle.cos();
        let text_y = menu.position.y + text_radius * mid_angle.sin();

        // Text with subtle shadow effect
        let text_color = if dark_mode {
            if is_hovered {
                Color::WHITE
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.9)
            }
        } else {
            if is_hovered {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::from_rgb(0.2, 0.2, 0.2)
            }
        };

        frame.fill_text(canvas::Text {
            content: item.label.clone(),
            position: Point::new(text_x, text_y),
            color: text_color,
            size: if is_hovered { 15.0 } else { 13.5 }.into(),
            font: iced::Font::with_name("SF Pro Display"),
            align_x: Center.into(),
            align_y: Center.into(),
            ..canvas::Text::default()
        });
    }

    // Draw modern center circle with gradient-like effect
    let center_outer = if dark_mode {
        Color::from_rgba(0.18, 0.18, 0.22, 0.8)
    } else {
        Color::from_rgba(0.95, 0.95, 0.98, 0.85)
    };

    frame.fill(
        &canvas::Path::circle(menu.position, inner_radius),
        center_outer,
    );

    // Inner highlight for depth
    let center_highlight = if dark_mode {
        Color::from_rgba(0.3, 0.3, 0.35, 0.4)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.7)
    };

    frame.fill(
        &canvas::Path::circle(menu.position, inner_radius * 0.7),
        center_highlight,
    );

    // Subtle border on center
    frame.stroke(
        &canvas::Path::circle(menu.position, inner_radius),
        canvas::Stroke::default()
            .with_color(if dark_mode {
                Color::from_rgba(1.0, 1.0, 1.0, 0.1)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.15)
            })
            .with_width(1.0),
    );
}
