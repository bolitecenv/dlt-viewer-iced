use crate::components::dlt_data_manager::{
    DltDataChartItem, DltDataGattChartItem, DltDataModuleItem, DltDataRegexItem,
};
use crate::components::tcp_handler::{apply_ecu_updates, tcp_connection_subscription};
use crate::components::view::dlt_settings::{DltSelection, DltSettingsView};
use crate::components::view::gantt_chart_setting::{
    ModalWindow_ModuleGanttChartWidgetSettingsView, ModuleGanttChartWidgetSettingsMessage,
};
use crate::components::view::module_view_settings::{
    ModalWindow_ModuleChartWidgetSettingsView, ModuleChartWidgetSettingsMessage,
};
use crate::components::{navigation, top_bar};
use crate::message::{Message, Page};
use crate::module_view;
use crate::module_view::canvas::{ContextMenu, ContextMenuAction, handle_mouse_wheel}; // NEW: Add context menu imports
use crate::module_view::module_widget::{
    ChartData, ChartSettings, ChartWidget, GanttChartData, GanttChartDataPoint, GanttChartSettings,
    GanttChartWidget, ModuleWidgetCommonSettings, ModuleWidgetWindowView, WidgetTpye,
};
use crate::module_view::{DragState, ModuleWidget};
use crate::pages::ecu_setting::{EcuListView, EcuSelection};
use crate::pages::{self};
use crate::plugin::DashboardContext;
use crate::plugin_registry::PluginRegistry;
use crate::types::{FrontDltAppIdItem, FrontDltEcuItem};

use crate::message::ConnectionEvent;
use iced::futures::{self};
use iced::window::drag;
use iced::{
    Color, Font, Point, Size,
    font::{Family, Stretch, Style, Weight},
};
use iced::{
    Element, Length, Subscription, Task, Theme,
    widget::{column, container, row},
};
use pages::table::DltMessageRow;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

