use crate::message::Message;
use crate::pages::table::DltMessageRow;
use crate::types::{FrontDltAppIdItem, FrontDltCtxIdItem, FrontDltEcuItem};

use std::collections::HashMap;
use std::time::Duration;

use dlt_format_parser::{
    DltFormat, DltParse, LogInfoData, Mtin, ServiceGetLogInfoResponse, ServiceHandler, 
    ServiceParser, ServiceResponse, ServiceResult, ServiceSetLogLevelRequest, 
    ServiceSetTraceStatusRequest, find_dlt_header
};
use iced::Subscription;
use tokio::net::TcpStream;
use tokio::time::sleep;
use crate::message::ConnectionEvent;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

pub fn tcp_connection_subscription(ip: String, port: String) -> Subscription<Message> {
    let subscription_id = format!("tcp-dlt-{}-{}", ip, port);

    Subscription::run_with_id(
        subscription_id,
        futures::stream::unfold(
            ConnectionState::Disconnected,
            move |state| {
                let addr = format!("{}:{}", ip, port);
                async move {
                    let result = handle_connection_state(state, addr.clone()).await;

                    // Always wait before the next iteration, even on failure
                    sleep(Duration::from_secs(1)).await;

                    result
                }
            },
        ),
    )
}

// ============================================================================
// Connection State Handlers
// ============================================================================

async fn handle_connection_state(
    state: ConnectionState,
    addr: String,
) -> Option<(Message, ConnectionState)> {
    match state {
        ConnectionState::Disconnected => handle_disconnected_state(addr).await,
        ConnectionState::Connected { stream, buffer, messages_parsed } => {
            handle_connected_state(stream, buffer, messages_parsed).await
        }
    }
}

async fn handle_disconnected_state(addr: String) -> Option<(Message, ConnectionState)> {
    match TcpStream::connect(&addr).await {
        Ok(stream) => handle_successful_connection(stream, addr).await,
        Err(e) => handle_connection_failure(e, addr).await,
    }
}

async fn handle_successful_connection(
    mut stream: TcpStream,
    addr: String,
) -> Option<(Message, ConnectionState)> {
    println!("Successfully connected to {}", addr);

    // Send get software version request
    let request_get_software_version = 
        dlt_format_parser::dlt_generate_service_get_software_version_request();
    
    if let Err(e) = stream.try_write(&request_get_software_version) {
        let error_msg = format!("Failed to send initial message: {}", e);
        return Some((
            Message::ConnectionEvent(ConnectionEvent::Error(error_msg)),
            ConnectionState::Disconnected
        ));
    }
    
    Some((
        Message::ConnectionEvent(ConnectionEvent::Connected),
        ConnectionState::Connected {
            stream,
            buffer: Vec::new(),
            messages_parsed: 0,
        }
    ))
}

async fn handle_connection_failure(
    error: std::io::Error,
    addr: String,
) -> Option<(Message, ConnectionState)> {
    println!("Connection failed: {}, retrying in {} seconds...", error, RECONNECT_DELAY_SECS);
    sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
    
    Some((
        Message::ConnectionEvent(ConnectionEvent::Error(error.to_string())),
        ConnectionState::Disconnected
    ))
}

// ============================================================================
// Connected State Handlers
// ============================================================================

async fn handle_connected_state(
    mut stream: TcpStream,
    mut buffer: Vec<u8>,
    mut messages_parsed: usize,
) -> Option<(Message, ConnectionState)> {
    let mut temp_buffer = vec![0u8; BUFFER_SIZE];
    
    match stream.read(&mut temp_buffer).await {
        Ok(0) => handle_connection_closed(),
        Ok(n) => handle_data_received(stream, buffer, messages_parsed, &temp_buffer[..n]).await,
        Err(e) => handle_read_error(e),
    }
}

fn handle_connection_closed() -> Option<(Message, ConnectionState)> {
    println!("Connection closed by server");
    Some((
        Message::ConnectionEvent(ConnectionEvent::Disconnected),
        ConnectionState::Disconnected
    ))
}

async fn handle_data_received(
    stream: TcpStream,
    mut buffer: Vec<u8>,
    mut messages_parsed: usize,
    new_data: &[u8],
) -> Option<(Message, ConnectionState)> {
    buffer.extend_from_slice(new_data);
    println!("Received {} bytes, total buffer size: {}", new_data.len(), buffer.len());
    
    let message_to_send = parse_dlt_messages(&mut buffer, &mut messages_parsed);
    let message = message_to_send.unwrap_or(Message::Tick);
    
    println!("Total DLT messages parsed so far: {}", messages_parsed);
    
    Some((
        message,
        ConnectionState::Connected { 
            stream, 
            buffer,
            messages_parsed
        }
    ))
}

fn handle_read_error(error: std::io::Error) -> Option<(Message, ConnectionState)> {
    println!("Read error: {}", error);
    Some((
        Message::ConnectionEvent(ConnectionEvent::Error(error.to_string())),
        ConnectionState::Disconnected
    ))
}

fn parse_dlt_messages(buffer: &mut Vec<u8>, messages_parsed: &mut usize) -> Option<Message> {
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
                                *messages_parsed += 1;
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
                        println!("Found header at offset {} but message incomplete, waiting for more data", current_offset);
                        break;
                    }
                }
                current_offset += package_len as usize;
            }
            None => {
                if current_offset != buffer.len() {
                    println!("No more DLT headers found, stopping parse at offset {}", current_offset);
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
    
    // Return appropriate message based on what we parsed
    match (!parsed_messages.is_empty(), !service_responses.is_empty()) {
        (true, true) => {
            // Both DLT messages and service responses
            Some(Message::BatchUpdate {
                dlt_messages: parsed_messages,
                ecu_updates: service_responses,
            })
        }
        (true, false) => {
            // Only DLT messages
            Some(Message::ConnectionEvent(
                ConnectionEvent::DltMessageReceived(parsed_messages)
            ))
        }
        (false, true) => {
            // Only service responses
            Some(Message::EcuListUpdate(service_responses))
        }
        (false, false) => None,
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