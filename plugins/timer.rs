// plugins/timer.rs
use crate::{plugin::{DashboardContext, Plugin, PluginMessage}, utility::util::deserialize_message};
use bincode::{Decode, Encode, config, encode_to_vec};
use iced::{widget::{column, text, button, row, scrollable, container}, Element, Task, Length};

pub struct TimerPlugin {
    seconds: u32,
    is_running: bool,
    show_context: bool, // Add toggle for showing context data
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum TimerMessage {
    StartPressed,
    StopPressed,
    ResetPressed,
    ToggleContext, // Add new message for toggling context view
}

impl TimerMessage {
    fn create_custom_message(msg: TimerMessage) -> PluginMessage {
        let data = encode_to_vec(&msg, config::standard()).unwrap();
        PluginMessage::Custom("timer_plugin".to_string(), data)
    }
}

impl Plugin for TimerPlugin {
    fn name(&self) -> &str {
        "Timer"
    }
    
    fn nav_name(&self) -> &str {
        "Timer"
    }
    
    fn new() -> Self {
        Self { 
            seconds: 0,
            is_running: false,
            show_context: false, // Initialize to false
        }
    }
    
    fn update(&mut self, message: PluginMessage, _context: &DashboardContext) -> Task<PluginMessage> {
        match message {
            PluginMessage::Tick(_count) => {
                if self.is_running {
                    self.seconds += 1;
                }
            }
            PluginMessage::Custom(_name, data) => {
                if let Ok(msg) = deserialize_message::<TimerMessage>(&data) {
                    match msg {
                        TimerMessage::StartPressed => {
                            self.is_running = true;
                        }
                        TimerMessage::StopPressed => {
                            self.is_running = false;
                        }
                        TimerMessage::ResetPressed => {
                            self.seconds = 0;
                            self.is_running = false;
                        }
                        TimerMessage::ToggleContext => {
                            self.show_context = !self.show_context;
                        }
                    }
                }
            }
        }
        Task::none()
    }
    
    fn view(&self, context: &DashboardContext) -> Element<'_, PluginMessage> {
        let minutes = self.seconds / 60;
        let seconds = self.seconds % 60;
        
        let timer_section = column![
            text("Timer Example Plugin").size(24),
            text(format!("{:02}:{:02}", minutes, seconds)).size(32),
            row![
                button("Start")
                    .on_press(TimerMessage::create_custom_message(TimerMessage::StartPressed)),
                button("Stop")
                    .on_press(TimerMessage::create_custom_message(TimerMessage::StopPressed)),
                button("Reset")
                    .on_press(TimerMessage::create_custom_message(TimerMessage::ResetPressed)),
                button(if self.show_context { "Hide Context" } else { "Show Context" })
                    .on_press(TimerMessage::create_custom_message(TimerMessage::ToggleContext)),
            ]
            .spacing(10),
        ]
        .spacing(10);
        
        if self.show_context {
            // Display context data
            let context_view = self.build_context_view(context);
            
            column![
                timer_section,
                container(scrollable(context_view))
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .spacing(10)
            .padding(20)
            .into()
        } else {
            timer_section
                .padding(20)
                .into()
        }
    }
}

impl TimerPlugin {
    fn build_context_view(&self, context: &DashboardContext) -> Element<'_, PluginMessage> {
        let mut content = column![
            text("Dashboard Context Data").size(20),
        ].spacing(10);
        
        // Display ECU information
        if context.ecu_list.is_empty() {
            content = content.push(text("No ECU data available"));
        } else {
            content = content.push(text(format!("Total ECUs: {}", context.ecu_list.len())));

            for ecu in context.ecu_list.iter() {
                let ecu_section = column![
                    text(format!("ECU ID: {}", ecu.ecuid)).size(16),
                    text(format!("Description: {}", ecu.description)),
                    text(format!("Total App IDs: {}", ecu.app_ids.len())),
                ]
                .spacing(5)
                .padding(10);
                
                content = content.push(ecu_section);
                
                // Show App IDs for this ECU
                for app in &ecu.app_ids {
                    let app_section = column![
                        text(format!("  App ID: {}", app.apid)),
                        text(format!("  Description: {}", app.description)),
                        text(format!("  Context IDs: {}", app.ctx_ids.len())),
                    ]
                    .spacing(3)
                    .padding(10);
                    
                    content = content.push(app_section);
                    
                    // Show Context IDs for this App
                    for ctx in &app.ctx_ids {
                        let ctx_section = column![
                            text(format!("    Context ID: {}", ctx.context_id)),
                            text(format!("    Description: {}", ctx.description)),
                            text(format!("    Log Level: {}", ctx.log_level)),
                            text(format!("    Trace Status: {}", ctx.trace_status)),
                        ]
                        .spacing(2)
                        .padding(10);
                        
                        content = content.push(ctx_section);
                    }
                }
            }
        }
        
        // Display recent messages
        content = content.push(text("Recent Messages").size(20));
        
        if context.dlt_buffer.is_empty() {
            content = content.push(text("No messages available"));
        } else {
            content = content.push(text(format!("Total Messages: {}", context.dlt_buffer.len())));
            
            // Show last 10 messages
            let messages_to_show = context.dlt_buffer.iter().rev().take(10);

            for msg in messages_to_show {
                let msg_section = column![
                    text(format!("Index: {} | Timestamp: {}", msg.index, msg.timestamp)),
                    text(format!("ECU: {} | App: {} | Context: {}", msg.ecu_id, msg.app_id, msg.context_id)),
                    text(format!("Type: {} | Length: {}", msg.message_type, msg.length)),
                    text(format!("Payload: {}", msg.payload)),
                ]
                .spacing(3)
                .padding(10);
                
                content = content.push(msg_section);
            }
        }
        
        content.into()
    }
}