use crate::message::Message;
use crate::module_view::ChartWidget;
use crate::module_view::ModuleWidget;
use crate::module_view::chart_widget::ChartSettings;
use crate::module_view::module_widget::*;
use iced::widget::canvas::{self, Canvas};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme, keyboard, mouse};
use std::collections::HashMap;

pub const GRID_SIZE: f32 = 50.0;
pub const SNAP_THRESHOLD: f32 = 10.0;

#[derive(Clone)]
pub struct ModuleCanvas {
    pub module_widget: HashMap<usize, ModuleWidget>,
    pub dark_mode: bool,
    pub context_menu: Option<ContextMenu>,
    pub selected_module: Option<usize>,  // Track which module is selected
    pub hovered_module: Option<usize>,   // Track which module is hovered
    pub resize_module: Option<(usize, ResizeType)>, // Track which module is being resized
    pub state: CanvasState,
}

#[derive(Debug, Clone)]
pub enum ModuleCanvasMessage {
    AddChart,
    AddGanttChart,
    Delete,
    Duplicate,
    Settings,
    Move(Point),
    Resize,
    ShowContextMenu(Point),
    CloseContextMenu,
    SelectModule(Option<usize>),
    RightMouseReleased(Point),
    RightMousePressed(Point),
    LeftMouseReleased(Point),
    LeftMousePressed(Point),
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub position: Point,
    pub items: Vec<ContextMenuItem>,
    pub target_module: Option<usize>,  // Which module this menu is for
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

#[derive(Clone)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub press_position: Option<Point>,  // Where the button was pressed
}

impl Default for ButtonState {
    fn default() -> Self {
        Self { 
            is_pressed: false,
            press_position: None,
        }
    }
}

#[derive(Clone)]
pub struct CanvasState {
    pub left_mouse_button: ButtonState,
    pub right_mouse_button: ButtonState,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            left_mouse_button: ButtonState::default(),
            right_mouse_button: ButtonState::default(),
        }
    }
}

impl ModuleCanvas {
    pub fn new() -> Self {
        Self {
            module_widget: HashMap::new(),
            dark_mode: false,
            context_menu: None,
            selected_module: None,
            hovered_module: None,
            resize_module: None,
            state: CanvasState::default(),
        }
    }

