/// DLT Register Pattern Test Example
/// 
/// This example sends DLT messages with register patterns that the viewer can parse
/// and display in the Register widget.
/// 
/// Usage:
/// Terminal 1: cargo run --example dlt_test_server
/// Terminal 2: cargo run
/// In the viewer:
///   1. Connect to 127.0.0.1:3490
///   2. Right-click on canvas
///   3. Select "Add Register"
///   4. Watch as register values appear!

use dlt_protocol::*;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DLT Register Pattern Test Server ===");
    println!("Starting server...");
    
    let addr: SocketAddr = "127.0.0.1:3490".parse()?;
    let listener = TcpListener::bind(&addr).await?;
    
    println!("✓ Listening on {}", addr);
    println!("\nConnect your DLT viewer to this address.");
    println!("Press Ctrl+C to stop.\n");
    
    loop {
        let (socket, client_addr) = listener.accept().await?;
        println!("  Client connected: {}", client_addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let ecu_id = b"ECU1";
    let app_id = b"REGS";
    let ctx_id = b"MON1";
    
    let mut counter: u32 = 0;
    let mut status_reg: u32 = 0x0000;
    
    loop {
        counter += 1;
        
        // Simulate changing register values
        status_reg = (status_reg + 0x0100) & 0xFFFF;
        let control_reg = 0x8000 | (counter % 256);
        let version_reg = 0x0123;
        let error_reg = if counter % 10 == 0 { 0x0001 } else { 0x0000 };
        
        // Send multiple register updates in one message
        send_register_message(
            &mut socket,
            ecu_id,
            app_id,
            ctx_id,
            counter,
            &format!(
                "#REG: STATUS_REG: 0x{:04X} #REG: CONTROL_REG: 0x{:04X} #REG: VERSION_REG: 0x{:04X} #REG: ERROR_REG: 0x{:04X}",
                status_reg, control_reg, version_reg, error_reg
            ),
        )
        .await?;
        
        // Also send decimal register values
        send_register_message(
            &mut socket,
            ecu_id,
            app_id,
            ctx_id,
            counter + 1000000,
            &format!(
                "#REG: TEMP_SENSOR: {} #REG: VOLTAGE: {} #REG: CURRENT: {}",
                23 + (counter % 10),
                3300 + (counter % 50),
                150 + (counter % 20)
            ),
        )
        .await?;
        
        if counter % 10 == 0 {
            println!("  Sent {} register updates...", counter * 2);
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}

async fn send_register_message(
    socket: &mut TcpStream,
    ecu_id: &[u8; 4],
    app_id: &[u8; 4],
    ctx_id: &[u8; 4],
    timestamp: u32,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let builder = DltMessageBuilder::new();
    let mut builder = builder
        .with_ecu_id(ecu_id)
        .with_app_id(app_id)
        .with_context_id(ctx_id)
        .with_session_id(1)
        .with_timestamp(timestamp);
    
    let mut payload_buffer = [0u8; 512];
    let payload_len = {
        let mut payload_builder = PayloadBuilder::new(&mut payload_buffer);
        
        if let Err(e) = payload_builder.add_string(message) {
            eprintln!("Failed to add string: {:?}", e);
            return Ok(());
        }
        
        payload_builder.len()
    };
    
    let mut dlt_buffer = [0u8; 1024];
    match builder.generate_log_message_with_payload(
        &mut dlt_buffer,
        &payload_buffer[..payload_len],
        MtinTypeDltLog::DltLogInfo,
        1, // number of arguments
        true, // verbose mode
    ) {
        Ok(total_size) => {
            socket.write_all(&dlt_buffer[..total_size]).await?;
        }
        Err(e) => {
            eprintln!("✗ Failed to generate DLT message: {:?}", e);
        }
    }
    
    Ok(())
}
