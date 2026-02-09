use crate::message::Message;
use crate::pages::table::DltMessageRow;
use crate::types::{FrontDltAppIdItem, FrontDltCtxIdItem, FrontDltEcuItem};
use crate::components::dlt_parser::{parse_dlt_message, ParsedDltMessage};

use std::collections::HashMap;
use std::ops::Sub;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::time::Duration;

use dlt_protocol::*;
use futures::io::BufReader;
use iced::{Subscription, Task};
use tokio::net::TcpStream;
use tokio::time::sleep;
use crate::message::ConnectionEvent;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use iced::advanced::subscription::{self, Recipe};


enum ConnectionState {
    Disconnected,
    Connected {
        stream: TcpStream,
        buffer: Vec<u8>,
        messages_parsed: usize,
    },
}

// ============================================================================
// Constants
// ============================================================================

const BUFFER_SIZE: usize = 4096*4096*10; // 10 MB buffer
const RECONNECT_DELAY_SECS: u64 = 5;

// ============================================================================
// Public API
// ============================================================================

use futures::{StreamExt, stream}; // for unfold
use std::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub ip: String,
    pub port: String,
    pub serial_port: String,
    pub baud_rate: String,
    pub is_serial: bool,
}

#[derive(Clone)]
pub struct TCPClient {
    pub name: String,
    pub status: bool,
    pub config: ConnectionConfig,
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub buffer: Vec<u8>,
    pub messages_parsed: u32,
}

#[derive(Clone)]
pub struct TCPClientsHandler {
    pub clients: HashMap<String, TCPClient>,
}

impl TCPClientsHandler {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, name: &str, ip: String, port: String) -> Result<(), String> {
        // Check if client already exists
        if self.clients.contains_key(name) {
            return Err("Client already exists".into());
        }
        
        let config = ConnectionConfig { ip, port, 
            serial_port: String::new(), 
            baud_rate: String::new(),
            is_serial: false,
        };
        let client = TCPClient {
            name: name.to_string(),
            status: false,
            config,
            stream: None,
            buffer: Vec::new(),
            messages_parsed: 0,
        };
        self.clients.insert(name.to_string(), client);
        Ok(())
    }

    pub fn set_client_status(&mut self, name: &str, status: bool) {
        if let Some(client) = self.clients.get_mut(name) {
            client.status = status;
        }
    }

    pub fn get_client(&self, name: &str) -> Option<&TCPClient> {
        self.clients.get(name)
    }

    pub fn get_all_clients(&self) -> &HashMap<String, TCPClient> {
        &self.clients
    }

    pub fn remove_client(&mut self, name: &str) {
        self.clients.remove(name);
    }

    pub fn update_client_stream(&mut self, name: &str, stream: Arc<Mutex<TcpStream>>) {
        if let Some(client) = self.clients.get_mut(name) {
            client.stream = Some(stream);
        }
    }
    pub fn try_connect(&mut self, name: &str) -> Task<Message> {
        if let Some(client) = self.clients.get(name) {
            let address = format!("{}:{}", client.config.ip, client.config.port);
            let name = name.to_string();
            
            Task::perform(
                async move {
                    match TcpStream::connect(&address).await {
                        Ok(stream) => {
                            Message::ConnectionEvent(
                                ConnectionEvent::Connected(name, Arc::new(Mutex::new(stream)))
                            )
                        }
                        Err(e) => {
                            Message::ConnectionEvent(ConnectionEvent::Error(e.to_string()))
                        }
                    }
                },
                |msg| msg,
            )
        } else {
            Task::perform(
                async { () },
                |_| Message::ConnectionEvent(ConnectionEvent::Error("Client not found".into()))
            )
        }
    }

    pub fn try_send_by_name(&self, name: &String, data: &[u8]) -> Task<Message> {
        if let Some(client) = self.clients.get(name) {
            if let Some(stream_arc) = &client.stream {
                let data = data.to_vec();
                let stream_clone = Arc::clone(stream_arc);
                tokio::spawn(async move {
                    let mut stream_lock = stream_clone.lock().await;
                    match stream_lock.write_all(&data).await {
                        Ok(_) => Message::Tick,
                        Err(e) => Message::ConnectionEvent(ConnectionEvent::Error(e.to_string())),
                    }
                });
            }
        }
        
        Task::perform(
            async { () },
            |_| Message::ConnectionEvent(ConnectionEvent::Error("Client not connected".into()))
        )
    }

    pub fn create_client_subscription(name: String, stream: Arc<Mutex<TcpStream>>) -> Subscription<Message> {
        struct SubscriptionState {
            id: String,
            stream: Arc<Mutex<TcpStream>>,
        }
        
        impl std::hash::Hash for SubscriptionState {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        
        Subscription::run_with(
            SubscriptionState {
                id: name.clone(),
                stream: stream.clone(),
            },
            |state| {
                let stream = state.stream.clone();
                let id = state.id.clone();
                
                struct ReadState {
                    stream: Arc<Mutex<TcpStream>>,
                    buffer: Vec<u8>,
                    id: String,
                    should_terminate: bool,
                }
                
                stream::unfold(
                    ReadState {
                        stream,
                        buffer: Vec::with_capacity(BUFFER_SIZE),
                        id,
                        should_terminate: false,
                    },
                    |mut read_state| async move {
                        if read_state.should_terminate {
                            return None;
                        }
                        
                        let value = read_state.id.clone();
                        let mut temp_buffer = vec![0u8; BUFFER_SIZE];
                        
                        let read_result = {
                            let mut lock = read_state.stream.lock().await;
                            lock.read(&mut temp_buffer).await
                        };
                        
                        match read_result {
                            Ok(0) => {
                                read_state.should_terminate = true;
                                Some((
                                    Message::ConnectionEvent(ConnectionEvent::Disconnected(value)),
                                    read_state
                                ))
                            }
                            Ok(n) => {
                                read_state.buffer.extend_from_slice(&temp_buffer[..n]);
                                let message = parse_dlt_messages(&mut read_state.buffer);
                                // sleep(Duration::from_secs(1)).await;
                                Some((message, read_state))
                            }
                            Err(e) => {
                                use std::io::ErrorKind;
                                
                                let is_temporary = matches!(
                                    e.kind(),
                                    ErrorKind::WouldBlock | ErrorKind::Interrupted
                                );
                                
                                if is_temporary {
                                    None
                                } else {
                                    read_state.should_terminate = true;
                                    Some((
                                        Message::ConnectionEvent(ConnectionEvent::Error(
                                            format!("Fatal error: {}", e)
                                        )),
                                        read_state
                                    ))
                                }
                            }
                        }
                    }
                )
            }
        )
    }
}

