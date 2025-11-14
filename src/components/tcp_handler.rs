use crate::message::Message;
use crate::pages::table::DltMessageRow;
use crate::types::{FrontDltAppIdItem, FrontDltCtxIdItem, FrontDltEcuItem};
use crate::components::dlt_data_manager::DLT_ECU_CONTEXT_STORE;
use crate::components::dlt_data_manager::{analzye_dlt_data_regex, DLT_DATA_REGEX_STORE};

use std::collections::HashMap;
use std::time::Duration;

use dlt_format_parser::{DltFormat, DltParse, LogInfoData, Mtin, ServiceGetLogInfoResponse, ServiceHandler, ServiceParser, ServiceResponse, ServiceResult, ServiceSetLogLevelRequest, ServiceSetTraceStatusRequest, find_next_dlt_header};
use iced::{Subscription, window::raw_window_handle::AppKitDisplayHandle};
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

const BUFFER_SIZE: usize = 4096;
const RECONNECT_DELAY_SECS: u64 = 5;
const INITIAL_MESSAGE: &[u8] = b"Hello from client";

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
    let request_get_software_version = dlt_format_parser::dlt_generate_service_get_software_version_request();
    
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
    mut buffer: Vec<u8>,  // Fixed: consistent naming
    mut messages_parsed: usize,  // Fixed: consistent naming
    new_data: &[u8],
) -> Option<(Message, ConnectionState)> {
    buffer.extend_from_slice(new_data);  // Fixed: removed extra underscores
    println!("Received {} bytes, total buffer size: {}", new_data.len(), buffer.len());
    let message_to_send = parse_dlt_messages(&mut buffer, &mut messages_parsed);
    let message = message_to_send.unwrap_or(Message::Tick);
    println!("Total DLT messages parsed so far: {}", messages_parsed);
    Some((
        message,
        ConnectionState::Connected { 
            stream, 
            buffer,           // Fixed: consistent naming
            messages_parsed   // Fixed: consistent naming
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
    let mut current_offset = 0;
    
    loop {
        // Try to find the next valid DLT header
        match find_next_dlt_header(buffer, current_offset) {
            Some(header_offset) => {
                // If header is not at the beginning, we have corrupt/incomplete data before it
                if header_offset > current_offset {
                    break;
                }
                
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

                                let mut handler = MyServiceHandler;
                                let mut parser = ServiceParser::new();
                                let service_msg = parser.parse_raw_message(&dlt_format.payload).unwrap();
                                println!("ID {}", service_msg.service_id);
                                let result = parser.handle_message(&mut handler, service_msg);
                            }

                            _ => {
                                // Unknown or unsupported message type
                                println!("Unknown MTIN message");
                            }
                        }
                        
                        
                        // Calculate how many bytes we consumed
                        let bytes_consumed = parse_buffer.len() - remaining.len();
                        current_offset += bytes_consumed;
                        println!("Parsed DLT message #{}, {} bytes", *messages_parsed, bytes_consumed);
                    }
                    Err(_) => {
                        // We found a header but couldn't parse the full message
                        // This likely means we don't have all the data yet
                        println!("Found header at offset {} but message incomplete, waiting for more data", current_offset);
                        break;
                    }
                }
            }
            None => {
                // No more valid headers found in the remaining buffer
                if current_offset == 0 && buffer.len() > 0 {
                    // No valid header found at all - this might be incomplete data
                    println!("No valid DLT header found in {} bytes, keeping buffer for next read", buffer.len());
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
    
    // Return the parsed messages wrapped in your Message type
    if !parsed_messages.is_empty() {
        Some(Message::ConnectionEvent(ConnectionEvent::DltMessageReceived(parsed_messages)))
    } else {
        None
    }
}


struct MyServiceHandler;

impl ServiceHandler for MyServiceHandler {
    fn handle_set_log_level(&mut self, request: ServiceSetLogLevelRequest) -> ServiceResult<ServiceResponse> {
        println!("Setting log level to {} for APID: {:?}, CTID: {:?}", 
                request.new_log_level, 
                std::str::from_utf8(&request.apid).unwrap_or("invalid"),
                std::str::from_utf8(&request.ctid).unwrap_or("invalid"));
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_set_trace_status(&mut self, request: ServiceSetTraceStatusRequest) -> ServiceResult<ServiceResponse> {
        println!("not implemented");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_get_log_info(&mut self, response: ServiceGetLogInfoResponse) -> ServiceResult<ServiceResponse> {
        println!("Get log info");
        match &response.log_info_data {
            LogInfoData::ApplicationIds(apps) => {
                for n in apps {
                    let app_id = String::from_utf8_lossy(&n.app_id)
                        .trim_end_matches('\0')
                        .to_string();
                    
                    println!("App ID: {:?}", &app_id);
                    println!("Context Count: {}", n.context_id_count);
                    
                    // Convert context list to FrontDltCtxIdItem vector
                    let new_contexts: Vec<FrontDltCtxIdItem> = n
                        .context_id_list
                        .iter()
                        .map(|ctx| {
                            println!(
                                "  Context ID: {:?}",
                                std::str::from_utf8(&ctx.context_id).unwrap_or("invalid")
                            );
                            println!("  Hex Context ID: {:?}", ctx.context_id);
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
                    
                    if let Some(app_desc) = &n.app_description {
                        if let Ok(app_desc_str) = std::str::from_utf8(app_desc) {
                            println!("App Description: {}", app_desc_str);
                        }
                    }
                    
                    // Check if app ID already exists and update or create accordingly
                    if let Ok(existing_app_info) = get_app_info("ECU1", &app_id) {
                        // App ID exists - merge contexts
                        println!("App ID {} already exists, merging contexts", app_id);
                        
                        // Create a set of existing context IDs for efficient lookup
                        let mut merged_contexts = existing_app_info.ctx_ids.clone();
                        let existing_ctx_ids: std::collections::HashSet<String> = 
                            merged_contexts.iter().map(|c| c.context_id.clone()).collect();
                        
                        // Add only new contexts that don't already exist
                        for new_ctx in new_contexts {
                            if !existing_ctx_ids.contains(&new_ctx.context_id) {
                                println!("  Adding new context: {}", new_ctx.context_id);
                                merged_contexts.push(new_ctx);
                            } else {
                                // Optionally update existing context with new values
                                if let Some(existing_ctx) = merged_contexts
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
                        
                        let updated_app_info = FrontDltAppIdItem {
                            apid: app_id.clone(),
                            description: n
                                .app_description
                                .as_ref()
                                .and_then(|d| std::str::from_utf8(d).ok())
                                .unwrap_or(&existing_app_info.description)
                                .to_string(),
                            ctx_ids: merged_contexts,
                        };
                        
                        if let Err(e) = update_app_info("ECU1", updated_app_info) {
                            println!("Error updating app info: {}", e);
                        }
                    } else {
                        // App ID doesn't exist - create new
                        println!("Creating new app ID: {}", app_id);
                        
                        let app_info = FrontDltAppIdItem {
                            apid: app_id,
                            description: n
                                .app_description
                                .as_ref()
                                .and_then(|d| std::str::from_utf8(d).ok())
                                .unwrap_or("")
                                .to_string(),
                            ctx_ids: new_contexts,
                        };
                        
                        if let Err(e) = add_app_info("ECU1", app_info) {
                            println!("Error storing app info: {}", e);
                        }
                    }
                }
            }
            _ => {
                println!("Unsupported log info data type");
            }
        }
        Ok(ServiceResponse::success(vec![]))
    }

    
    fn handle_get_software_version(&mut self, version: &String) -> ServiceResult<ServiceResponse> {
        println!("Getting software version");
        println!("Software Version: {}", version);
        Ok(ServiceResponse::success(vec![]))
    }
    
    fn handle_store_configuration(&mut self) -> ServiceResult<ServiceResponse> {
        println!("Storing configuration");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_swc_injection(&mut self, service_id: u32, payload: &[u8]) -> ServiceResult<ServiceResponse> {
        println!("SWC Injection");
        Ok(ServiceResponse::success(vec![]))
    }

    fn handle_unknown_service(&mut self, service_id: u32, payload: &[u8]) -> ServiceResult<ServiceResponse> {
        println!("Unknown service");
        Ok(ServiceResponse::success(vec![]))
    }
}

fn get_app_info(ecu_id: &str, app_id: &str) -> Result<FrontDltAppIdItem, String> {
    // If using Arc<Mutex<HashMap>>
    let storage = DLT_ECU_CONTEXT_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    storage
        .iter().find(|ecu| ecu.ecuid == ecu_id)
        .and_then(|ecu_apps| ecu_apps.app_ids.iter().find(|app| app.apid == app_id))
        .cloned()
        .ok_or_else(|| format!("App ID {} not found for ECU {}", app_id, ecu_id))
}

fn update_app_info(ecu_id: &str, app_info: FrontDltAppIdItem) -> Result<(), String> {
    let mut storage = DLT_ECU_CONTEXT_STORE.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    let ecu_apps = storage
        .iter_mut()  // Changed from .iter() to .iter_mut()
        .find(|ecu| ecu.ecuid == ecu_id)
        .ok_or_else(|| format!("ECU {} not found", ecu_id))?
        .app_ids
        .iter_mut()  // Changed from .iter() to .iter_mut()
        .find(|app| app.apid == app_info.apid)
        .ok_or_else(|| format!("App ID {} not found for ECU {}", app_info.apid, ecu_id))?;
    
    // Update the app info
    *ecu_apps = app_info;
    
    Ok(())
}

fn add_app_info(ecu_id: &str, app_info: FrontDltAppIdItem) -> Result<(), String> {
    let mut storage = DLT_ECU_CONTEXT_STORE.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    // Find the ECU in the Vec
    let ecu = storage
        .iter_mut()
        .find(|ecu| ecu.ecuid == ecu_id)
        .ok_or_else(|| format!("ECU {} not found", ecu_id))?;
    
    // Check if app already exists
    if ecu.app_ids.iter().any(|app| app.apid == app_info.apid) {
        return Err(format!("App ID {} already exists for ECU {}", app_info.apid, ecu_id));
    }
    
    // Add the new app
    ecu.app_ids.push(app_info);
    
    Ok(())
}