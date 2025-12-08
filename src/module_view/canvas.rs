use crate::components::dlt_data_manager::DltDataRegexItem;
use crate::message::Message;
use crate::modal_window::confirm_modal_window::ConfirmModal;
use crate::modal_window::modal_window::ModalWindowView;
use crate::module_view::ChartWidget;
use crate::module_view::ModuleWidget;
use crate::module_view::chart_widget::ChartData;
use crate::module_view::chart_widget::ChartSettings;
use crate::module_view::circular_context_menu::CircularContextMenu;
use crate::module_view::circular_context_menu::CircularContextMenuAction;
use crate::module_view::circular_context_menu::CircularContextMenuItem;
use crate::module_view::circular_context_menu::draw_circular_context_menu;
use crate::module_view::context_menu::ContextMenu;
use crate::module_view::context_menu::ContextMenuAction;
use crate::module_view::context_menu::draw_context_menu;
use crate::module_view::meter_widget;
use crate::module_view::meter_widget::MeterSettings;
use crate::module_view::meter_widget::MeterWidget;
use crate::module_view::module_widget;
use crate::module_view::module_widget::*;
use crate::module_view::setting_modals::chart_widget_setting_modal::ChartWidgetModal;
use iced::widget::canvas::{self, Canvas};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme, keyboard, mouse};
use iced_aw::context_menu;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;

pub const GRID_SIZE: f32 = 50.0;
pub const SNAP_THRESHOLD: f32 = 10.0;

#[derive(Clone)]
pub struct ModuleCanvas {
    pub module_widget: HashMap<usize, ModuleWidget>,
    pub dark_mode: bool,
    pub circular_context_menu: Option<CircularContextMenu>,
    pub context_menu: Option<ContextMenu>, // NEW: Context menu state
    pub selected_module: Option<usize>,  // Track which module is selected
    pub hovered_module: Option<usize>,   // Track which module is hovered
    pub resize_module: Option<(usize, ResizeType)>, // Track which module is being resized
    pub panning_chart: Option<usize>,  // Track which chart is being panned
    pub state: CanvasState,
    pub shift_pressed: bool,  // Track Shift key state
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
    MouseWheel(f32, Point),  // New: for zoom
    ResetChartView(usize),   // New: reset chart pan/zoom
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
    pub last_cursor_position: Option<Point>,  // Track last cursor position for panning
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            left_mouse_button: ButtonState::default(),
            right_mouse_button: ButtonState::default(),
            last_cursor_position: None,
        }
    }
}

impl ModuleCanvas {
    pub fn new() -> Self {
        Self {
            module_widget: HashMap::new(),
            dark_mode: false,
            circular_context_menu: None,
            context_menu: None,
            selected_module: None,
            hovered_module: None,
            resize_module: None,
            panning_chart: None,
            state: CanvasState::default(),
            shift_pressed: false,
        }
    }