fn parse_dlt_messages(buffer: &mut Vec<u8>) -> Message {
    let mut parsed_messages = Vec::new();
    let mut service_responses = Vec::new();
    let mut current_offset = 0;
    
    loop {
        let parse_buffer = &buffer[current_offset..];
        
        // Try to parse DLT message
        match parse_dlt_message(parse_buffer) {
            Ok((parsed_msg, remaining)) => {
                let package_len = parsed_msg.raw_bytes.len();
                
                // Check message type from extended header
                let msin = parsed_msg.extended_header.msin;
                let mstp = (msin >> 1) & 0x07; // Extract MSTP (bits 1-3)
                
                let mstp_type = MstpType::parse(mstp);
                
                match mstp_type {
                    MstpType::DltTypeLog => {
                        // Successfully parsed a log message
                        let dlt_message_row = DltMessageRow::from_parsed_message(&parsed_msg);
                        parsed_messages.push(dlt_message_row);
                    }

                    MstpType::DltTypeControl => {
                        println!("Control message received (service messages not yet implemented)");
                        // TODO: Implement service message handling
                        // For now, just create a basic message row
                        let dlt_message_row = DltMessageRow::from_parsed_message(&parsed_msg);
                        parsed_messages.push(dlt_message_row);
                    }

                    _ => {
                        println!("Unknown MSTP message type: {:?}", mstp_type);
                    }
                }
                
                current_offset += package_len;
                if current_offset >= buffer.len() {
                    current_offset = 0;
                    buffer.clear();
                    break;
                }
            }
            Err(_) => {
                if current_offset == 0 {
                    println!("Failed to parse DLT message at offset {}, waiting for more data", current_offset);
                    // The message is incomplete. Wait for more data
                    println!("Buffer has {} bytes, first bytes: {:?}", 
                        buffer.len(), 
                        &buffer[..std::cmp::min(50, buffer.len())]);
                } else {
                    println!("Failed to parse DLT message at offset {}, discarding up to this point", current_offset);
                }
                break;
            }
        }
    }
    
    // Remove all successfully parsed data from the buffer
    if current_offset > 0 {
        buffer.drain(0..current_offset);
        println!("Removed {} bytes from buffer, {} bytes remaining", current_offset, buffer.len());
    }

    Message::BatchUpdate {
        dlt_messages: parsed_messages,
        ecu_updates: service_responses,
    }
}

