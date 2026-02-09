// tests/dlt_test_server.rs
// Test DLT server that sends DLT messages over TCP for testing the viewer

use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use dlt_protocol::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DLT Test Server starting...");
    
    let listener = TcpListener::bind("127.0.0.1:3490").await?;
    println!("Listening on 127.0.0.1:3490");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    println!("Client handler started");
    
    let ecu_id = b"ECU1";
    let app_id = b"TEST";
    let ctx_id = b"LOG\0";
    
    let mut counter: u32 = 0;
    
    loop {
        // Create DLT message builder
        let mut builder = DltMessageBuilder::new();
        builder
            .with_ecu_id(ecu_id)
            .with_app_id(app_id)
            .with_context_id(ctx_id)
            .with_session_id(1)
            .with_timestamp(counter);
        
        // Create payload with multiple arguments
        let mut payload_buffer = [0u8; 512];
        let payload_len = {
            let mut payload_builder = PayloadBuilder::new(&mut payload_buffer);
            payload_builder.add_string("Message number:")?;
            payload_builder.add_u32(counter)?;
            payload_builder.add_string("Temperature:")?;
            payload_builder.add_f32(23.5 + (counter % 10) as f32)?;
            payload_builder.len()
        };
        
        // Generate complete DLT message
        let mut dlt_buffer = [0u8; 1024];
        let total_size = builder.generate_log_message_with_payload(
            &mut dlt_buffer,
            &payload_buffer[..payload_len],
            MtinTypeDltLog::DltLogInfo,
            4, // number of arguments
            true, // verbose mode
        )?;
        
        // Send the DLT message
        socket.write_all(&dlt_buffer[..total_size]).await?;
        println!("Sent DLT message #{} ({} bytes)", counter, total_size);
        
        counter += 1;
        
        // Send messages every second
        sleep(Duration::from_secs(1)).await;
    }
}