    pub fn update(&mut self, message: ModuleCanvasMessage, app_view: &mut Option<Box<dyn ModalWindowView>>) -> Task<Message> {
        match message {
            ModuleCanvasMessage::AddChart => {
                println!("Add Chart action triggered");
                
                let mut chart_widget = ChartWidget::new(self.dark_mode, ChartSettings {
                    show_grid: true,
                    show_legend: true,
                    line_smoothness: 0.5,
                    x_label: "X-Axis".to_string(),
                    y_label: "Y-Axis".to_string(),
                });

                // Add random data for testing
                for i in 0..50 {
                    let x = i as f32;
                    let y = (i as f32 * 10.0) + (rand::random::<f32>() * 20.0 - 10.0);
                    chart_widget.datas.push(ChartData { x_value: x, y_value: y });
                }

                let new_id = self.module_widget.keys().max().unwrap_or(&0) + 1;
                let dlt_data_regex_item = DltDataRegexItem {
                    regex: r"X:\s*(?<X>[-+]?[0-9]*\.?[0-9]+).*Y:\s*(?<Y>[-+]?[0-9]*\.?[0-9]+)".to_string(),
                    id: new_id as usize,
                    description: "Default Regex".to_string(),
                };
                let module_widget = ModuleWidget {
                    id: new_id,
                    module_widget: Box::new(chart_widget),
                    dlt_data_regex_item: Some(dlt_data_regex_item),
                };
                self.module_widget.insert(new_id, module_widget);

                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::AddGanttChart => {
                println!("Add Gantt Chart action triggered");
                app_view.replace(
                    Box::new(ConfirmModal::new("Gantt Chart Added".to_string(),
                                                "Gantt Chart has been added successfully.".to_string())
                    )
                );

                let meter_widget = MeterWidget::new(self.dark_mode, MeterSettings {
                    min_value: 0.0,
                    max_value: 100.0,
                    unit: "%".to_string(),
                    warning_threshold: Some(75.0),
                    danger_threshold: Some(90.0),
                    show_digital_readout: true,
                    label: "CPU Usage".to_string(),
                });

                let new_id = self.module_widget.keys().max().unwrap_or(&0) + 1;

                self.module_widget.insert(new_id, ModuleWidget {
                    id: new_id,
                    module_widget: Box::new(meter_widget),
                    dlt_data_regex_item: None,
                });

                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::Delete => {
                println!("Delete action triggered");
                if let Some(menu) = &self.circular_context_menu {
                    if let Some(module_id) = menu.target_module {
                        self.module_widget.remove(&module_id);
                    }
                }
                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::Duplicate => {
                println!("Duplicate action triggered");
                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::Settings => {
                println!("Settings action triggered");
                if let Some(menu) = &self.circular_context_menu {
                    if let Some(module_id) = menu.target_module {
                        println!("Opening settings for module: {}", module_id);
                        let mut module = self.module_widget.get_mut(&module_id);

                        if let Some(module) = module {
                            if let Some(chart_widget) = module.module_widget.as_any_mut().downcast_mut::<ChartWidget>() {
                                let regex_item = module.dlt_data_regex_item.clone().unwrap_or(DltDataRegexItem {
                                    regex: "".to_string(),
                                    id: 0,
                                    description: "todo".to_string(),
                                });
                                // Clone the chart widget so the modal owns its copy and doesn't borrow from self
                                app_view.replace(
                                    Box::new(ChartWidgetModal::new(
                                        module_id as u32,
                                        format!("Chart Settings - {}", module_id),
                                        regex_item,
                                        chart_widget.clone(),
                                    ))
                                );
                            }
                        }

                    }
                }
                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::Move(position) => {
                // Calculate delta for panning
                let delta = if let Some(last_pos) = self.state.last_cursor_position {
                    Point::new(position.x - last_pos.x, position.y - last_pos.y)
                } else {
                    Point::new(0.0, 0.0)
                };
                self.state.last_cursor_position = Some(position);

                // Update hovered module
                self.hovered_module = self.get_module_at_position(position);
                
                if self.state.left_mouse_button.is_pressed {
                    // Handle chart panning when Shift is pressed
                    if self.shift_pressed && self.panning_chart.is_some() {
                        if let Some(panning_id) = self.panning_chart {
                            if let Some(module) = self.module_widget.get_mut(&panning_id) {
                                if let Some(chart_widget) = module.module_widget.as_any_mut().downcast_mut::<ChartWidget>() {
                                    // Pan the chart (invert delta for natural panning)
                                    chart_widget.pan(-delta.x * 0.5, delta.y * 0.5);
                                }
                            }
                        }
                    }
                    // Handle moving the selected module (only if not panning)
                    else if let Some(selected_id) = self.selected_module {
                        if self.panning_chart.is_none() {
                            if let Some(module) = self.module_widget.get_mut(&selected_id) {
                                let window = module.module_widget.get_window_mut();
                                window.position = Point {
                                    x: position.x - window.size.width / 2.0,
                                    y: position.y - window.size.height / 2.0,
                                };
                            }
                        }
                    }

                    // Handle resizing (only if not panning)
                    if self.panning_chart.is_none() {
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
                }

                if self.circular_context_menu.is_none() {
                    if self.state.right_mouse_button.is_pressed {
                        // Potentially handle right-drag actions
                        
                        // if the right drag is more than a threshold, open context menu
                        let press_pos = self.state.right_mouse_button.press_position;
                        let drag_distance = press_pos.map_or(0.0, |press_pos| {
                            ((position.x - press_pos.x).powi(2) + (position.y - press_pos.y).powi(2)).sqrt()
                        });
                        if drag_distance > 10.0 {
                            let target_module = self.get_module_at_position(position);
                            self.circular_context_menu = Some(CircularContextMenu::new(position, target_module));
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

                self.circular_context_menu = Some(CircularContextMenu::new(position, target_module));
            }
            ModuleCanvasMessage::CloseContextMenu => {
                self.circular_context_menu = None;
            }
            ModuleCanvasMessage::SelectModule(module_id) => {
                self.selected_module = module_id;
                println!("Selected module: {:?}", module_id);
            }
            ModuleCanvasMessage::MouseWheel(delta, position) => {
                // Handle zoom when hovering over a chart
                if let Some(hovered_id) = self.get_module_at_position(position) {
                    if let Some(module) = self.module_widget.get_mut(&hovered_id) {
                        if let Some(chart_widget) = module.module_widget.as_any_mut().downcast_mut::<ChartWidget>() {
                            // Zoom based on Shift key state
                            // Shift + Scroll = horizontal zoom
                            // Ctrl + Scroll = vertical zoom
                            // Just Scroll = both axes zoom
                            chart_widget.zoom(delta, !self.shift_pressed, !self.shift_pressed);
                        }
                    }
                }
            }
            ModuleCanvasMessage::ResetChartView(module_id) => {
                if let Some(module) = self.module_widget.get_mut(&module_id) {
                    if let Some(chart_widget) = module.module_widget.as_any_mut().downcast_mut::<ChartWidget>() {
                        chart_widget.reset_view();
                    }
                }
            }

            // Handle mouse button messages
            ModuleCanvasMessage::RightMouseReleased(_position) => {
                // Handle right mouse button release if needed
                self.state.right_mouse_button.is_pressed = false;

                if let Some(menu) = &self.circular_context_menu {
                    if let Some(action) = menu.get_action_at(_position, 90.0) {
                        let message = match action {
                            CircularContextMenuAction::AddChart => ModuleCanvasMessage::AddChart,
                            CircularContextMenuAction::AddGanttChart => ModuleCanvasMessage::AddGanttChart,
                            CircularContextMenuAction::Delete => ModuleCanvasMessage::Delete,
                            CircularContextMenuAction::Duplicate => ModuleCanvasMessage::Duplicate,
                            CircularContextMenuAction::Settings => ModuleCanvasMessage::Settings,
                        };

                        // Handle the action synchronously to avoid sending non-Send types via Task::done
                        let _ = self.update(message, app_view);
                        return Task::none();
                    }

                    self.circular_context_menu = None;
                }else{
                    let target_module = self.get_module_at_position(_position);

                    self.context_menu = Some(ContextMenu::new(
                        _position,
                        target_module,
                    ));
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
                self.panning_chart = None;  // Stop panning
                self.state.last_cursor_position = None;

                if let Some(context_menu) = &self.context_menu {
                    if let Some(action) = context_menu.get_action_at(_position) {
                        let message = match action {
                            ContextMenuAction::AddChart => ModuleCanvasMessage::AddChart,
                            ContextMenuAction::AddGanttChart => ModuleCanvasMessage::AddGanttChart,
                            ContextMenuAction::AddInjectionWindow => ModuleCanvasMessage::AddGanttChart, // Placeholder
                            ContextMenuAction::AddMeterWindow => ModuleCanvasMessage::AddGanttChart, // Placeholder
                            ContextMenuAction::Delete => ModuleCanvasMessage::Delete,
                            ContextMenuAction::Duplicate => ModuleCanvasMessage::Duplicate,
                            ContextMenuAction::Settings => ModuleCanvasMessage::Settings,
                        };

                        // Handle the action synchronously to avoid sending non-Send types via Task::done
                        let _ = self.update(message, app_view);
                    }

                    self.context_menu = None;
                }
            }
            ModuleCanvasMessage::LeftMousePressed(_position) => {
                self.state.left_mouse_button.is_pressed = true;
                self.state.left_mouse_button.press_position = Some(_position);
                self.state.last_cursor_position = Some(_position);

                self.selected_module = None;
                self.resize_module = None;
                self.panning_chart = None;

                // Check if Shift is pressed and we're clicking on a chart
                if self.shift_pressed {
                    let clicked_module = self.get_module_at_position(_position);
                    if let Some(module_id) = clicked_module {
                        if let Some(module) = self.module_widget.get(&module_id) {
                            // Check if it's a chart widget
                            if module.module_widget.as_any().is::<ChartWidget>() {
                                self.panning_chart = Some(module_id);
                                println!("Started panning chart: {}", module_id);
                            }
                        }
                    }
                }

                // Only select/resize if not panning
                if self.panning_chart.is_none() {
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
        if let Some(menu) = &self.circular_context_menu {
            draw_circular_context_menu(&mut frame, menu, cursor_position, self.dark_mode);
        }

        if let Some(context_menu) = &self.context_menu {
            draw_context_menu(&mut frame, context_menu, cursor_position, self.dark_mode);
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
                    // Handle zoom
                    if let Some(position) = cursor_position {
                        let scroll_delta = match delta {
                            mouse::ScrollDelta::Lines { y, .. } => y,
                            mouse::ScrollDelta::Pixels { y, .. } => y / 10.0,
                        };
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ModuleCanvasMessage(
                                ModuleCanvasMessage::MouseWheel(scroll_delta, position)
                            )),
                        );
                    }
                }
                _ => {}
            },
            canvas::Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::ModifiersChanged(modifiers) => {
                    let shift_pressed = modifiers.shift();
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::ShiftKeyChanged(shift_pressed)),
                    );
                }
                keyboard::Event::KeyPressed { key, .. } => {
                    // Handle 'R' key to reset view of hovered chart
                    if let keyboard::Key::Character(c) = &key {
                        if c.to_lowercase() == "r" {
                            if let Some(hovered_id) = self.hovered_module {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(Message::ModuleCanvasMessage(
                                        ModuleCanvasMessage::ResetChartView(hovered_id)
                                    )),
                                );
                            }
                        }
                    }
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
            // Show panning cursor when Shift is pressed and hovering over a chart
            if self.shift_pressed {
                if let Some(hovered_id) = self.hovered_module {
                    if let Some(module) = self.module_widget.get(&hovered_id) {
                        if module.module_widget.as_any().is::<ChartWidget>() {
                            if state.left_mouse_button.is_pressed {
                                return mouse::Interaction::Grabbing;
                            } else {
                                return mouse::Interaction::Grab;
                            }
                        }
                    }
                }
            }

            // Show different cursor based on what's being hovered
            if let Some(_) = self.hovered_module {
                if !self.shift_pressed {
                    if state.left_mouse_button.is_pressed {
                        return mouse::Interaction::Grabbing;
                    } else {
                        return mouse::Interaction::Grab;
                    }
                }
            }

            // Show pointer when hovering circular context menu
            if let Some(menu) = &self.circular_context_menu {
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