// ============================================================================
// Service Message Extraction - TODO: Re-implement with dlt-protocol
// ============================================================================

#[derive(Debug, Clone)]
pub struct EcuUpdateInfo {
    pub ecu_id: String,
    pub app_updates: Vec<AppUpdateInfo>,
    pub software_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppUpdateInfo {
    pub app_info: FrontDltAppIdItem,
    pub merge_with_existing: bool,
}

/*
// TODO: Re-implement service message handling with dlt-protocol
fn extract_service_info(
    service_msg: ServiceMessage,
    dlt_header_info: DltMessageRow,
) -> Result<EcuUpdateInfo, String> {
    // Service message handling needs to be re-implemented
    Err("Service messages not yet implemented".to_string())
}
*/

// ============================================================================
// Helper Functions for ECU List Management
// ============================================================================

/// Apply ECU updates to the ecu_list in your dashboard app
pub fn apply_ecu_updates(
    ecu_list: &mut Vec<FrontDltEcuItem>,
    updates: Vec<EcuUpdateInfo>,
) {
    for update in updates {
        apply_single_ecu_update(ecu_list, update);
    }
}

fn apply_single_ecu_update(
    ecu_list: &mut Vec<FrontDltEcuItem>,
    update: EcuUpdateInfo,
) {
    // Find or create ECU entry
    let ecu_entry = ecu_list
        .iter_mut()
        .find(|ecu| ecu.ecuid == update.ecu_id);
    
    match ecu_entry {
        Some(ecu) => {
            // ECU exists, update it
            if let Some(version) = update.software_version {
                // Store software version if your FrontDltEcuItem has that field
                println!("Updated software version for {}: {}", update.ecu_id, version);
            }
            
            for app_update in update.app_updates {
                apply_app_update(ecu, app_update);
            }
        }
        None => {
            // ECU doesn't exist, create it
            let mut new_ecu = FrontDltEcuItem {
                ecuid: update.ecu_id.clone(),
                app_ids: Vec::new(),
                description: String::new(),
            };
            
            for app_update in update.app_updates {
                new_ecu.app_ids.push(app_update.app_info);
            }
            
            ecu_list.push(new_ecu);
        }
    }
}

fn apply_app_update(ecu: &mut FrontDltEcuItem, app_update: AppUpdateInfo) {
    let app_id = &app_update.app_info.apid;
    
    // Check if app already exists
    if let Some(existing_app) = ecu.app_ids.iter_mut().find(|app| &app.apid == app_id) {
        if app_update.merge_with_existing {
            // Merge contexts
            merge_contexts(existing_app, app_update.app_info);
        } else {
            // Replace entirely
            *existing_app = app_update.app_info;
        }
    } else {
        // App doesn't exist, add it
        ecu.app_ids.push(app_update.app_info);
    }
}

fn merge_contexts(existing_app: &mut FrontDltAppIdItem, new_app: FrontDltAppIdItem) {
    println!("Merging contexts for app {}", existing_app.apid);
    
    // Create a set of existing context IDs for efficient lookup
    let existing_ctx_ids: std::collections::HashSet<String> = 
        existing_app.ctx_ids.iter().map(|c| c.context_id.clone()).collect();
    
    // Add only new contexts that don't already exist
    for new_ctx in new_app.ctx_ids {
        if !existing_ctx_ids.contains(&new_ctx.context_id) {
            println!("  Adding new context: {}", new_ctx.context_id);
            existing_app.ctx_ids.push(new_ctx);
        } else {
            // Update existing context with new values
            if let Some(existing_ctx) = existing_app
                .ctx_ids
                .iter_mut()
                .find(|c| c.context_id == new_ctx.context_id) 
            {
                existing_ctx.log_level = new_ctx.log_level;
                existing_ctx.trace_status = new_ctx.trace_status;
                if !new_ctx.description.is_empty() {
                    existing_ctx.description = new_ctx.description;
                }
                println!("  Updated existing context: {}", existing_ctx.context_id);
            }
        }
    }
    
    // Update description if new one is provided
    if !new_app.description.is_empty() {
        existing_app.description = new_app.description;
    }
}