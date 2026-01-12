use crate::components::tcp_handler::{TCPClientsHandler, apply_ecu_updates};
use crate::components::{navigation, top_bar};
use crate::message::{Message, Page};
use crate::module_view::ModuleCanvas;
use crate::modal_window::modal_window::*;
use crate::module_view::canvas::ModuleCanvasMessage;
use crate::pages::ecu_setting::{EcuListView, EcuSelection};
use crate::pages::{self};
use crate::plugin::{DashboardContext, PluginMessage};
use crate::plugin_registry::PluginRegistry;
use crate::types::FrontDltEcuItem;
use crate::message::ConnectionEvent;
use crate::ui::footer_bar;
use dlt_format_parser::service_generate;
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
    pub tcp_client_name: String,
    pub tcp_ip: String,
    pub tcp_port: String,
    pub connection_status: String,
    pub messages: Vec<DltMessageRow>,
    pub message_id_counter: u32,
    pub max_messages: usize,
    pub module_canvas: ModuleCanvas,
    pub registry: PluginRegistry,
    pub ecu_list: Vec<FrontDltEcuItem>,
    pub ecu_list_view: EcuListView,
    pub modal_window: Option<Box<dyn ModalWindowView>>,
    pub dlt_table_scroll_offset: f32,
    pub tcp_clients: TCPClientsHandler
}

impl Default for Dashboard {
    fn default() -> Self {
        // Initialize with some example DLT data (optional)
        let ecu_list = Vec::new();

        Self {
            current_page: Page::Overview,
            dark_mode: false,
            tcp_client_name: "main_tcp_client".to_string(),
            tcp_ip: "127.0.0.1".to_string(),
            tcp_port: "3490".to_string(),
            connection_status: "Disconnected".to_string(),
            messages: Vec::new(),
            message_id_counter: 0,
            max_messages: 1000000000,
            module_canvas: ModuleCanvas::new(),
            registry: PluginRegistry::new(),
            ecu_list: Vec::new(),
            ecu_list_view: EcuListView::new(ecu_list.clone()),
            modal_window: None,
            dlt_table_scroll_offset: 0.0,
            tcp_clients: TCPClientsHandler::new(),
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                self.module_canvas.update(ModuleCanvasMessage::ToggleTheme(self.dark_mode), &mut self.modal_window);
            }
            Message::NavigateTo(page) => self.current_page = page,
            Message::Tick => {
                // Clone the data so the context does not borrow `self` immutably while
                // `self.registry.update_all` needs a mutable borrow of `self.registry`.
                let ecu_list_clone = self.ecu_list.clone();
                let dlt_buffer_clone = self.messages.clone();
                let context = DashboardContext {
                    ecu_list: &ecu_list_clone,
                    dlt_buffer: &dlt_buffer_clone,
                };
                self.registry.update_all(PluginMessage::Tick(0), &context);
            }
            Message::TcpClientNameChanged(name) => self.tcp_client_name = name,
            Message::TcpIpChanged(ip) => self.tcp_ip = ip,
            Message::TcpPortChanged(port) => self.tcp_port = port,
            Message::ConnectTcp => {
                let _ip = self.tcp_ip.clone();
                let _port = self.tcp_port.clone();
                // self.should_connect = true;

                if self.tcp_clients.add_client(&self.tcp_client_name, _ip, _port).is_err() {
                    self.modal_window = Some(Box::new(crate::modal_window::confirm_modal_window::ConfirmModal::new(
                        "Error".to_string(),
                        format!("TCP Client with name '{}' already exists.", self.tcp_client_name),
                    )));
                    return Task::none();
                }

                return self.tcp_clients.try_connect(&self.tcp_client_name);
            }
            Message::ClearMessages => {
                self.messages.clear();
            }
            Message::ScrollChanged(viewport) => {
                self.dlt_table_scroll_offset = viewport.absolute_offset().y;
            }
            Message::ConnectionEvent(event) => match event {
                ConnectionEvent::Connected(name, stream) => {
                    self.connection_status = "Connected".to_string();
                    println!("TCP Client '{}' connected.", name);

                    // Send version info request
                    self.tcp_clients.try_send_by_name(&name, service_generate::dlt_generate_service_get_software_version_request().as_slice());

                    self.tcp_clients.update_client_stream(&name, stream);
                    self.tcp_clients.set_client_status(&name, true);
                }
                ConnectionEvent::Disconnected(name) => {
                    println!("TCP Client '{}' disconnected.", name);
                    self.tcp_clients.remove_client(&name);
                }
                ConnectionEvent::Error(err) => {
                    println!("Connection error: {}", err);
                    self.connection_status = format!("Error: {}", err);
                }
            },
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
            Message::BatchUpdate {
                dlt_messages,
                ecu_updates,
            } => {
                self.process_dlt_messages(dlt_messages);
                apply_ecu_updates(&mut self.ecu_list, ecu_updates);
                self.ecu_list_view.set_ecu_list(self.ecu_list.clone());
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

        let mut tcp_subscription: Subscription<Message> = Subscription::none();

        // Register subscriptions for all active TCP clients
        for (name, client) in self.tcp_clients.get_all_clients() {
            if client.status == true {
                if let Some(stream) = &client.stream {
                    tcp_subscription = TCPClientsHandler::create_client_subscription(
                        name.clone(),
                        stream.clone(),
                    );
                }
            }
        }

        Subscription::batch(vec![tick_subscription, tcp_subscription])
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
            Page::Overview => pages::overview::view( &self.tcp_clients, 
                                                            &self.tcp_client_name,
                                                          &self.tcp_ip, 
                                                                        &self.tcp_port),
            Page::ECUSetting => self.ecu_list_view.view(self.dark_mode),
            Page::Settings => pages::settings::view(
                self.dark_mode,
                &self.tcp_ip,
                &self.tcp_port,
                &self.connection_status,
            ),
            Page::Table => pages::table::view(self.dark_mode, &self.messages, self.dlt_table_scroll_offset),
            Page::ChartCanvas => self.module_canvas.view(),
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

        let footer = footer_bar::view(
            &self.connection_status,
            self.messages.len(),
            self.dark_mode
        );

        let main_layout = column![
            top,
            row![nav, content_area].height(Length::Fill),
            footer
        ];

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
    pub fn new() -> Self {
        Self::default()
    }

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
                widget.add_new_data(&row);
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
