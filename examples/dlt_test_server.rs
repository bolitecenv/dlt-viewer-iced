// examples/dlt_test_server.rs
// Test DLT server that sends DLT messages over TCP for testing the viewer
//
// Run with: cargo run --example dlt_test_server
// Then connect the viewer to 127.0.0.1:3490

use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use dlt_protocol::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== DLT Test Server ===");
    println!("Starting DLT test server...");
    
    let listener = TcpListener::bind("127.0.0.1:3490").await?;
    println!("✓ Listening on 127.0.0.1:3490");
    println!("\nConnect your DLT viewer to this address.");
    println!("Press Ctrl+C to stop.\n");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("✓ New client connected: {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("✗ Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("  Client handler started, sending messages...");
    
    let ecu_id = b"ECU1";
    let app_id = b"TEST";
    let ctx_id = b"LOG\0";
    
    let mut counter: u32 = 0;
    
    loop {
        // Create DLT message builder
        let builder = DltMessageBuilder::new();
        let mut builder = builder
            .with_ecu_id(ecu_id)
            .with_app_id(app_id)
            .with_context_id(ctx_id)
            .with_session_id(1)
            .with_timestamp(counter);
        
        // Create payload with multiple arguments
        let mut payload_buffer = [0u8; 512];
        let payload_len = {
            let mut payload_builder = PayloadBuilder::new(&mut payload_buffer);
            
            // Add message content
            if let Err(e) = payload_builder.add_string("Message") {
                eprintln!("Failed to add string: {:?}", e);
                continue;
            }
            if let Err(e) = payload_builder.add_u32(counter) {
                eprintln!("Failed to add counter: {:?}", e);
                continue;
            }
            if let Err(e) = payload_builder.add_string("Temperature:") {
                eprintln!("Failed to add label: {:?}", e);
                continue;
            }
            if let Err(e) = payload_builder.add_f32(23.5 + (counter % 10) as f32) {
                eprintln!("Failed to add temperature: {:?}", e);
                continue;
            }
            
            payload_builder.len()
        };
        
        // Generate complete DLT message
        let mut dlt_buffer = [0u8; 1024];
        match builder.generate_log_message_with_payload(
            &mut dlt_buffer,
            &payload_buffer[..payload_len],
            MtinTypeDltLog::DltLogInfo,
            4, // number of arguments
            true, // verbose mode
        ) {
            Ok(total_size) => {
                // Send the DLT message
                if let Err(e) = socket.write_all(&dlt_buffer[..total_size]).await {
                    eprintln!("✗ Failed to send message: {}", e);
                    break;
                }
                
                if counter % 10 == 0 {
                    println!("  Sent {} messages...", counter);
                }
                
                counter += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to generate DLT message: {:?}", e);
                continue;
            }
        }
        
        // Send messages every second
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("  Client disconnected");
    Ok(())
}
