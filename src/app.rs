use crate::components::tcp_handler::{apply_ecu_updates, tcp_connection_subscription};
use crate::components::{navigation, top_bar};
use crate::message::{Message, Page};
use crate::module_view::ModuleCanvas;
use crate::modal_window::modal_window::*;
use crate::pages::ecu_setting::{EcuListView, EcuSelection};
use crate::pages::{self};
use crate::plugin::DashboardContext;
use crate::plugin_registry::PluginRegistry;
use crate::types::FrontDltEcuItem;
use crate::message::ConnectionEvent;
use iced::futures::{self};
use iced::widget::stack;
use iced::{
    Font,
    font::{Family, Stretch, Style, Weight},
};
use iced::{
    Element, Length, Subscription, Task, Theme,
    widget::{column, container, row},
};
use pages::table::DltMessageRow;
use std::time::Duration;
use tokio::time::sleep;


pub const ICON_FONT: Font = Font {
    family: Family::Name("Font Awesome 7 Free"),
    weight: Weight::Black,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub struct Dashboard {
    pub current_page: Page,
    pub dark_mode: bool,
    pub tcp_ip: String,
    pub tcp_port: String,
    pub connection_status: String,
    pub should_connect: bool,
    pub messages: Vec<DltMessageRow>,
    pub message_id_counter: u32,
    pub max_messages: usize,
    pub module_canvas: ModuleCanvas,
    pub registry: PluginRegistry,
    pub current_plugin: Option<String>,
    pub ecu_list: Vec<FrontDltEcuItem>,
    pub ecu_list_view: EcuListView,
    pub modal_window: Option<Box<dyn ModalWindowView>>,
}

impl Default for Dashboard {
    fn default() -> Self {
        // Initialize with some example DLT data (optional)
        let ecu_list = Vec::new();

        Self {
            current_page: Page::Table,
            dark_mode: false,
            tcp_ip: "127.0.0.1".to_string(),
            tcp_port: "3490".to_string(),
            connection_status: "Disconnected".to_string(),
            should_connect: false,
            messages: Vec::new(),
            message_id_counter: 0,
            max_messages: 1000000000,
            module_canvas: ModuleCanvas::new(),
            registry: PluginRegistry::new(),
            current_plugin: None,
            ecu_list: Vec::new(),
            ecu_list_view: EcuListView::new(ecu_list.clone()),
            modal_window: None,
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
                    self.process_dlt_messages(data);
                }
            },
            Message::PluginSelected(name) => {
                self.current_plugin = Some(name);
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
                // apply_ecu_updates(&mut self.ecu_list, ecu_updates);
                // self.ecu_list_view.set_ecu_list(self.ecu_list.clone());
            }

            Message::BatchUpdate {
                dlt_messages,
                ecu_updates,
            } => {
                self.process_dlt_messages(dlt_messages);

                // // 2. Apply ECU updates
                // apply_ecu_updates(&mut self.ecu_list, ecu_updates);
                // self.ecu_list_view.set_ecu_list(self.ecu_list.clone());

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
            Message::ModuleCanvasMessage(message) => {
                return self.module_canvas.update(message, &mut self.modal_window);
            },
            Message::OpenSettingsModal => {
                // Open your settings modal here
            },
            Message::CloseSettingsModal => {
                // Close your settings modal here
                self.modal_window = None;
            },
            Message::ModalWindowMessage(content) => {
                if let Some(modal) = &mut self.modal_window {
                    let task: Task<Message>;
                    if let Some(ref_id) = modal.get_id() {                      
                        let widget = self.module_canvas.module_widget.get_mut(&(ref_id as usize));

                        task = modal.update(content.into(), widget);
                    }else{
                        task = modal.update(content.into(), None);
                    }
                    
                    return task;
                }
            },
            
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

    pub fn view(&self) -> Element<'_, Message> {
        let top = top_bar::view(self.dark_mode);
        let nav = navigation::view(self.current_page.clone(), &self.registry, self.dark_mode);

        let main_content = match self.current_page {
            Page::Reports => pages::placeholder::view("Reports", "📋", self.dark_mode),
            Page::ECUSetting => self.ecu_list_view.view(self.dark_mode),
            Page::Settings => pages::settings::view(
                self.dark_mode,
                &self.tcp_ip,
                &self.tcp_port,
                &self.connection_status,
            ),
            Page::Table => pages::table::view(self.dark_mode, &self.messages),
            Page::ChartCanvas => self.module_canvas.view(self.dark_mode),
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

        if let Some(modal) = &self.modal_window {
            let modal_element = modal.draw(self.dark_mode);
            return container(stack![base_view, modal_element])
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        base_view.into()
    }
}

impl Dashboard {
    fn get_context(&self) -> DashboardContext<'_> {
        DashboardContext {
            ecu_list: &self.ecu_list,
            dlt_buffer: &self.messages,
        }
    }

    fn process_dlt_messages(&mut self, mut messages: Vec<DltMessageRow>) {
        if self.messages.len() > self.max_messages {
            let excess = self.messages.len() + messages.len() - self.max_messages;
            self.messages.drain(0..excess);
        }


        for row in &messages {
            let canvas_widgets = self.module_canvas.module_widget.values_mut();
            for widget in canvas_widgets {
                widget.add_new_data(&row.payload);
            }
        }

        for row in &mut messages {
            row.index = self.message_id_counter;
            self.message_id_counter += 1;
            self.messages.push(row.clone());
            // println!("Added DLT message with index {}", row.index);
        }
    }

}
