use crate::components::view::dlt_settings::{DltSelection, DltSettingsView};
use crate::components::view::gantt_chart_setting::{ModalWindow_ModuleGanttChartWidgetSettingsView, ModuleGanttChartWidgetSettingsMessage};
use crate::components::view::module_view_settings::{ModalWindow_ModuleChartWidgetSettingsView, ModuleChartWidgetSettingsMessage};
use crate::components::{navigation, top_bar};
use crate::message::{Message, Page};
use crate::module_view::module_widget::{ChartData, ChartSettings, ChartWidget, GanttChartData, GanttChartDataPoint, GanttChartSettings, GanttChartWidget, ModuleWidgetCommonSettings, WidgetTpye};
use crate::pages::{self};
use crate::components::tcp_handler::tcp_connection_subscription;
use crate::plugin::DashboardContext;
use crate::plugin_registry::PluginRegistry;
use crate::types::FrontDltEcuItem;
use crate::module_view;
use crate::module_view::{ModuleWidget, DragState};
use crate::module_view::canvas::{ContextMenu, ContextMenuAction, handle_mouse_wheel};  // NEW: Add context menu imports
use crate::components::dlt_data_manager::{DLT_DATA_REGEX_STORE, DLT_ECU_CONTEXT_STORE, DltDataChartItem, DltDataGattChartItem, DltDataModuleItem, DltDataRegexItem, add_dlt_data_regex_item};

use iced::{Color, Font, Point, Size, font::{Family, Stretch, Weight, Style}};
use iced::futures::{self};
use iced::{
    Element, Length, Subscription, Task, Theme,
    widget::{column, container, row},
};
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use pages::table::DltMessageRow;
use crate::message::ConnectionEvent;

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

pub struct Dashboard {
    pub metric1: i32,
    pub metric2: i32,
    pub total_users: u32,
    pub active_sessions: u32,
    pub current_page: Page,
    pub dark_mode: bool,
    pub tcp_ip: String,
    pub tcp_port: String,
    pub connection_status: String,
    pub should_connect: bool,
    pub messages: Vec<DltMessageRow>,
    pub max_messages: usize,
    pub module_widgets: HashMap<usize, ModuleWidget>,
    pub next_id: usize,
    pub dragging: Option<DragState>,
    pub resizing: Option<ResizeState>,  // NEW: Add resize state
    pub context_menu: Option<ContextMenu>,  // NEW: Add context menu state
    pub selected_chart_id: Option<usize>,  // NEW: Track which chart is selected for context actions
    pub hovered_action: Option<ContextMenuAction>,  // NEW: Track hovered action in context menu
    pub dlt_settings: DltSettingsView,
    pub chart_settings_modal: ModalWindow_ModuleChartWidgetSettingsView,
    pub gantt_chart_settings_modal: ModalWindow_ModuleGanttChartWidgetSettingsView,
    pub shift_pressed: bool,
    pub registry: PluginRegistry,
    pub current_plugin: Option<String>,
}