    pub fn update(&mut self, message: ModuleCanvasMessage) -> Task<Message> {
        match message {
            ModuleCanvasMessage::AddChart => {
                println!("Add Chart action triggered");
                
                // Create new chart at context menu position
                if let Some(menu) = &self.context_menu {
                    let chart_widget = ChartWidget::new(self.dark_mode, ChartSettings {
                        show_grid: true,
                        show_legend: true,
                        line_smoothness: 0.5,
                        x_label: "X-Axis".to_string(),
                        y_label: "Y-Axis".to_string(),
                    });
                    
                    let new_id = self.module_widget.keys().max().unwrap_or(&0) + 1;
                    let module_widget = ModuleWidget {
                        id: new_id,
                        module_widget: Box::new(chart_widget),
                        dlt_data_regex_item: None,
                    };
                    self.module_widget.insert(new_id, module_widget);
                }

                self.context_menu = None;
            }
            ModuleCanvasMessage::AddGanttChart => {
                println!("Add Gantt Chart action triggered");
                self.context_menu = None;
            }
            ModuleCanvasMessage::Delete => {
                println!("Delete action triggered");
                if let Some(menu) = &self.context_menu {
                    if let Some(module_id) = menu.target_module {
                        self.module_widget.remove(&module_id);
                    }
                }
                self.context_menu = None;
            }
            ModuleCanvasMessage::Duplicate => {
                println!("Duplicate action triggered");
                self.context_menu = None;
            }
            ModuleCanvasMessage::Settings => {
                println!("Settings action triggered");
                if let Some(menu) = &self.context_menu {
                    if let Some(module_id) = menu.target_module {
                        println!("Opening settings for module: {}", module_id);
                        // TODO: Open settings panel for this module
                    }
                }
                self.context_menu = None;
            }
            ModuleCanvasMessage::Move(position) => {
                // Update hovered module
                self.hovered_module = self.get_module_at_position(position);
                
                if self.state.left_mouse_button.is_pressed {
                    // Handle moving the selected module
                    if let Some(selected_id) = self.selected_module {
                        if let Some(module) = self.module_widget.get_mut(&selected_id) {
                            let window = module.module_widget.get_window_mut();
                            window.position = Point {
                                x: position.x - window.size.width / 2.0,
                                y: position.y - window.size.height / 2.0,
                            };
                        }
                    }

                    // Handle resizing
                    if let Some((resize_id, resize_type)) = self.resize_module {
                        if let Some(module) = self.module_widget.get_mut(&resize_id) {
                            let window = module.module_widget.get_window_mut();
                            match resize_type {
                                ResizeType::Right => {
                                    window.size.width = (position.x - window.position.x).max(MIN_CHART_WIDTH);
                                }
                                ResizeType::Bottom => {
                                    window.size.height = (position.y - window.position.y).max(MIN_CHART_HEIGHT);
                                }
                                ResizeType::Corner => {
                                    window.size.width = (position.x - window.position.x).max(MIN_CHART_WIDTH);
                                    window.size.height = (position.y - window.position.y).max(MIN_CHART_HEIGHT);
                                }
                                _ => {}
                            }
                            // Snap the window position and size to the grid
                            window.size = Self::sticky_snap_to_grid_size(window.size);
                        }
                    }
                }

                if self.context_menu.is_none() {
                        if self.state.right_mouse_button.is_pressed {
                        // Potentially handle right-drag actions
                        
                        // if the right drag is more than a threashold, open context menu
                        let press_pos = self.state.right_mouse_button.press_position;
                        let drag_distance = press_pos.map_or(0.0, |press_pos| {
                            ((position.x - press_pos.x).powi(2) + (position.y - press_pos.y).powi(2)).sqrt()
                        });
                        if drag_distance > 10.0 {
                            let target_module = self.get_module_at_position(position);
                            self.context_menu = Some(ContextMenu::new(position, target_module));
                        }
                    }
                }
                
            }
            ModuleCanvasMessage::Resize => {
                println!("Resize action triggered");
            }
            ModuleCanvasMessage::ShowContextMenu(position) => {
                println!("Show context menu at position: {:?}", position);
                
                // Determine which module was right-clicked
                let target_module = self.get_module_at_position(position);
                
                self.context_menu = Some(ContextMenu::new(position, target_module));
            }
            ModuleCanvasMessage::CloseContextMenu => {
                self.context_menu = None;
            }
            ModuleCanvasMessage::SelectModule(module_id) => {
                self.selected_module = module_id;
                println!("Selected module: {:?}", module_id);
            }

            // Handle mouse button messages
            ModuleCanvasMessage::RightMouseReleased(_position) => {
                // Handle right mouse button release if needed
                self.state.right_mouse_button.is_pressed = false;

                if let Some(menu) = &self.context_menu {
                    if let Some(action) = menu.get_action_at(_position, 90.0) {
                        let message = match action {
                            ContextMenuAction::AddChart => ModuleCanvasMessage::AddChart,
                            ContextMenuAction::AddGanttChart => ModuleCanvasMessage::AddGanttChart,
                            ContextMenuAction::Delete => ModuleCanvasMessage::Delete,
                            ContextMenuAction::Duplicate => ModuleCanvasMessage::Duplicate,
                            ContextMenuAction::Settings => ModuleCanvasMessage::Settings,
                        };
                        return Task::perform(async {}, move |_| Message::ModuleCanvasMessage(message.clone()));
                    }

                    self.context_menu = None;
                }
            }
            ModuleCanvasMessage::RightMousePressed(_position) => {
                // Handle right mouse button press if needed
                self.state.right_mouse_button.is_pressed = true;
                self.state.right_mouse_button.press_position = Some(_position);
            }
            ModuleCanvasMessage::LeftMouseReleased(_position) => {
                // Handle left mouse button release if needed
                self.state.left_mouse_button.is_pressed = false;
            }
            ModuleCanvasMessage::LeftMousePressed(_position) => {
                self.state.left_mouse_button.is_pressed = true;
                self.state.left_mouse_button.press_position = Some(_position);

                self.selected_module = None;
                self.resize_module = None;

                self.selected_module = self.get_module_at_position(_position);
                if self.selected_module.is_none() {
                    self.resize_module = self.get_module_resize_at_position(_position);
                }

                // Debug resize or select
                if let Some(selected_id) = self.selected_module {
                    println!("Selected module for moving: {}", selected_id);
                }
                if let Some((resize_id, resize_type)) = self.resize_module {
                    println!("Selected module for resizing: {} with type {:?}", resize_id, resize_type);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self, dark_mode: bool) -> Element<'_, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn get_module_at_position(&self, position: Point) -> Option<usize> {
        // Check which module contains this position
        for (id, module) in &self.module_widget {
            if module.module_widget.get_window_contains_point(position) {
                return Some(*id);
            }
        }
        None
    }

    fn get_module_resize_at_position(&self, position: Point) -> Option<(usize, ResizeType)> {
        // Check which module resize handle contains this position
        for (id, module) in &self.module_widget {
            if let Some(resize_type) = module.module_widget.get_window_resize_type_contains_point(position) {
                return Some((*id, resize_type));
            }
        }
        None
    }

    fn sticky_snap_to_grid(value: f32, grid_size: f32, threshold: f32) -> f32 {
        let remainder = value % grid_size;
        if remainder < threshold {
            value - remainder
        } else if remainder > grid_size - threshold {
            value + (grid_size - remainder)
        } else {
            value
        }
    }

    fn sticky_snap_to_grid_size(size: Size) -> Size {
        let snapped_width = Self::sticky_snap_to_grid(size.width, GRID_SIZE, SNAP_THRESHOLD);
        let snapped_height = Self::sticky_snap_to_grid(size.height, GRID_SIZE, SNAP_THRESHOLD);
        Size::new(snapped_width, snapped_height)
    }

    fn draw_grid(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let grid_size = GRID_SIZE;
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
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(1.0),
            );
        }
        for i in 0..=grid_cols {
            let x = i as f32 * grid_size;
            frame.stroke(
                &canvas::Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(1.0),
            );
        }
    }

