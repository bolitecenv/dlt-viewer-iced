use crate::module_view::ganttchart_renderer::GanttChartRenderer;
use crate::module_view::module_widget::WidgetTpye;
use crate::module_view::{ModuleWidget, chart_renderer};
use crate::module_view::chart_renderer::ChartRenderer;
use crate::message::Message;
use iced::widget::canvas::{self, Canvas};
use iced::{mouse, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, keyboard};
use std::collections::HashMap;

pub struct ModuleCanvas {
    pub module_widget: HashMap<usize, ModuleWidget>,
    pub dark_mode: bool,
    pub context_menu: Option<ContextMenu>,
}

pub struct DragState {
    pub chart_id: usize,
    pub offset: Point,
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub position: Point,
    pub items: Vec<ContextMenuItem>,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub angle_start: f32,
    pub angle_end: f32,
    pub action: ContextMenuAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextMenuAction {
    AddChart,
    AddGanttChart,
    Delete,
    Duplicate,
    Settings,
}

#[derive(Default)]
pub struct InteractionState {
    pub dragging_chart_index: Option<usize>,
    pub drag_start: Option<Point>,
}

// Handle mouse wheel events
pub fn handle_mouse_wheel(
    module_widget: &mut ModuleWidget,
    delta: f32,
    shift_pressed: bool,
) {
    const ZOOM_FACTOR: f32 = 1.2;
    const MIN_ZOOM: f32 = 0.1;
    const MAX_ZOOM: f32 = 10.0;
    
    if delta.abs() < f32::EPSILON {
        return; // Ignore very small deltas
    }
    
    if shift_pressed {
        // Shift + wheel: zoom Y-axis
        let new_zoom = if delta > 0.0 {
            module_widget.settings.y_zoom * ZOOM_FACTOR
        } else {
            module_widget.settings.y_zoom / ZOOM_FACTOR
        };
        module_widget.settings.y_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    } else {
        // Wheel only: zoom X-axis
        let new_zoom = if delta > 0.0 {
            module_widget.settings.x_zoom * ZOOM_FACTOR
        } else {
            module_widget.settings.x_zoom / ZOOM_FACTOR
        };
        module_widget.settings.x_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

// Handle mouse drag for panning
pub fn handle_mouse_drag(
    module_widgets: &mut ModuleWidget,
    delta: Point,
    shift_pressed: bool,
    chart_area: &Rectangle,
) {
    if shift_pressed {
        // Shift + drag: pan X-axis
        let pan_sensitivity = 1.0 / (chart_area.width * module_widgets.settings.x_zoom);
        module_widgets.settings.x_offset -= delta.x * pan_sensitivity;
        module_widgets.settings.x_offset = module_widgets.settings.x_offset.clamp(-0.5, 0.5);
    }
}

pub fn is_point_in_chart(point: Point, chart: &ModuleWidget) -> bool {
    point.x >= chart.position.x
        && point.x <= chart.position.x + chart.size.width
        && point.y >= chart.position.y
        && point.y <= chart.position.y + chart.size.height
}

impl ContextMenu {
    pub fn new(position: Point) -> Self {
        let mut items = vec![
            ContextMenuItem {
                label: "Add Chart".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: ContextMenuAction::AddChart,
            },
            ContextMenuItem {
                label: "Add Gantt Chart".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: ContextMenuAction::AddGanttChart,
            },
            ContextMenuItem {
                label: "Delete".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: ContextMenuAction::Delete,
            },
            ContextMenuItem {
                label: "Duplicate".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: ContextMenuAction::Duplicate,
            },
            ContextMenuItem {
                label: "Settings".to_string(),
                angle_start: 0.0,
                angle_end: 0.0,
                action: ContextMenuAction::Settings,
            },
        ];

        // Automatically calculate angles
        let item_count = items.len();
        let angle_per_item = 2.0 * std::f32::consts::PI / item_count as f32;

        for (i, item) in items.iter_mut().enumerate() {
            item.angle_start = i as f32 * angle_per_item;
            item.angle_end = (i + 1) as f32 * angle_per_item;
        }
                
        Self { position, items }
    }
    
    pub fn get_action_at(&self, point: Point, radius: f32) -> Option<ContextMenuAction> {
        let dx = point.x - self.position.x;
        let dy = point.y - self.position.y;
        
        let angle = dy.atan2(dx);
        let normalized_angle = if angle < 0.0 {
            angle + 2.0 * std::f32::consts::PI
        } else {
            angle
        };
        
        for item in &self.items {
            if normalized_angle >= item.angle_start && normalized_angle < item.angle_end {
                return Some(item.action);
            }
        }
        
        None
    }
}

impl canvas::Program<Message> for ModuleCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw background
        let bg_color = if self.dark_mode {
            Color::from_rgb(0.1, 0.1, 0.12)
        } else {
            Color::from_rgb(0.99, 0.99, 0.99)
        };
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg_color);

        // Draw grid
        //self.draw_grid(&mut frame, bounds);

        // Draw each chart using the renderer
        let cursor_position = cursor.position_in(bounds);
        let chart_renderer = ChartRenderer::new(self.dark_mode);
        let gantt_chart_renderer = GanttChartRenderer::new(self.dark_mode);
        
        for widget in self.module_widget.values() {
            match &widget.widget_type {
                WidgetTpye::LineChart(chart_widget) | WidgetTpye::BarChart(chart_widget) => {
                    chart_renderer.draw_chart(
                        &mut frame,
                        &widget,
                        cursor_position,
                    );
                }
                WidgetTpye::GanttChart(gantt_chart_widget) => {
                    gantt_chart_renderer.draw_chart(
                        &mut frame,
                        &widget,
                        cursor_position,
                    );
                }
                _ => {}
            }
        }

        // Draw context menu if present
        if let Some(menu) = &self.context_menu {
            draw_context_menu(&mut frame, menu, cursor_position, self.dark_mode);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let cursor_position = cursor.position_in(bounds);

        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor_position {
                        let renderer = ChartRenderer::new(self.dark_mode);
                        
                        // Check if clicking on a resize handle
                        for chart in self.module_widget.values() {
                            if renderer.is_on_resize_handle(chart, position) {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(Message::StartResize(chart.id, position)),
                                );
                            }
                        }
                        
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::MousePressed(position)),
                        );
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    if let Some(position) = cursor_position {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ShowContextMenu(position)),
                        );
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    return (canvas::event::Status::Captured, Some(Message::MouseReleased));
                }
                mouse::Event::ButtonReleased(mouse::Button::Right) => {
                    if let Some(position) = cursor_position {
                        return (
                            canvas::event::Status::Captured, 
                            Some(Message::RightMouseReleased(position))
                        );
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor_position {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::MouseMoved(position)),
                        );
                    }
                }

                mouse::Event::WheelScrolled { delta } => {
                    if let Some(cursor_position) = cursor_position {
                        // Check if cursor is over a chart
                        for (chart_id, chart) in &self.module_widget {
                            if is_point_in_chart(cursor_position, chart) {
                                let scroll_delta = match delta {
                                    mouse::ScrollDelta::Lines { y, .. } => y,
                                    mouse::ScrollDelta::Pixels { y, .. } => y,
                                };
                                
                                // Return a message instead of mutating directly
                                return (
                                    canvas::event::Status::Captured,
                                    Some(Message::MouseWheel(*chart_id, scroll_delta))
                                );
                            }
                        }
                    }
                }
                _ => {}
            },
            canvas::Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::ModifiersChanged(modifiers) => {
                    let shift_pressed = modifiers.control();
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::ShiftKeyChanged(shift_pressed)),
                    );
                }
                _ => {}
            }

            _ => {}
         }
        (canvas::event::Status::Ignored, None)
    }
}