pub const ICON_FONT: Font = Font {
    family: Family::Name("Font Awesome 7 Free"),
    weight: Weight::Black,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

// NEW: Add ResizeState struct
#[derive(Debug, Clone)]
pub struct ResizeState {
    pub chart_id: usize,
    pub initial_size: Size,
    pub initial_cursor: Point,
}

pub struct Dashboard<T: ModuleWidgetWindowView> {
    pub current_page: Page,
    pub dark_mode: bool,
    pub tcp_ip: String,
    pub tcp_port: String,
    pub connection_status: String,
    pub should_connect: bool,
    pub messages: Vec<DltMessageRow>,
    pub message_id_counter: u32,
    pub max_messages: usize,
    pub module_widgets: HashMap<usize, ModuleWidget<T>>,
    pub next_id: usize,
    pub dragging: Option<DragState>,
    pub resizing: Option<ResizeState>,
    pub context_menu: Option<ContextMenu>,
    pub selected_chart_id: Option<usize>,
    pub hovered_action: Option<ContextMenuAction>,
    pub dlt_settings: DltSettingsView,
    pub chart_settings_modal: ModalWindow_ModuleChartWidgetSettingsView,
    pub gantt_chart_settings_modal: ModalWindow_ModuleGanttChartWidgetSettingsView,
    pub shift_pressed: bool,
    pub registry: PluginRegistry,
    pub current_plugin: Option<String>,
    pub ecu_list: Vec<FrontDltEcuItem>,
    pub regex_items: Vec<DltDataRegexItem>,
    pub ecu_list_view: EcuListView,
}

impl Default for Dashboard {
    fn default() -> Self {
        // Initialize with some example DLT data (optional)
        let ecu_list = Vec::new();

        Self {
            current_page: Page::Overview,
            dark_mode: false,
            tcp_ip: "127.0.0.1".to_string(),
            tcp_port: "3490".to_string(),
            connection_status: "Disconnected".to_string(),
            should_connect: false,
            messages: Vec::new(),
            message_id_counter: 0,
            max_messages: 1000,
            dlt_settings: DltSettingsView::new(),
            module_widgets: HashMap::new(),
            next_id: 0,
            dragging: None,
            resizing: None,
            context_menu: None,
            selected_chart_id: None,
            hovered_action: None,
            chart_settings_modal: ModalWindow_ModuleChartWidgetSettingsView::new(),
            gantt_chart_settings_modal: ModalWindow_ModuleGanttChartWidgetSettingsView::new(),
            shift_pressed: false,
            registry: PluginRegistry::new(),
            current_plugin: None,
            ecu_list: Vec::new(),
            regex_items: Vec::new(),
            ecu_list_view: EcuListView::new(ecu_list.clone()),
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => self.dark_mode = !self.dark_mode,
            Message::NavigateTo(page) => self.current_page = page,
            Message::Tick => {}
            Message::TcpIpChanged(ip) => self.tcp_ip = ip,
            Message::TcpPortChanged(port) => self.tcp_port = port,
            Message::ConnectTcp => {
                let _ip = self.tcp_ip.clone();
                let _port = self.tcp_port.clone();
                self.should_connect = true;
            }
            Message::ClearMessages => {
                self.messages.clear();
            }
            Message::ConnectionEvent(event) => match event {
                ConnectionEvent::Connecting => {
                    self.connection_status = "Connecting...".to_string();
                }
                ConnectionEvent::Connected => {
                    self.connection_status = "Connected".to_string();
                }
                ConnectionEvent::Disconnected => {
                    self.connection_status = "Disconnected".to_string();
                }
                ConnectionEvent::Error(err) => {
                    self.connection_status = format!("Error: {}", err);
                }
                ConnectionEvent::DltMessageReceived(data) => {
                    for row in data {
                        if self.messages.len() >= self.max_messages {
                            self.messages.remove(0);
                        }

                        self.messages.push(row);
                    }
                }
            },
            Message::OpenDltSettings => {
                self.dlt_settings.open();
                self.dlt_settings.set_dlt_items(self.ecu_list.clone());
            }
            Message::CloseDltSettings => {
                self.dlt_settings.close();
            }
            Message::SelectDltEcu(ecu_id) => {
                self.dlt_settings.toggle_ecu(ecu_id.clone());
                self.dlt_settings.select_item(DltSelection::Ecu(ecu_id));
            }
            Message::SelectDltApp(ecu_id, app_id) => {
                self.dlt_settings.toggle_app(ecu_id.clone(), app_id.clone());
                self.dlt_settings
                    .select_item(DltSelection::App(ecu_id, app_id));
            }
            Message::SelectDltContext(ecu_id, app_id, ctx_id) => {
                self.dlt_settings
                    .select_item(DltSelection::Context(ecu_id, app_id, ctx_id));
            }
            Message::RefreshDltItems => {
                println!("Refreshing DLT items...");
                // let dlt_item = DLT_ECU_CONTEXT_STORE.lock().unwrap().clone();
                // self.dlt_settings.set_dlt_items(dlt_item.clone());
            }
            Message::ApplyDltSettings => {
                println!("Applying DLT settings...");
                self.dlt_settings.close();
            }
            Message::UpdateLogLevel(new_level_str) => {
                println!("Updating log level to {}", new_level_str);
                if new_level_str.is_empty() {
                    self.dlt_settings.update_log_level(new_level_str);
                } else if new_level_str.parse::<i32>().is_ok() {
                    self.dlt_settings.update_log_level(new_level_str);
                }
            }
            Message::EditContext(log_level, trace_status) => {
                self.dlt_settings.start_editing(log_level, trace_status);
            }
            Message::CancelEditContext => {
                self.dlt_settings.close();
            }
            Message::StartResize(chart_id, cursor_position) => {
                if let Some(chart) = self.module_widgets.get(&chart_id) {
                    self.resizing = Some(ResizeState {
                        chart_id,
                        initial_size: chart.size,
                        initial_cursor: cursor_position,
                    });
                    // Cancel any drag operation
                    self.dragging = None;
                    // Close context menu
                    self.context_menu = None;
                }
            }

            Message::ShowContextMenu(cursor_position) => {
                // Find which chart was right-clicked on
                let mut clicked_chart_id = None;
                for (id, chart) in self.module_widgets.iter() {
                    let bounds = iced::Rectangle::new(chart.position, chart.size);
                    if bounds.contains(cursor_position) {
                        clicked_chart_id = Some(*id);
                        break;
                    }
                }

                self.selected_chart_id = clicked_chart_id;
                self.context_menu = Some(ContextMenu::new(cursor_position));
            }

            Message::RightMouseReleased(cursor_position) => {
                // Execute the action that was hovered when right button is released
                if self.context_menu.is_some() {
                    // Check if clicked on context menu
                    if let Some(menu) = &self.context_menu {
                        const MENU_RADIUS: f32 = 80.0;
                        let action = menu.get_action_at(cursor_position, MENU_RADIUS);

                        if let Some(action) = action {
                            // Trigger the action
                            return self.update(Message::ContextMenuAction(action));
                        }
                    }

                    // Clicked outside menu, close it
                    self.context_menu = None;
                    self.selected_chart_id = None;
                    return Task::none();
                }

                // No action was hovered, just close the menu
                self.context_menu = None;
                self.selected_chart_id = None;
                self.hovered_action = None;
            }

            Message::ContextMenuAction(action) => {
                match action {
                    ContextMenuAction::AddChart => {
                        // Create a new chart at the context menu position
                        if let Some(menu) = &self.context_menu {
                            let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
                            let data: Vec<f32> =
                                (0..6).map(|_| rng.gen_range(10.0..100.0)).collect();

                            let chart_data: Vec<ChartData> = data
                                .iter()
                                .enumerate()
                                .map(|(i, &y)| ChartData {
                                    x_value: i as f32,
                                    y_value: y,
                                })
                                .collect();

                            let chart_settings = ChartSettings {
                                show_grid: true,
                                show_legend: false,
                                line_smoothness: 0.0,
                                x_label: "X-Axis".to_string(),
                                y_label: "Y-Axis".to_string(),
                            };

                            let chart_widget = ChartWidget {
                                chart_data,
                                settings: chart_settings,
                            };

                            let common_settings = ModuleWidgetCommonSettings {
                                title: "Analytics".to_string(),
                                show_title: true,
                                background_color: if self.dark_mode {
                                    Color::from_rgba(0.2, 0.2, 0.25, 0.95)
                                } else {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.95)
                                },
                                color: Color::from_rgb(
                                    rng.gen_range(0.3..1.0),
                                    rng.gen_range(0.3..1.0),
                                    rng.gen_range(0.3..1.0),
                                ),
                                x_zoom: 1.0,
                                y_zoom: 1.0,
                                x_offset: 0.0,
                                y_offset: 0.0,
                            };

                            let mut module_widget = ModuleWidget::new(
                                self.next_id,
                                Point::new(rng.gen_range(50.0..500.0), rng.gen_range(50.0..400.0)),
                                Size::new(300.0, 200.0),
                                common_settings,
                                WidgetTpye::LineChart(chart_widget),
                            );

                            module_widget.dlt_data_regex_item = DltDataRegexItem {
                                id: 0,
                                regex: r"X:(?P<Xvalue>\d+\.?\d*),Y:(?P<Yvalue>\d+\.?\d*)"
                                    .to_string(),
                                description: "Simple X,Y Data Extractor".to_string(),
                            }
                            .into();

                            self.module_widgets.insert(self.next_id, module_widget);
                            self.next_id += 1;
                        }
                    }
                    ContextMenuAction::AddGanttChart => {
                        self.module_widgets.insert(
                            self.next_id,
                            ModuleWidget::default_gantt_chart_widget(
                                self.next_id,
                                Point::new(100.0, 100.0),
                                Size::new(400.0, 300.0),
                            ),
                        );
                        self.next_id += 1;
                    }
                    ContextMenuAction::Delete => {
                        // Delete the selected chart
                        if let Some(chart_id) = self.selected_chart_id {
                            self.module_widgets.remove(&chart_id);
                        }
                    }
                    ContextMenuAction::Duplicate => {
                        // Duplicate the selected chart
                    }
                    ContextMenuAction::Settings => {
                        // Open settings for the selected chart
                        if let Some(chart_id) = self.selected_chart_id {
                            println!("Opening settings for chart {}", chart_id);
                            match self.module_widgets.get(&chart_id) {
                                Some(widget) => match &widget.widget_type {
                                    WidgetTpye::LineChart(_) | WidgetTpye::BarChart(_) => {
                                        println!("It's a chart widget.");
                                        self.chart_settings_modal.open(
                                            self.module_widgets.get(&chart_id).cloned().unwrap(),
                                        );
                                        self.selected_chart_id = Some(chart_id);
                                    }
                                    WidgetTpye::GanttChart(_) => {
                                        println!("It's a Gantt chart widget.");
                                        self.gantt_chart_settings_modal.open(
                                            self.module_widgets.get(&chart_id).cloned().unwrap(),
                                        );
                                        self.selected_chart_id = Some(chart_id);
                                    }
                                },
                                None => {
                                    println!("Widget with ID {} not found!", chart_id);
                                }
                            }

                            // You can implement a settings dialog here
                        }
                    }
                }

                // Close the context menu after action
                self.context_menu = None;
            }

            Message::MousePressed(cursor_position) => {
                // Close context menu on any click
                if self.context_menu.is_some() {
                    // Check if clicked on context menu
                    if let Some(menu) = &self.context_menu {
                        const MENU_RADIUS: f32 = 80.0;
                        let action = menu.get_action_at(cursor_position, MENU_RADIUS);

                        if let Some(action) = action {
                            // Trigger the action
                            return self.update(Message::ContextMenuAction(action));
                        }
                    }

                    // Clicked outside menu, close it
                    self.context_menu = None;
                    self.selected_chart_id = None;
                    return Task::none();
                }

                // Note: StartResize is called first by the canvas if clicking resize handle
                // This will only be called if NOT clicking on a resize handle

                let mut clicked_id = None;
                for (id, chart) in self.module_widgets.iter() {
                    let bounds = iced::Rectangle::new(chart.position, chart.size);
                    if bounds.contains(cursor_position) {
                        clicked_id = Some(*id);
                    }
                }

                if let Some(id) = clicked_id {
                    let chart = &self.module_widgets[&id];
                    self.dragging = Some(DragState {
                        chart_id: id,
                        offset: iced::Point::new(
                            cursor_position.x - chart.position.x,
                            cursor_position.y - chart.position.y,
                        ),
                    });
                    // Cancel any resize operation
                    self.resizing = None;
                }
            }

            Message::MouseReleased => {
                // Snap to grid when releasing
                const GRID_SIZE: f32 = 50.0;

                if let Some(resize_state) = &self.resizing {
                    if let Some(chart) = self.module_widgets.get_mut(&resize_state.chart_id) {
                        chart.size = Size::new(
                            (chart.size.width / GRID_SIZE).round() * GRID_SIZE,
                            (chart.size.height / GRID_SIZE).round() * GRID_SIZE,
                        );
                    }
                }

                if let Some(drag_state) = &self.dragging {
                    if let Some(chart) = self.module_widgets.get_mut(&drag_state.chart_id) {
                        chart.position = iced::Point::new(
                            (chart.position.x / GRID_SIZE).round() * GRID_SIZE,
                            (chart.position.y / GRID_SIZE).round() * GRID_SIZE,
                        );
                    }
                }

                // End both drag and resize operations
                self.dragging = None;
                self.resizing = None;
            }

            Message::MouseMoved(cursor_position) => {
                // Update hovered action if context menu is showing
                if let Some(menu) = &self.context_menu {
                    const MENU_RADIUS: f32 = 80.0;
                    self.hovered_action = menu.get_action_at(cursor_position, MENU_RADIUS);
                }

                // UPDATED: Handle both resize and drag
                if let Some(resize_state) = &self.resizing {
                    // Handle resizing
                    if let Some(chart) = self.module_widgets.get_mut(&resize_state.chart_id) {
                        let delta_x = cursor_position.x - resize_state.initial_cursor.x;
                        let delta_y = cursor_position.y - resize_state.initial_cursor.y;

                        // Apply minimum size constraints
                        const MIN_WIDTH: f32 = 200.0;
                        const MIN_HEIGHT: f32 = 200.0;

                        let new_width = (resize_state.initial_size.width + delta_x).max(MIN_WIDTH);
                        let new_height =
                            (resize_state.initial_size.height + delta_y).max(MIN_HEIGHT);

                        chart.size = Size::new(new_width, new_height);
                    }
                } else if let Some(drag_state) = &self.dragging {
                    // Handle dragging
                    if let Some(chart) = self.module_widgets.get_mut(&drag_state.chart_id) {
                        let mut new_widget_x = cursor_position.x - drag_state.offset.x;
                        let mut new_widget_y = cursor_position.y - drag_state.offset.y;
                        if new_widget_x < 0.0 && new_widget_y < 0.0 {
                            new_widget_x = 0.0;
                            new_widget_y = 0.0;
                        } else if new_widget_x < 0.0 {
                            new_widget_x = 0.0;
                        } else if new_widget_y < 0.0 {
                            new_widget_y = 0.0;
                        }
                        chart.position = iced::Point::new(new_widget_x, new_widget_y);
                    }
                }
            }
            Message::CloseChartSettings(module_widget) => {
                if let Some(target_widget) = self
                    .module_widgets
                    .get_mut(&self.selected_chart_id.unwrap())
                {
                    *target_widget = module_widget;
                }
                match &self
                    .module_widgets
                    .get(&self.selected_chart_id.unwrap())
                    .unwrap()
                    .widget_type
                {
                    WidgetTpye::LineChart(_) | WidgetTpye::BarChart(_) => {
                        self.chart_settings_modal.close();
                    }
                    WidgetTpye::GanttChart(_) => {
                        self.gantt_chart_settings_modal.close();
                    }
                }
            }
            // Message::UpdateChartTitle(new_title) => {
            //     self.chart_settings_modal.update_title(new_title);
            // }
            // Message::UpdateXAxisLabel(new_label) => {
            //     self.chart_settings_modal.update_x_axis(new_label);
            // }
            // Message::UpdateYAxisLabel(new_label) => {
            //     self.chart_settings_modal.update_y_axis(new_label);
            // }
            Message::MouseWheel(chart_id, delta) => {
                if let Some(chart) = self.module_widgets.get_mut(&chart_id) {
                    handle_mouse_wheel(chart, delta, self.shift_pressed);
                }
            }
            Message::ShiftKeyChanged(pressed) => {
                self.shift_pressed = pressed;
            }
            Message::UpdateModuleChartWidgetSettingsMessage(msg) => match msg {
                ModuleChartWidgetSettingsMessage::UpdateChartTitle(new_title) => {
                    if let Some(widget) = &mut self.chart_settings_modal.widget {
                        self.chart_settings_modal.update_title(new_title);
                    }
                }
                ModuleChartWidgetSettingsMessage::UpdateXAxisLabel(new_label) => {
                    if let Some(widget) = &mut self.chart_settings_modal.widget {
                        self.chart_settings_modal.update_x_label(new_label);
                    }
                }
                ModuleChartWidgetSettingsMessage::UpdateYAxisLabel(new_label) => {
                    if let Some(widget) = &mut self.chart_settings_modal.widget {
                        self.chart_settings_modal.update_y_label(new_label);
                    }
                }
                ModuleChartWidgetSettingsMessage::UpdateRegexPattern(new_pattern) => {
                    if let Some(widget) = &mut self.chart_settings_modal.widget {
                        self.chart_settings_modal.update_regex_pattern(new_pattern);
                    }
                }
                _ => {}
            },
            Message::UpdateGanttChartWidgetSettingsMessage(msg) => match msg {
                ModuleGanttChartWidgetSettingsMessage::UpdateChartTitle(new_title) => {
                    if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                        self.gantt_chart_settings_modal.update_title(new_title);
                    }
                }
                ModuleGanttChartWidgetSettingsMessage::UpdateTimeScale(new_scale_str) => {
                    if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                        self.gantt_chart_settings_modal
                            .update_time_scale(new_scale_str);
                    }
                }
                ModuleGanttChartWidgetSettingsMessage::UpdateRegexPattern(new_pattern) => {
                    if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                        self.gantt_chart_settings_modal
                            .update_regex_pattern(new_pattern);
                    }
                }
                _ => {}
            },
            Message::PluginSelected(name) => {
                self.current_plugin = Some(name);
                return Task::none();
            }
            Message::PluginMessage(plugin_name, msg) => {
                // Clone the dashboard data so the context does not hold an immutable borrow of `self`
                // while we need a mutable borrow for `self.registry.update`.
                let ecu_list_clone = self.ecu_list.clone();
                let dlt_buffer_clone = self.messages.clone();
                let context = DashboardContext {
                    ecu_list: &ecu_list_clone,
                    dlt_buffer: &dlt_buffer_clone,
                };
                let task = self.registry.update(&plugin_name, msg, &context);
                return task.map(move |plugin_msg| {
                    Message::PluginMessage(plugin_name.clone(), plugin_msg)
                });
            }
            Message::EcuListUpdate(ecu_updates) => {
                // Apply ECU updates to your ecu_list
                apply_ecu_updates(&mut self.ecu_list, ecu_updates);
                self.ecu_list_view.set_ecu_list(self.ecu_list.clone());
            }

            Message::BatchUpdate {
                dlt_messages,
                ecu_updates,
            } => {
                self.process_dlt_messages(dlt_messages);

                // 2. Apply ECU updates
                apply_ecu_updates(&mut self.ecu_list, ecu_updates);
                self.ecu_list_view.set_ecu_list(self.ecu_list.clone());
            }
            Message::SelectEcu(ecu_id) => {
                self.ecu_list_view.toggle_ecu(ecu_id.clone());
                self.ecu_list_view.select_item(EcuSelection::Ecu(ecu_id));
            }
            Message::SelectApp(ecu_id, app_id) => {
                self.ecu_list_view
                    .toggle_app(ecu_id.clone(), app_id.clone());
                self.ecu_list_view
                    .select_item(EcuSelection::App(ecu_id, app_id));
            }
            Message::SelectContext(ecu_id, app_id, ctx_id) => {
                self.ecu_list_view
                    .select_item(EcuSelection::Context(ecu_id, app_id, ctx_id));
            }

            // Context Editing Messages
            Message::ECUViewEditContext(log_level, trace_status) => {
                println!(
                    "Editing context: log_level={}, trace_status={}",
                    log_level, trace_status
                );
                self.ecu_list_view.start_editing(log_level, trace_status);
            }
            Message::UpdateLogLevel(value) => {
                self.ecu_list_view.update_log_level(value);
            }
            Message::UpdateTraceStatus(value) => {
                self.ecu_list_view.update_trace_status(value);
            }
            Message::SaveContextSettings => {
                // Parse the input values and save
                if let Ok(log_level) = self.ecu_list_view.edit_state.log_level_input.parse::<i8>() {
                    if let Ok(trace_status) = self
                        .ecu_list_view
                        .edit_state
                        .trace_status_input
                        .parse::<i8>()
                    {
                        // Update the actual data in ecu_list
                        if let EcuSelection::Context(ecu_id, app_id, ctx_id) =
                            &self.ecu_list_view.selected_item
                        {
                            self.ecu_list_view.update_context_settings(
                                ecu_id.clone(),
                                app_id.clone(),
                                ctx_id.clone(),
                                log_level,
                                trace_status,
                            );
                        }
                    }
                }
                self.ecu_list_view.cancel_editing();
            }
            Message::CancelEditContext => {
                self.ecu_list_view.cancel_editing();
            }

            // Injection Message Messages
            Message::UpdateMessageType(value) => {
                self.ecu_list_view.update_message_type(value);
            }
            Message::UpdateInjectionMessage(value) => {
                self.ecu_list_view.update_message(value);
            }
            Message::InjectMessage(ecu_id, app_id, ctx_id, message) => {
                // Implement your message injection logic here
                println!(
                    "Injecting message to {}/{}/{}: {}",
                    ecu_id, app_id, ctx_id, message
                );
                // TODO: Call your DLT injection function
            }
            Message::ClearInjectionMessage => {
                self.ecu_list_view.clear_message();
            }
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let tick_subscription = Subscription::run(|| {
            futures::stream::unfold((), |_| async {
                sleep(Duration::from_millis(100)).await;
                Some((Message::Tick, ()))
            })
        });

        let connection_subscription = if self.should_connect {
            tcp_connection_subscription(self.tcp_ip.clone(), self.tcp_port.clone())
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![tick_subscription, connection_subscription])
    }

    pub fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn view(&self) -> Element<Message> {
        let top = top_bar::view(self.dark_mode);
        let nav = navigation::view(self.current_page.clone(), &self.registry, self.dark_mode);

        let canvas_content = module_view::canvas::view(
            self.module_widgets.clone(),
            self.dark_mode,
            self.context_menu.clone(), // NEW: Pass context menu to canvas
        );

        let main_content = match self.current_page {
            Page::Overview => pages::overview::view(self),
            Page::Reports => pages::placeholder::view("Reports", "📋", self.dark_mode),
            Page::ECUSetting => self.ecu_list_view.view(self.dark_mode),
            Page::Settings => pages::settings::view(
                self.dark_mode,
                &self.tcp_ip,
                &self.tcp_port,
                &self.connection_status,
            ),
            Page::Table => pages::table::view(self.dark_mode, &self.messages),
            Page::ChartCanvas => canvas_content,
            Page::PluginPage(ref plugin_name) => {
                if let Some(plugin) = self.registry.get_plugin(plugin_name) {
                    plugin.view(&self.get_context()).map(move |plugin_msg| {
                        Message::PluginMessage(plugin_name.clone(), plugin_msg)
                    })
                } else {
                    pages::placeholder::view("Plugin Not Found", "❓", self.dark_mode)
                }
            }
        };

        let content_area = container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        let main_layout = column![top, row![nav, content_area].height(Length::Fill)];

        let base_view = container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(dlt_popup) = self.dlt_settings.view(self.dark_mode) {
            use iced::widget::stack;

            stack![base_view, dlt_popup,].into()
        } else if let Some(chart_settings_popup) = self.chart_settings_modal.view(self.dark_mode) {
            use iced::widget::stack;

            stack![base_view, chart_settings_popup,].into()
        } else if let Some(gantt_chart_settings_popup) =
            self.gantt_chart_settings_modal.view(self.dark_mode)
        {
            use iced::widget::stack;

            stack![base_view, gantt_chart_settings_popup,].into()
        } else {
            base_view.into()
        }
    }
}

