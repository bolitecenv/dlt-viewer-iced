# DLT Viewer Iced - AI Coding Agent Instructions

## Project Overview
A desktop GUI application for viewing and analyzing **DLT (Diagnostic Log and Trace)** protocol messages, built with Rust and the Iced GUI framework. The app connects to DLT sources via TCP, parses binary protocol data, displays messages in tables, and provides interactive visualization widgets (charts, Gantt charts, meters).

**Note**: This project uses **git submodules** for DLT parsing libraries. Always clone with `--recursive` or run `git submodule update --init --recursive` after cloning.

## Architecture

### Core Components
- **`src/app.rs`**: Central `Dashboard` struct using Iced's Elm architecture (update/view/subscription pattern)
- **`src/message.rs`**: All UI events as `Message` enum variants (NavigateTo, ConnectionEvent, SerialConnectionEvent, PluginMessage, ModuleCanvasMessage, etc.)
- **`crate/dlt-protocol/`**: **Git submodule** - DLT protocol library for parsing/generating binary DLT messages (headers, payloads)
- **`src/components/dlt_parser.rs`**: Parser adapter that uses `dlt-protocol` to parse incoming TCP bytes into `ParsedDltMessage`
- **`src/components/tcp_handler.rs`**: Manages multiple named TCP clients with async streams, batches DLT messages
- **`src/components/serial_handler.rs`**: Manages serial port connections with async streams, uses same DLT parser
- **`src/module_view/`**: Canvas-based widget system for data visualization (charts, Gantt charts, meters)
- **`src/plugin_registry.rs`**: Dynamic plugin system - auto-generates registration code via `build.rs`

### Critical Data Flow
1. **TCP → Parser → Batch**: `tcp_handler.rs` reads bytes from `TcpStream`, uses `parse_dlt_message()` from `dlt_parser.rs` to extract `ParsedDltMessage` structs, batches messages
2. **Serial → Parser → Batch**: `serial_handler.rs` reads bytes from `SerialStream`, uses same `parse_dlt_message()` function, batches messages
3. **Batch → Dashboard**: Sends `Message::BatchUpdate` with `Vec<DltMessageRow>` and ECU metadata updates
4. **Dashboard → Widgets**: In `process_dlt_messages()`, iterates over `module_canvas.module_widget` HashMap, calls `add_new_data()` with regex matching
5. **Widgets → Display**: Each widget implements `ModuleWidgetWindowView` trait, draws to Iced canvas using `iced::widget::canvas` primitives

## Key Patterns

### Type-Erased Widget System
Widgets are stored as `Box<dyn ModuleWidgetWindowView>` in `HashMap<usize, ModuleWidget>`. Key trait methods:
```rust
pub trait ModuleWidgetWindowView: Send + Sync {
    fn get_window(&self) -> &ModuleWidgetWindow;
    fn add_new_data_item(&mut self, data: &WidgetData);
    fn draw(&self, frame: &mut canvas::Frame, theme: &Theme);
    fn zoom(&mut self, delta: f32, shift: bool, ctrl: bool);
    fn as_any(&self) -> &dyn Any;  // For downcasting to concrete types
    fn clone_box(&self) -> Box<dyn ModuleWidgetWindowView>;
}
```

**Common Error**: When accessing widgets via `HashMap::get_mut(&id)`, you receive `Option<&mut ModuleWidget>`. Handle the Option - don't unwrap without checking. For iteration, use `iter_mut()` which yields `(&K, &mut V)` tuples.

### Plugin Auto-Registration (build.rs)
- **Adding Plugins**: Create `plugins/my_plugin.rs` implementing `Plugin` trait
- **Build Script**: `build.rs` scans `plugins/` directory, auto-generates `plugins/mod.rs` and registration code in `plugin_registry.rs` between `// Auto Generated:` markers
- **Serialization**: Plugin messages use `bincode` for type-safe communication (see `TimerMessage::create_custom_message()`)