impl ModuleCanvas {
    fn draw_grid(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let grid_size = 50.0;
        let grid_color = if self.dark_mode {
            Color::from_rgba(1.0, 1.0, 1.0, 0.05)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.1)
        };
        
        let grid_rows = (bounds.height / grid_size).ceil() as usize;
        let grid_cols = (bounds.width / grid_size).ceil() as usize;
        
        for i in 0..=grid_rows {
            let y = i as f32 * grid_size;
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
                canvas::Stroke::default().with_color(grid_color).with_width(1.0),
            );
        }
        for i in 0..=grid_cols {
            let x = i as f32 * grid_size;
            frame.stroke(
                &canvas::Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                canvas::Stroke::default().with_color(grid_color).with_width(1.0),
            );
        }
    }
}

// Helper function to draw the context menu
pub fn draw_context_menu(
    frame: &mut canvas::Frame,
    menu: &ContextMenu,
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
    frame.fill(&canvas::Path::circle(menu.position, radius + 5.0), backdrop_color);
    
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
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            font: iced::Font::with_name("SF Pro Display"),
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

pub fn view(module_widget: HashMap<usize, ModuleWidget>, dark_mode: bool, context_menu: Option<ContextMenu>) -> Element<'static, Message> {
    Canvas::new(ModuleCanvas { module_widget, dark_mode, context_menu })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}