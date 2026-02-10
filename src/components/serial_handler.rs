use crate::message::Message;
use crate::message::SerialConnectionEvent;
use crate::components::tcp_handler::{parse_dlt_messages, BUFFER_SIZE};

use std::sync::Arc;
use tokio::sync::Mutex;
use iced::Subscription;
use tokio_serial::SerialPortBuilderExt;
use futures::{StreamExt, stream};
use std::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
}

/// Create a serial port connection and return a subscription
pub fn create_serial_subscription(name: String, config: SerialConfig) -> Subscription<Message> {
    struct SubscriptionState {
        id: String,
        config: SerialConfig,
    }
    
    impl std::hash::Hash for SubscriptionState {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
            self.config.hash(state);
        }
    }
    
    Subscription::run_with(
        SubscriptionState {
            id: name.clone(),
            config: config.clone(),
        },
        |state| {
            let config = state.config.clone();
            let id = state.id.clone();
            
            struct ReadState {
                port: Option<tokio_serial::SerialStream>,
                buffer: Vec<u8>,
                id: String,
                config: SerialConfig,
                should_terminate: bool,
                reconnect_attempts: u32,
            }
            
            stream::unfold(
                ReadState {
                    port: None,
                    buffer: Vec::with_capacity(BUFFER_SIZE),
                    id: id.clone(),
                    config: config.clone(),
                    should_terminate: false,
                    reconnect_attempts: 0,
                },
                |mut read_state| async move {
                    if read_state.should_terminate {
                        return None;
                    }
                    
                    // Try to open port if not already open
                    if read_state.port.is_none() {
                        match tokio_serial::new(&read_state.config.port, read_state.config.baud_rate)
                            .open_native_async()
                        {
                            Ok(port) => {
                                println!("Serial port {} opened successfully at {} baud", 
                                    read_state.config.port, read_state.config.baud_rate);
                                read_state.port = Some(port);
                                read_state.reconnect_attempts = 0;
                                
                                return Some((
                                    Message::SerialConnectionEvent(
                                        SerialConnectionEvent::Connected(read_state.id.clone())
                                    ),
                                    read_state
                                ));
                            }
                            Err(e) => {
                                read_state.reconnect_attempts += 1;
                                let error_msg = format!(
                                    "Failed to open serial port {} (attempt {}): {}", 
                                    read_state.config.port, 
                                    read_state.reconnect_attempts,
                                    e
                                );
                                println!("{}", error_msg);
                                
                                // Wait before retry
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                
                                return Some((
                                    Message::SerialConnectionEvent(
                                        SerialConnectionEvent::Error(error_msg)
                                    ),
                                    read_state
                                ));
                            }
                        }
                    }
                    
                    // Read from serial port
                    if let Some(port) = &mut read_state.port {
                        let mut temp_buffer = vec![0u8; BUFFER_SIZE];
                        
                        use tokio::io::AsyncReadExt;
                        let read_result = port.read(&mut temp_buffer).await;
                        
                        match read_result {
                            Ok(0) => {
                                // EOF - serial port disconnected
                                read_state.port = None;
                                Some((
                                    Message::SerialConnectionEvent(
                                        SerialConnectionEvent::Disconnected(read_state.id.clone())
                                    ),
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
                                    ErrorKind::WouldBlock | ErrorKind::Interrupted | ErrorKind::TimedOut
                                );
                                
                                if is_temporary {
                                    // Just wait a bit and try again
                                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                    None
                                } else {
                                    // Fatal error, close port and try to reconnect
                                    println!("Serial port error: {}, will attempt to reconnect", e);
                                    read_state.port = None;
                                    Some((
                                        Message::SerialConnectionEvent(
                                            SerialConnectionEvent::Error(
                                                format!("Serial error: {}", e)
                                            )
                                        ),
                                        read_state
                                    ))
                                }
                            }
                        }
                    } else {
                        // No port, wait and retry connection
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        None
                    }
                }
            )
        }
    )
}
