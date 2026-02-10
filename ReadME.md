# DLT Viewer Iced

A modern desktop GUI application for viewing and analyzing **DLT (Diagnostic Log and Trace)** protocol messages, built with Rust and the Iced GUI framework.

## Features

- **Multiple Connection Types**
  - TCP/IP connection to DLT daemons
  - Serial/TTY connection for direct device communication
  - Multiple simultaneous connections supported
  
- **Real-time Message Viewing**
  - Table view with ECU ID, App ID, Context ID, and message content
  - Auto-parsing of DLT binary protocol
  - Support for verbose and non-verbose payloads
  
- **Data Visualization**
  - Interactive canvas with chart widgets
  - Gantt chart for timing analysis
  - Meter widgets for sensor data
  - Register monitor for hardware register values
  - Custom regex-based data extraction from payloads
  - User-configurable pattern file (`pattern_config.toml`)

- **Plugin System**
  - Dynamic plugin loading
  - Auto-generated plugin registry
  - Example timer plugin included

## Installation

### Prerequisites
- Rust 1.70+ (2024 edition)
- Git with submodule support

### Build from Source

```bash
# Clone with submodules
git clone --recursive https://github.com/bolitecenv/dlt-viewer-iced.git
cd dlt-viewer-iced

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

### TCP/IP Connection

1. Select "TCP/IP" from the connection type dropdown
2. Enter client name (e.g., "main_client")
3. Enter IP address (e.g., "127.0.0.1")
4. Enter port number (e.g., "3490")
5. Click "Add TCP Client"

### Serial Port Connection

1. Select "TTY/USB Serial" from the connection type dropdown
2. Enter client name (e.g., "serial_client")
3. Enter serial port path:
   - Linux: `/dev/ttyUSB0`, `/dev/ttyACM0`
   - macOS: `/dev/cu.usbserial-*`
   - Windows: `COM3`, `COM4`
4. Enter baud rate (e.g., "115200")
5. Click "Add Serial Client"

The viewer will automatically connect and start receiving DLT messages.

### Testing with DLT Test Server

A test TCP server is included for testing without a real DLT daemon:

```bash
# Terminal 1: Start test server
cargo run --example dlt_test_server

# Terminal 2: Run the viewer and connect to 127.0.0.1:3490
cargo run
```

### Testing Serial Connections

#### Option 1: Using Virtual Serial Ports (Recommended for Testing)

**On Linux:**
```bash
# Terminal 1: Create virtual serial ports
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# Note the created ports, e.g., /dev/pts/3 and /dev/pts/4

# Terminal 2: Run the serial test sender
cargo run --example dlt_serial_test_sender /dev/pts/3

# Terminal 3: Run the viewer
cargo run
# In the UI:
# 1. Select "TTY/USB Serial"
# 2. Enter client name (e.g., "test_serial")
# 3. Enter serial port: /dev/pts/4
# 4. Enter baud rate: 115200
# 5. Click "Add Serial Client"
```

**On macOS:**
```bash
# Install socat if needed
brew install socat

# Terminal 1: Create virtual serial ports
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# Note the created ports, e.g., /dev/ttys001 and /dev/ttys002

# Terminal 2: Run the serial test sender
cargo run --example dlt_serial_test_sender /dev/ttys001

# Terminal 3: Run the viewer and connect to the other port
cargo run
```

#### Option 2: Using Real Serial Hardware

If you have a device sending DLT messages over serial:

```bash
# Find your serial port
ls /dev/tty*  # Look for /dev/ttyUSB0, /dev/ttyACM0, etc.

# Run the viewer
cargo run

# In the UI:
# 1. Select "TTY/USB Serial"
# 2. Enter client name
# 3. Enter your serial port (e.g., /dev/ttyUSB0)
# 4. Enter baud rate (typically 115200, 9600, or 38400)
# 5. Click "Add Serial Client"
```

The viewer will automatically attempt to connect and will retry if the device disconnects.

## Pattern-Based Payload Parsing

The viewer supports regex-based pattern matching to extract structured data from DLT message payloads. Patterns are defined in `pattern_config.toml`.

### Supported Widget Types

1. **Register Widget** - Displays hardware register values
   - Pattern: `#REG: <NAME>: <VALUE>`
   - Example: `#REG: STATUS_REG: 0x1234`
   - Values can be hex (0x format) or decimal

2. **Chart Widget** - Plots X-Y coordinate data
   - Pattern: `X: <value> Y: <value>`
   - Example: `X: 1.23 Y: 4.56`

3. **Gantt Chart Widget** - Timeline visualization
   - Start: `START: <time> LABEL: <name>`
   - End: `END: <time> LABEL: <name>`

### Example Pattern Configuration

```toml
[[pattern]]
name = "register"
widget_type = "register"
description = "Hardware register values"
regex = '#REG:\s*(?P<Name>\w+):\s*(?P<Value>0x[0-9a-fA-F]+|[0-9]+)'
example = "#REG: STATUS_REG: 0x1234"
```

### Testing Pattern Recognition

```bash
# Terminal 1: Start register test server
cargo run --example dlt_register_test

# Terminal 2: Run the viewer
cargo run

# In the UI:
# 1. Connect to 127.0.0.1:3490
# 2. Right-click on canvas → Add Register
# 3. Watch register values update in real-time!
```

### Adding Custom Patterns

Edit `pattern_config.toml` to add your own regex patterns:

1. Define a named capture group for each data field
2. Choose a widget_type (chart, gantt, register)
3. The viewer will automatically match payloads and route data to widgets

## Architecture

The application uses:
- **Iced 0.14.0** - GUI framework with Elm-like architecture
- **tokio** - Async runtime for TCP and serial I/O
- **tokio-serial** - Serial port support
- **dlt-protocol** - DLT binary protocol parser (git submodule)

See `.github/copilot-instructions.md` for detailed architecture documentation.

## License

See LICENSE.txt for details.