impl Default for Dashboard {
    fn default() -> Self {
        // Initialize with some example DLT data (optional)
        use crate::types::{FrontDltAppIdItem, FrontDltCtxIdItem};

        let regex = r"x:(?P<Xvalue>\d+\.?\d*),y:(?P<Yvalue>\d+\.?\d*)".to_string();
        add_dlt_data_regex_item(
            regex,
            "Chart data extractor".to_string(),
            DltDataModuleItem::Chart(
                DltDataChartItem {
                    id: 0,
                    x_label: "X-Axis".to_string(),
                    y_label: "Y-Axis".to_string(),
                    description: "Extracts x and y values from DLT payload".to_string(),
                    data_points: Vec::new(),
                },
            ),
        );

        let mut dlt_ecu_item = DLT_ECU_CONTEXT_STORE.lock().unwrap();
        // Add ECU1 with some apps and contexts
        dlt_ecu_item.push(FrontDltEcuItem {
            ecuid: "ECU1".to_string(),
            app_ids: vec![
                FrontDltAppIdItem {
                    apid: "APP1".to_string(),
                    description: "Application 1".to_string(),
                    ctx_ids: vec![
                        FrontDltCtxIdItem {
                            context_id: "CTX1".to_string(),
                            description: "Context 1".to_string(),
                            log_level: 3,
                            trace_status: 1,
                        },
                        FrontDltCtxIdItem {
                            context_id: "CTX2".to_string(),
                            description: "Context 2".to_string(),
                            log_level: 4,
                            trace_status: 0,
                        },
                    ],
                },
            ],
            description: "Engine Control Unit".to_string(),
        });

        println!("Initialized DLT ECU Context Store: {:?}", dlt_ecu_item);

        Self {
            metric1: 42,
            metric2: 78,
            total_users: 1247,
            active_sessions: 89,
            current_page: Page::Overview,
            dark_mode: true,
            tcp_ip: "127.0.0.1".to_string(),
            tcp_port: "3490".to_string(),
            connection_status: "Disconnected".to_string(),
            should_connect: false,
            messages: Vec::new(),
            max_messages: 1000,
            dlt_settings: DltSettingsView::new(),
            module_widgets: HashMap::new(),
            next_id: 0,
            dragging: None,
            resizing: None,  // NEW: Initialize resize state
            context_menu: None,  // NEW: Initialize context menu
            selected_chart_id: None,  // NEW: Initialize selected chart
            hovered_action: None,  // NEW: Initialize hovered action
            chart_settings_modal: ModalWindow_ModuleChartWidgetSettingsView::new(),
            gantt_chart_settings_modal: ModalWindow_ModuleGanttChartWidgetSettingsView::new(),
            shift_pressed: false,
            registry: PluginRegistry::new(),
            current_plugin: None,
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => self.dark_mode = !self.dark_mode,
            Message::NavigateTo(page) => self.current_page = page,
            Message::Tick => {
            }
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

                        if self.module_widgets.len() > 0 {
                            for (_id, widget) in self.module_widgets.iter_mut() {
                                let regex = Regex::new(&widget.dlt_data_regex_item.as_ref().unwrap().regex).unwrap();

                                if regex.is_match(&row.payload) {
                                    match &mut widget.dlt_data_regex_item {
                                        Some(item) => {
                                            match &mut widget.widget_type {
                                                WidgetTpye::LineChart(chart_widget) => {
                                                    // extract x,y data and process
                                                    let captures = regex.captures(&row.payload).unwrap();
                                                    let x_value: f32 = captures.name("Xvalue").unwrap().as_str().parse().unwrap();
                                                    let y_value: f32 = captures.name("Yvalue").unwrap().as_str().parse().unwrap();

                                                    // Print colorized debug info
                                                    println!("\x1b[32m[DLT Data Matched]\x1b[0m Payload: '{}', Extracted x: {}, y: {}", row.payload, x_value, y_value);

                                                    chart_widget.chart_data.push(ChartData {
                                                        x_value,
                                                        y_value,
                                                    });
                                                },
                                                WidgetTpye::BarChart(chart_widget) => {
                                                    // extract x,y data and process
                                                    let captures = regex.captures(&row.payload).unwrap();
                                                    let x_value: f32 = captures.name("Xvalue").unwrap().as_str().parse().unwrap();
                                                    let y_value: f32 = captures.name("Yvalue").unwrap().as_str().parse().unwrap();

                                                    chart_widget.chart_data.push(ChartData {
                                                        x_value,
                                                        y_value,
                                                    });
                                                },
                                                WidgetTpye::GanttChart(_) => {
                                                    println!("Processing Gantt Chart DLT payload: {}", row.payload);
                                                    if let Some(captures) = regex.captures(&row.payload) {
                                                        let function_name = captures.get(1).unwrap().as_str();
                                                        let marker_type = captures.get(2).unwrap().as_str();

                                                        println!("\x1b[32mGantt Chart Marker Detected\x1b[0m: Function='{}', Type='{}'", function_name, marker_type);
                                                        
                                                        let start_time: f32 = match row.timestamp.parse::<f32>() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                println!("Warning: failed to parse timestamp '{}' to f32", row.timestamp);
                                                                0.0
                                                            }
                                                        };

                                                        match marker_type {
                                                            "S" => {
                                                                // Handle function start
                                                                println!("Function '{}' started", function_name);
                                                            },
                                                            "E" => {
                                                                // Handle function end
                                                                println!("Function '{}' ended", function_name);
                                                            },
                                                            "D" => {
                                                                // Handle function duration marker
                                                                let duration: f32 = captures.get(3).unwrap().as_str().parse().unwrap();
                                                                let end_time = start_time + duration;
                                                                if let WidgetTpye::GanttChart(gantt_widget) = &mut widget.widget_type {
                                                                    gantt_widget.chart_data.data_points.push(
                                                                        GanttChartDataPoint {
                                                                            y_label: function_name.to_string(),
                                                                            start_time,
                                                                            end_time,
                                                                        }
                                                                    );
                                                                    println!("Added Gantt chart data point: Function='{}', Start={}, End={}", function_name, start_time, end_time);
                                                                }
                                                            },
                                                            _ => unreachable!(),
                                                        }
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        None => {},
                                    }
                                }
                            }
                        }

                        self.messages.push(row);
                    }
                }
            },
            Message::OpenDltSettings => {
                self.dlt_settings.open();
                let dlt_item = DLT_ECU_CONTEXT_STORE.lock().unwrap();
                self.dlt_settings.set_dlt_items(dlt_item.clone());

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
                self.dlt_settings.select_item(DltSelection::App(ecu_id, app_id));
            }
            Message::SelectDltContext(ecu_id, app_id, ctx_id) => {
                self.dlt_settings.select_item(DltSelection::Context(ecu_id, app_id, ctx_id));
            }
            Message::RefreshDltItems => {
                println!("Refreshing DLT items...");
                let dlt_item = DLT_ECU_CONTEXT_STORE.lock().unwrap().clone();
                self.dlt_settings.set_dlt_items(dlt_item.clone());
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
            Message::EditContext(log_level, trace_status, ) => {
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
                            let data: Vec<f32> = (0..6).map(|_| rng.gen_range(10.0..100.0)).collect();

                            let chart_data: Vec<ChartData> = data.iter().enumerate().map(|(i, &y)| {
                                ChartData {
                                    x_value: i as f32,
                                    y_value: y,
                                }
                            }).collect();

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

                            let dlt_regex_store = DLT_DATA_REGEX_STORE.lock().unwrap();

                            let mut module_widget = ModuleWidget::new(
                                self.next_id,
                                Point::new(
                                    rng.gen_range(50.0..500.0),
                                    rng.gen_range(50.0..400.0),
                                ),
                                Size::new(300.0, 200.0),
                                common_settings,
                                WidgetTpye::LineChart(chart_widget),
                            );

                            module_widget.dlt_data_regex_item = dlt_regex_store.get(0).cloned();

                            self.module_widgets.insert(self.next_id, module_widget);
                            self.next_id += 1;
                        }
                    }
                    ContextMenuAction::AddGanttChart => {
                        // Create a new Gantt chart module widget
                        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

                        let gantt_chart_settings = GanttChartSettings {
                            time_scale: 1.0,
                            show_dependencies: false,
                        };
                        
                        let gantt_chart_widget = GanttChartWidget {
                            chart_data: GanttChartData {
                                data_points: vec![
                                    // Example data points
                                    GanttChartDataPoint {
                                        y_label: "Task 1".to_string(),
                                        start_time: 0.0,
                                        end_time: 5.0,
                                    },
                                    GanttChartDataPoint {
                                        y_label: "Task 2".to_string(),
                                        start_time: 3.0,
                                        end_time: 8.0,
                                    },
                                    GanttChartDataPoint {
                                        y_label: "Task 3".to_string(),
                                        start_time: 6.0,
                                        end_time: 10.0,
                                    },
                                ],
                            },
                            settings: gantt_chart_settings,
                        };
                        let common_settings = ModuleWidgetCommonSettings {
                            title: "Gantt Chart".to_string(),
                            show_title: true,
                            background_color: if self.dark_mode {
                                Color::from_rgba(0.2, 0.2, 0.25, 0.05)
                            } else {
                                Color::from_rgba(1.0, 1.0, 1.0, 0.05)
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
                            Point::new(
                                rng.gen_range(50.0..500.0),
                                rng.gen_range(50.0..400.0),
                            ),
                            Size::new(400.0, 300.0),
                            common_settings,
                            WidgetTpye::GanttChart(gantt_chart_widget),
                        );

                        module_widget.dlt_data_regex_item = DltDataRegexItem {
                            id: 0,
                            regex: r"([^>]+),([D]),(\d+)".to_string(),
                            description: "Gantt Chart Function Marker Extractor".to_string(),
                            item_type: DltDataModuleItem::GattChart(
                                DltDataGattChartItem {
                                    id: 0,
                                    label: "Gantt Chart Extractor".to_string(),
                                    description: "Extracts function start/end markers for Gantt chart".to_string(),
                                    point_items: Vec::new(),
                                }
                            ),
                        }.into();

                        self.module_widgets.insert(self.next_id, module_widget);
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
                                Some(widget) => {
                                    match &widget.widget_type {
                                        WidgetTpye::LineChart(_) | WidgetTpye::BarChart(_) => {
                                            println!("It's a chart widget.");
                                            self.chart_settings_modal.open(self.module_widgets.get(&chart_id).cloned().unwrap());
                                            self.selected_chart_id = Some(chart_id);
                                        }
                                        WidgetTpye::GanttChart(_) => {
                                            println!("It's a Gantt chart widget.");
                                            self.gantt_chart_settings_modal.open(self.module_widgets.get(&chart_id).cloned().unwrap());
                                            self.selected_chart_id = Some(chart_id);
                                        }
                                    }
                                }
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
                        let new_height = (resize_state.initial_size.height + delta_y).max(MIN_HEIGHT);
                        
                        chart.size = Size::new(new_width, new_height);
                    }
                } else if let Some(drag_state) = &self.dragging {
                    // Handle dragging
                    if let Some(chart) = self.module_widgets.get_mut(&drag_state.chart_id) {
                        chart.position = iced::Point::new(
                            cursor_position.x - drag_state.offset.x,
                            cursor_position.y - drag_state.offset.y,
                        );
                    }
                }
            }
            Message::CloseChartSettings(module_widget) => {
                if let Some(target_widget) = self.module_widgets.get_mut(&self.selected_chart_id.unwrap()) {
                    *target_widget = module_widget;
                }
                match &self.module_widgets.get(&self.selected_chart_id.unwrap()).unwrap().widget_type {
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
            Message::UpdateModuleChartWidgetSettingsMessage(msg) => {
                match msg {
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
                }
            }
            Message::UpdateGanttChartWidgetSettingsMessage(msg) => {
                match msg {
                    ModuleGanttChartWidgetSettingsMessage::UpdateChartTitle(new_title) => {
                        if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                            self.gantt_chart_settings_modal.update_title(new_title);
                        }
                    }
                    ModuleGanttChartWidgetSettingsMessage::UpdateTimeScale(new_scale_str) => {
                        if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                            self.gantt_chart_settings_modal.update_time_scale(new_scale_str);
                        }
                    }
                    ModuleGanttChartWidgetSettingsMessage::UpdateRegexPattern(new_pattern) => {
                        if let Some(widget) = &mut self.gantt_chart_settings_modal.widget {
                            self.gantt_chart_settings_modal.update_regex_pattern(new_pattern);
                        }
                    }
                    _ => {}
                }
            }
            Message::PluginSelected(name) => {
                self.current_plugin = Some(name);
                return Task::none()
            }
            Message::PluginMessage(plugin_name, msg) => {
                let context = self.get_context();
                let task = self.registry.update(&plugin_name, msg, &context);
                return task.map(move |plugin_msg| Message::PluginMessage(plugin_name.clone(), plugin_msg));
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
            self.context_menu.clone(),  // NEW: Pass context menu to canvas
        );

        let main_content = match self.current_page {
            Page::Overview => pages::overview::view(self),
            Page::Reports => pages::placeholder::view("Reports", "📋", self.dark_mode),
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
                    pages::placeholder::view(
                        "Plugin Not Found",
                        "❓",
                        self.dark_mode,
                    )
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
            
            stack![
                base_view,
                dlt_popup,
            ].into()
        } else if let Some(chart_settings_popup) = self.chart_settings_modal.view(self.dark_mode) {
            use iced::widget::stack;
            
            stack![
                base_view,
                chart_settings_popup,
            ].into()
        } else if let Some(gantt_chart_settings_popup) = self.gantt_chart_settings_modal.view(self.dark_mode) {
            use iced::widget::stack;
            
            stack![
                base_view,
                gantt_chart_settings_popup,
            ].into()
        } else {
            base_view.into()
        }
    }
}

impl Dashboard {
    fn get_context(&self) -> DashboardContext {
        DashboardContext {
            ecu_list: DLT_ECU_CONTEXT_STORE.lock().unwrap().clone(),
            dlt_buffer: self.messages.clone(),
        }
    }
}