impl Dashboard {
    fn get_context(&self) -> DashboardContext<'_> {
        DashboardContext {
            ecu_list: &self.ecu_list,
            dlt_buffer: &self.messages,
        }
    }

    pub fn get_ecu_apps(&self, ecu_id: &str) -> Option<&Vec<FrontDltAppIdItem>> {
        self.ecu_list
            .iter()
            .find(|ecu| ecu.ecuid == ecu_id)
            .map(|ecu| &ecu.app_ids)
    }

    /// Get a specific app info
    pub fn get_app_info(&self, ecu_id: &str, app_id: &str) -> Option<&FrontDltAppIdItem> {
        self.ecu_list
            .iter()
            .find(|ecu| ecu.ecuid == ecu_id)
            .and_then(|ecu| ecu.app_ids.iter().find(|app| app.apid == app_id))
    }

    /// Get all ECU IDs
    pub fn get_all_ecu_ids(&self) -> Vec<String> {
        self.ecu_list.iter().map(|ecu| ecu.ecuid.clone()).collect()
    }

    /// Get all app IDs for an ECU
    pub fn get_all_app_ids(&self, ecu_id: &str) -> Vec<String> {
        self.ecu_list
            .iter()
            .find(|ecu| ecu.ecuid == ecu_id)
            .map(|ecu| ecu.app_ids.iter().map(|app| app.apid.clone()).collect())
            .unwrap_or_default()
    }

    /// Get context count for an app
    pub fn get_context_count(&self, ecu_id: &str, app_id: &str) -> usize {
        self.get_app_info(ecu_id, app_id)
            .map(|app| app.ctx_ids.len())
            .unwrap_or(0)
    }

    fn process_dlt_messages(&mut self, mut messages: Vec<DltMessageRow>) {
        println!("Received {} DLT messages", messages.len());
        if self.messages.len() > self.max_messages {
            let excess = self.messages.len() + messages.len() - self.max_messages;
            self.messages.drain(0..excess);
        }

        for row in &messages {
            if self.module_widgets.len() > 0 {
                for (_id, widget) in self.module_widgets.iter_mut() {
                    let regex =
                        Regex::new(&widget.dlt_data_regex_item.as_ref().unwrap().regex).unwrap();

                    if regex.is_match(&row.payload) {
                        match &mut widget.dlt_data_regex_item {
                            Some(item) => {
                                match &mut widget.widget_type {
                                    WidgetTpye::LineChart(chart_widget) => {
                                        // extract x,y data and process
                                        let captures = regex.captures(&row.payload).unwrap();
                                        let x_value: f32 = captures
                                            .name("Xvalue")
                                            .unwrap()
                                            .as_str()
                                            .parse()
                                            .unwrap();
                                        let y_value: f32 = captures
                                            .name("Yvalue")
                                            .unwrap()
                                            .as_str()
                                            .parse()
                                            .unwrap();

                                        // Print colorized debug info
                                        println!(
                                            "\x1b[32m[DLT Data Matched]\x1b[0m Payload: '{}', Extracted x: {}, y: {}",
                                            row.payload, x_value, y_value
                                        );

                                        chart_widget
                                            .chart_data
                                            .push(ChartData { x_value, y_value });
                                    }
                                    WidgetTpye::BarChart(chart_widget) => {
                                        // extract x,y data and process
                                        let captures = regex.captures(&row.payload).unwrap();
                                        let x_value: f32 = captures
                                            .name("Xvalue")
                                            .unwrap()
                                            .as_str()
                                            .parse()
                                            .unwrap();
                                        let y_value: f32 = captures
                                            .name("Yvalue")
                                            .unwrap()
                                            .as_str()
                                            .parse()
                                            .unwrap();

                                        chart_widget
                                            .chart_data
                                            .push(ChartData { x_value, y_value });
                                    }
                                    WidgetTpye::GanttChart(_) => {
                                        println!(
                                            "Processing Gantt Chart DLT payload: {}",
                                            row.payload
                                        );
                                        if let Some(captures) = regex.captures(&row.payload) {
                                            let function_name = captures.get(1).unwrap().as_str();
                                            let marker_type = captures.get(2).unwrap().as_str();

                                            println!(
                                                "\x1b[32mGantt Chart Marker Detected\x1b[0m: Function='{}', Type='{}'",
                                                function_name, marker_type
                                            );

                                            let start_time: f32 = match row.timestamp.parse::<f32>()
                                            {
                                                Ok(v) => v,
                                                Err(_) => {
                                                    println!(
                                                        "Warning: failed to parse timestamp '{}' to f32",
                                                        row.timestamp
                                                    );
                                                    0.0
                                                }
                                            };

                                            match marker_type {
                                                "S" => {
                                                    // Handle function start
                                                    println!(
                                                        "Function '{}' started",
                                                        function_name
                                                    );
                                                }
                                                "E" => {
                                                    // Handle function end
                                                    println!("Function '{}' ended", function_name);
                                                }
                                                "D" => {
                                                    // Handle function duration marker
                                                    let duration: f32 = captures
                                                        .get(3)
                                                        .unwrap()
                                                        .as_str()
                                                        .parse()
                                                        .unwrap();
                                                    let end_time = start_time + duration;
                                                    if let WidgetTpye::GanttChart(gantt_widget) =
                                                        &mut widget.widget_type
                                                    {
                                                        gantt_widget.chart_data.data_points.push(
                                                            GanttChartDataPoint {
                                                                y_label: function_name.to_string(),
                                                                start_time,
                                                                end_time,
                                                            },
                                                        );
                                                        println!(
                                                            "Added Gantt chart data point: Function='{}', Start={}, End={}",
                                                            function_name, start_time, end_time
                                                        );
                                                    }
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        for row in &mut messages {
            row.index = self.message_id_counter;
            self.message_id_counter += 1;
            self.messages.push(row.clone());
        }
    }

    fn add_regex_item(&mut self, item: DltDataRegexItem) {
        self.regex_items.push(item);
    }
}