    fn draw_modules(&self, frame: &mut canvas::Frame) {
        for module in self.module_widget.values() {
            module.module_widget.window_draw(frame);
        }
    }
}

impl ContextMenu {
    pub fn new(position: Point, target_module: Option<usize>) -> Self {
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

        Self { position, items, target_module }
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

impl canvas::Program<Message> for ModuleCanvas {
    type State = CanvasState;

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
        // self.draw_grid(&mut frame, bounds);

        self.draw_modules(&mut frame);

        // Draw each chart using the renderer
        let cursor_position = cursor.position_in(bounds);

        // Draw context menu if present
        if let Some(menu) = &self.context_menu {
            draw_context_menu(&mut frame, menu, cursor_position, self.dark_mode);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let cursor_position = cursor.position_in(bounds);

        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.left_mouse_button.is_pressed = true;
                    return (canvas::event::Status::Captured, 
                        Some(Message::ModuleCanvasMessage(
                        ModuleCanvasMessage::LeftMousePressed(cursor_position.unwrap_or(Point::ORIGIN))
                    )));
                }
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    return (canvas::event::Status::Captured, 
                        Some(Message::ModuleCanvasMessage(
                        ModuleCanvasMessage::RightMousePressed(cursor_position.unwrap_or(Point::ORIGIN))
                    )));
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.left_mouse_button.is_pressed = false;
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::ModuleCanvasMessage(
                            ModuleCanvasMessage::LeftMouseReleased(
                                cursor_position.unwrap_or(Point::ORIGIN)
                            )
                        ),
                    ));
                }
                mouse::Event::ButtonReleased(mouse::Button::Right) => {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::ModuleCanvasMessage(
                            ModuleCanvasMessage::RightMouseReleased(cursor_position.unwrap_or(Point::ORIGIN))
                        )),
                    );            
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor_position {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ModuleCanvasMessage(ModuleCanvasMessage::Move(position))),
                        );
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    // Handle zoom or scroll
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
            },
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let cursor_position = cursor.position_in(bounds);

        if let Some(position) = cursor_position {
            // Show different cursor based on what's being hovered
            if let Some(_) = self.hovered_module {
                if state.left_mouse_button.is_pressed {
                    return mouse::Interaction::Grabbing;
                } else {
                    return mouse::Interaction::Grab;
                }
            }
            
            // Show pointer when hovering context menu
            if let Some(menu) = &self.context_menu {
                if menu.get_action_at(position, 90.0).is_some() {
                    return mouse::Interaction::Pointer;
                }
            }

            if let Some((id, resize_type)) = self.get_module_resize_at_position(position) {
                match resize_type {
                    ResizeType::Left => {
                        // Show left resize cursor
                        return mouse::Interaction::ResizingHorizontally;
                    }
                    ResizeType::Right => {
                        // Show right resize cursor
                        return mouse::Interaction::ResizingHorizontally;
                    }
                    ResizeType::Top => {
                        // Show top resize cursor
                        return mouse::Interaction::ResizingVertically;
                    }
                    ResizeType::Bottom => {
                        // Show bottom resize cursor
                        return mouse::Interaction::ResizingVertically;
                    }
                    ResizeType::Corner => {
                        // Show corner resize cursor
                        return mouse::Interaction::ResizingDiagonallyDown;
                    }
                }
            }
        }

        mouse::Interaction::default()
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
