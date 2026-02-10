/// DLT Serial Test Sender
/// 
/// This example sends DLT messages to a serial port for testing the viewer's serial functionality.
/// 
/// Usage:
/// 1. Create virtual serial ports (Linux/macOS):
///    socat -d -d pty,raw,echo=0 pty,raw,echo=0
///    # This creates two linked ports, e.g., /dev/pts/3 and /dev/pts/4
/// 
/// 2. Run this sender on one port:
///    cargo run --example dlt_serial_test_sender /dev/pts/3
/// 
/// 3. Connect the viewer to the other port:
///    Serial Port: /dev/pts/4
///    Baud Rate: 115200

use dlt_protocol::*;
use std::env;
use std::io::Write;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <serial_port>", args[0]);
        eprintln!("Example: {} /dev/pts/3", args[0]);
        std::process::exit(1);
    }
    
    let port_name = &args[1];
    let baud_rate = 115200;
    
    println!("=== DLT Serial Test Sender ===");
    println!("Opening serial port: {}", port_name);
    println!("Baud rate: {}", baud_rate);
    
    // Open serial port
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open()?;
    
    println!("✓ Serial port opened successfully");
    println!("Sending DLT messages every 500ms...");
    println!("Press Ctrl+C to stop.\n");
    
    let ecu_id = b"ECU1";
    let app_id = b"APP1";
    let ctx_id = b"CTX1";
    let mut counter = 0u32;
    
    loop {
        counter += 1;
        
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
            
            // Add message content (handle errors manually)
            if let Err(e) = payload_builder.add_string(&format!("Serial test message #{}", counter)) {
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
            if let Err(e) = payload_builder.add_f32(21.5 + (counter % 10) as f32) {
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
                match port.write_all(&dlt_buffer[..total_size]) {
                    Ok(_) => {
                        println!("[{}] Sent message #{} ({} bytes)", 
                            chrono::Local::now().format("%H:%M:%S"),
                            counter, 
                            total_size);
                    }
                    Err(e) => {
                        eprintln!("Error writing to serial port: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error building DLT message: {:?}", e);
            }
        }
        
        std::thread::sleep(Duration::from_millis(500));
    }
    
    Ok(())
}