### Modal Window Architecture
- All modals implement `ModalWindowView` trait with `draw()`, `update()`, `content()` methods
- Stored as `Option<Box<dyn ModalWindowView>>` in `Dashboard.modal_window`
- Rendered as overlay using `iced::widget::stack![base_view, modal_element]`
- **ID Pattern**: Modals can reference widgets via `get_id() -> Option<u32>` for editing widget settings

### DLT Protocol Specifics
- **Binary Parsing**: Use `parse_dlt_message()` function on `&[u8]` slices: `parse_dlt_message(data) -> Result<(ParsedDltMessage, &[u8]), DltParseError>`
- **Message Generation**: Use `DltMessageBuilder` from `dlt-protocol` to create DLT messages (see `examples/dlt_test_server.rs`)
- **Chunking**: TCP streams are buffered (160MB buffer), parser returns remaining bytes after each message
- **Service Messages**: Control messages not yet implemented - marked with TODO comments
- **ECU Hierarchy**: ECU → App ID → Context ID tree structure stored in `Vec<FrontDltEcuItem>`
- **Payload Parsing**: `ParsedDltMessage::parse_payload()` uses `PayloadParser` to extract verbose message strings

## Development Workflows

### Initial Setup
```bash
# Clone with submodules
git clone --recursive git@github.com:bolitecenv/dlt-viewer-iced.git

# Or if already cloned, initialize submodules
git submodule update --init --recursive
```

### Connection Types

#### TCP/IP Connection
The viewer can connect to DLT daemons via TCP:
- Configure IP address and port
- Uses `tokio::net::TcpStream` for async I/O
- Connection managed via `TCPClientsHandler`
- Supports multiple simultaneous TCP connections

#### Serial/TTY Connection  
The viewer can connect to DLT sources via serial ports:
- Configure serial port path (e.g., `/dev/ttyUSB0`, `/dev/ttyACM0`, `COM3`)
- Configure baud rate (default: 115200)
- Uses `tokio-serial` crate for async serial I/O
- Auto-reconnects on disconnection
- Uses same DLT parser as TCP connections

Both connection types:
- Share the same `parse_dlt_messages()` function
- Generate `Message::BatchUpdate` with parsed DLT messages
- Support multiple named connections simultaneously
- Display in unified connection manager UI

### Building & Running
```bash
cargo build --release        # Triggers build.rs to regenerate plugin code
cargo run                     # Dev build with debug output
cargo test                    # Runs tests in dlt-format-parser crate
```

### Working with Submodules
```bash
# Update submodules to latest commits
git submodule update --remote

# Make changes in a submodule
cd crate/dlt-format-parser
git checkout -b my-feature
# ... make changes ...
git commit -am "Update parser"
git push origin my-feature

# Update parent repo to reference new submodule commit
cd ../..
git add crate/dlt-format-parser
git commit -m "Update dlt-format-parser submodule"
```

**Important**: Submodule repository:
- `crate/dlt-protocol`: https://github.com/bolitecenv/dlt-protocol

When modifying DLT parsing logic, create branches/PRs in the submodule repos first, then update the parent repo's submodule reference.

### Testing DLT Parsing
Parser tests are in `src/components/dlt_parser.rs` under `#[cfg(test)]`. Use hardcoded byte arrays:
```rust
let data: [u8; 32] = [0x35, 0x00, 0x00, 0x20, ...];
let (parsed_msg, remaining) = parse_dlt_message(&data).unwrap();
```

### Testing with DLT Server
Run the test DLT server that generates and sends DLT messages:
```bash
# Terminal 1: Start test server
cargo run --example dlt_test_server

# Terminal 2: Run the viewer
cargo run
# Connect to 127.0.0.1:3490
```

The test server (`examples/dlt_test_server.rs`) demonstrates:
- Creating DLT messages with `DltMessageBuilder`
- Adding verbose payloads with `PayloadBuilder`
- Sending messages over TCP
- Message counter and timestamp management

