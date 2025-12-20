use crate::message::Message;
use crate::pages::table::DltMessageRow;
use crate::types::{FrontDltAppIdItem, FrontDltCtxIdItem, FrontDltEcuItem};

use std::collections::HashMap;
use std::ops::Sub;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::time::Duration;

use bincode::de;
use dlt_format_parser::{
    DltFormat, DltParse, LogInfoData, Mtin, ServiceGetLogInfoResponse, ServiceHandler, 
    ServiceParser, ServiceResponse, ServiceResult, ServiceSetLogLevelRequest, 
    ServiceSetTraceStatusRequest, find_dlt_header
};
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

    pub fn add_client(&mut self, name: String, ip: String, port: String) {
        let config = ConnectionConfig { ip, port };
        let client = TCPClient {
            name: name.clone(),
            status: false,
            config,
            stream: None,
            buffer: Vec::new(),
            messages_parsed: 0,
        };
        self.clients.insert(name, client);
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
        match find_dlt_header(buffer, current_offset) {
            Some(package_len) => {                
                // Try to parse the DLT message at this offset
                let parse_buffer = &buffer[current_offset..];
                match parse_buffer.dlt_parse() {
                    Ok((dlt_format, remaining)) => {
                        match dlt_format.extended_header.parse().2 {
                            Mtin::Log(_) => {
                                // Successfully parsed a message
                                let dlt_message_row = DltMessageRow::from_dlt_format(&dlt_format);
                                parsed_messages.push(dlt_message_row);
                            }

                            Mtin::Control(_) => {
                                println!("Control message received");

                                let dlt_header_info = DltMessageRow::from_dlt_format(&dlt_format);
                                
                                let mut parser = ServiceParser::new();
                                if let Ok(service_msg) = parser.parse_raw_message(&dlt_format.payload) {
                                    println!("Service ID: {}", service_msg.service_id);
                                    
                                    // Extract ECU information
                                    if let Ok(ecu_updates) = extract_service_info(service_msg, dlt_header_info) {
                                        service_responses.push(ecu_updates);
                                    }
                                }
                            }

                            _ => {
                                println!("Unknown MTIN message");
                            }
                        }
                    }
                    Err(_) => {
                        if current_offset == 0 {
                            println!("Failed to parse DLT message at offset {}, discarding entire buffer", current_offset);
                            buffer.clear();
                        } else {
                            println!("Failed to parse DLT message at offset {}, discarding up to next header", current_offset);
                        }
                    }
                }
                current_offset += package_len as usize;
                if current_offset >= buffer.len() {
                    current_offset = 0;
                    buffer.clear();
                    break;
                }
            }
            None => {
                // The message is incomplete. Wait for more data
                println!("No valid DLT header found, waiting for more data {} bytes in buffer, {}",
                    buffer.len(), current_offset);
                //debug first 50 bytes
                println!("Buffer snapshot: {:?}", &buffer[current_offset..std::cmp::min(current_offset + 50, buffer.len())]);
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
// Service Message Extraction
// ============================================================================

fn extract_service_info(
    service_msg: dlt_format_parser::ServiceMessage,
    dlt_header_info: DltMessageRow,
) -> Result<EcuUpdateInfo, String> {
    let mut handler = ServiceInfoExtractor::new(dlt_header_info);
    let mut parser = ServiceParser::new();
    
    parser.handle_message(&mut handler, service_msg)
        .map_err(|e| format!("Failed to handle service message: {:?}", e))?;
    
    handler.extracted_info.ok_or_else(|| "No info extracted".to_string())
}

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

struct ServiceInfoExtractor {
    extracted_info: Option<EcuUpdateInfo>,
    dlt_header_info: Option<DltMessageRow>,
}

impl ServiceInfoExtractor {
    fn new(dlt_header_info: DltMessageRow) -> Self {
        Self {
            extracted_info: None,
            dlt_header_info: Some(dlt_header_info),
        }
    }
}

impl ServiceHandler for ServiceInfoExtractor {
    fn handle_set_log_level(
        &mut self, 
        request: ServiceSetLogLevelRequest
    ) -> ServiceResult<ServiceResponse> {
        println!(
            "Setting log level to {} for APID: {:?}, CTID: {:?}", 
            request.new_log_level, 
            std::str::from_utf8(&request.apid).unwrap_or("invalid"),
            std::str::from_utf8(&request.ctid).unwrap_or("invalid")
        );
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_set_trace_status(
        &mut self, 
        request: ServiceSetTraceStatusRequest
    ) -> ServiceResult<ServiceResponse> {
        println!("Set trace status - not implemented");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_get_log_info(
        &mut self, 
        response: ServiceGetLogInfoResponse
    ) -> ServiceResult<ServiceResponse> {
        println!("Processing get log info response");
        
        let mut app_updates = Vec::new();
        
        match &response.log_info_data {
            LogInfoData::ApplicationIds(apps) => {
                for app_data in apps {
                    let app_id = String::from_utf8_lossy(&app_data.app_id)
                        .trim_end_matches('\0')
                        .to_string();
                    
                    println!("App ID: {:?}", &app_id);
                    println!("Context Count: {}", app_data.context_id_count);
                    
                    // Convert context list to FrontDltCtxIdItem vector
                    let contexts: Vec<FrontDltCtxIdItem> = app_data
                        .context_id_list
                        .iter()
                        .map(|ctx| {
                            println!(
                                "  Context ID: {:?}",
                                std::str::from_utf8(&ctx.context_id).unwrap_or("invalid")
                            );
                            println!("  Log Level: {}", ctx.log_level);
                            println!("  Trace Status: {}", ctx.trace_status);
                            
                            if let Some(desc) = &ctx.context_description {
                                if let Ok(desc_str) = std::str::from_utf8(desc) {
                                    println!("  Description: {}", desc_str);
                                }
                            }
                            
                            FrontDltCtxIdItem {
                                context_id: String::from_utf8_lossy(&ctx.context_id).to_string(),
                                log_level: ctx.log_level,
                                trace_status: ctx.trace_status,
                                description: ctx
                                    .context_description
                                    .as_ref()
                                    .and_then(|d| std::str::from_utf8(d).ok())
                                    .unwrap_or("")
                                    .to_string(),
                            }
                        })
                        .collect();
                    
                    let app_description = app_data
                        .app_description
                        .as_ref()
                        .and_then(|d| std::str::from_utf8(d).ok())
                        .unwrap_or("")
                        .to_string();
                    
                    if !app_description.is_empty() {
                        println!("App Description: {}", app_description);
                    }
                    
                    let app_info = FrontDltAppIdItem {
                        apid: app_id,
                        description: app_description,
                        ctx_ids: contexts,
                    };
                    
                    app_updates.push(AppUpdateInfo {
                        app_info,
                        merge_with_existing: true,
                    });
                }
            }
            _ => {
                println!("Unsupported log info data type");
            }
        }
        
        if !app_updates.is_empty() {
            self.extracted_info = Some(EcuUpdateInfo {
                ecu_id: "ECU1".to_string(), // You might want to extract this from the message
                app_updates,
                software_version: None,
            });
        }
        
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_get_software_version(
        &mut self, 
        version: &String
    ) -> ServiceResult<ServiceResponse> {
        println!("Software Version: {}", version);
        
        self.extracted_info = Some(EcuUpdateInfo {
            ecu_id: "ECU1".to_string(),
            app_updates: Vec::new(),
            software_version: Some(version.clone()),
        });
        
        Ok(ServiceResponse::success(vec![]))
    }
    
    fn handle_store_configuration(&mut self) -> ServiceResult<ServiceResponse> {
        println!("Storing configuration");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_swc_injection(
        &mut self, 
        service_id: u32, 
        payload: &[u8]
    ) -> ServiceResult<ServiceResponse> {
        println!("SWC Injection");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_unknown_service(
        &mut self, 
        service_id: u32, 
        payload: &[u8]
    ) -> ServiceResult<ServiceResponse> {
        println!("Unknown service: {}", service_id);
        Ok(ServiceResponse::success(vec![]))
    }
}

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