### Testing Serial Connections
To test serial port connections without physical hardware, you can use virtual serial ports:

#### On Linux:
```bash
# Create virtual serial ports using socat
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# This creates two linked ports, e.g., /dev/pts/3 and /dev/pts/4

# In one terminal, send DLT messages to one port
cat dlt_messages.bin > /dev/pts/3

# In the viewer, connect to the other port
# Serial Port: /dev/pts/4
# Baud Rate: 115200
```

#### On macOS:
```bash
# Create virtual serial ports using socat (install via brew)
brew install socat
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# Use the created /dev/ttys### ports
```

Alternatively, modify the test server to send to serial instead of TCP, or use real hardware with a USB-to-serial adapter.

### Adding New Widget Types
1. Create widget struct (e.g., `MeterWidget`) with settings struct
2. Implement `ModuleWidgetWindowView` trait
3. Add variant to `WidgetData` enum if parsing DLT payloads
4. Update regex matching logic in `ModuleWidget::process_data_for_widget()`
5. Add creation case in `ModuleCanvasMessage::AddMeter` handler

### Debugging TCP Connections
- Use `println!()` liberally - no logging framework yet
- Check `Dashboard.connection_status` string for error messages
- Inspect `TCPClientsHandler.clients` HashMap for per-client state
- Connection lifecycle: `try_connect()` → `ConnectionEvent::Connected` → `create_client_subscription()` → async read loop

## Common Gotchas

### Iced Subscriptions
- **Batching**: Use `Subscription::batch(vec![...])` to combine multiple subscriptions
- **Per-Client**: Each TCP client needs separate subscription with unique `name` parameter
- **Unfold Pattern**: TCP reading uses `futures::stream::unfold()` for continuous async reads

### Canvas Coordinate System
- Origin (0,0) is top-left
- Grid snapping uses `GRID_SIZE = 50.0` and `SNAP_THRESHOLD = 10.0`
- Widget positions stored as `Point`, sizes as `Size` (both from `iced::Point/Size`)
- Resize handles checked via `get_window_resize_type_contains_point()` with 5px margins

### Message Routing
- **Plugin messages**: `Message::PluginMessage(name, msg)` → routed to specific plugin by name
- **Canvas messages**: `ModuleCanvasMessage` → handled in `ModuleCanvas::update()` → may trigger modal creation
- **Modal messages**: `ModalWindowMessage` → handled in `Dashboard::update()` via `modal.update()` → may access widget by ID

### Regex Data Extraction
Default patterns expect named capture groups:
- **Chart**: `r"X:\s*(?<X>[-+]?[0-9]*\.?[0-9]+).*Y:\s*(?<Y>[-+]?[0-9]*\.?[0-9]+)"`
- **Gantt**: `r"(?:START:\s*(?P<Start>\d+\.?\d*)|END:\s*(?P<End>\d+\.?\d*))\s*LABEL:\s*(?P<Label>\w+)"`

Store patterns in `DltDataRegexItem` with widget ID for matching.

## Style Conventions
- **Theme Toggle**: `Dashboard.dark_mode` boolean controls all color schemes
- **Border Colors**: Widgets use `Color::from_rgb()` for consistent appearance
- **Icon Font**: Use `ICON_FONT` constant (Font Awesome 7 Free) for UI icons
- **Spacing**: Standard padding is 10-20px, use `Length::Fill` for expanding elements

## Files to Reference
- **Message Routing**: `src/message.rs` (all event types)
- **Widget Traits**: `src/module_view/module_widget.rs` (trait definitions)
### DLT Protocol**: `crate/dlt-protocol/src/types.rs` (constants, header structs)
- **TCP Handling**: `src/components/tcp_handler.rs` (`BatchUpdate` emission)
- **Plugin Example**: `plugins/timer.rs` (full implementation with state)
