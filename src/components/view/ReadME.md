# Modal Window System

This system provides a reusable modal window framework for creating consistent, styled popup dialogs in your Iced application.

## Architecture

The modal system is split into three files:

1. **modal_window.rs** - Base modal window infrastructure, styling, and common state management
2. **dlt_settings.rs** - DLT Settings implementation using the modal system
3. **chart_settings.rs** - Example chart settings implementation

## File Structure

```
components/
├── modal_window.rs      # Base modal framework with ModalState
├── dlt_settings.rs      # DLT-specific modal
└── chart_settings.rs    # Chart-specific modal (example)
```

## Key Components

### ModalState

A reusable struct that handles common modal state and operations:

```rust
pub struct ModalState {
    pub is_open: bool,
}

impl ModalState {
    pub fn new() -> Self;
    pub fn open(&mut self);
    pub fn close(&mut self);
    pub fn toggle(&mut self);
    pub fn is_open(&self) -> bool;
}
```

**Benefits:**
- No need to implement `open()`, `close()`, `toggle()` in every modal
- Consistent state management across all modals
- Reduces boilerplate code

## How to Create a New Modal

### Step 1: Create Your Modal Struct with ModalState

```rust
pub struct MySettingsView {
    pub modal_state: ModalState,  // ← Common state management
    // Add your custom fields here
    pub my_setting: String,
}
```

### Step 2: Implement Basic Methods (Using ModalState)

```rust
impl MySettingsView {
    pub fn new() -> Self {
        Self {
            modal_state: ModalState::new(),  // ← Initialize modal state
            my_setting: String::new(),
        }
    }

    // Delegate common operations to ModalState
    pub fn open(&mut self) {
        self.modal_state.open();
    }

    pub fn close(&mut self) {
        self.modal_state.close();
    }

    pub fn toggle(&mut self) {
        self.modal_state.toggle();
    }

    pub fn is_open(&self) -> bool {
        self.modal_state.is_open()
    }

    pub fn view<'a>(&self, dark_mode: bool) -> Option<Element<'a, Message>> {
        ModalWindow::view(self, dark_mode, &self.modal_state)
    }
}
```

**Note:** You can add custom logic to `close()` if needed:
```rust
pub fn close(&mut self) {
    self.modal_state.close();
    // Add your cleanup logic here
    self.is_editing = false;
    self.reset_form();
}
```

### Step 3: Implement ModalContent Trait

```rust
impl ModalContent<Message> for MySettingsView {
    fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        // Build your custom content here
        column![
            text("My Settings").size(18),
            // Add your UI elements
        ]
        .into()
    }

    fn close_message(&self) -> Message {
        Message::CloseMySettings
    }

    fn refresh_message(&self) -> Option<Message> {
        Some(Message::RefreshMySettings)
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::ApplyMySettings)
    }

    fn config(&self) -> ModalConfig {
        ModalConfig {
            width: 800.0,
            height: 600.0,
            title: "My Settings".to_string(),
            show_refresh: true,
            show_apply: true,
        }
    }
}
```

## ModalContent Trait Methods

### Required Methods

- **`build_content()`** - Returns the main content area of your modal
- **`close_message()`** - Returns the message to send when the close button is clicked

### Optional Methods

- **`refresh_message()`** - Returns message for refresh button (button only shows if this returns Some)
- **`apply_message()`** - Returns message for apply button (button only shows if this returns Some)
- **`config()`** - Returns modal configuration (size, title, button visibility)

## Modal Configuration

The `ModalConfig` struct controls the modal's appearance:

```rust
pub struct ModalConfig {
    pub width: f32,           // Modal width in pixels
    pub height: f32,          // Modal height in pixels
    pub title: String,        // Title text in header
    pub show_refresh: bool,   // Show refresh button in footer
    pub show_apply: bool,     // Show apply button in footer
}
```

## Benefits of This Architecture

1. **Less Boilerplate** - `ModalState` eliminates repetitive `open()`, `close()`, `toggle()` implementations
2. **Consistency** - All modals behave the same way for common operations
3. **Reusability** - Write the modal framework once, reuse everywhere
4. **Maintainability** - Changes to modal styling or behavior happen in one place
5. **Flexibility** - Easy to add custom logic to common methods when needed
6. **Type Safety** - Compile-time checking ensures proper implementation

## Helper Methods

The `ModalWindow` struct provides helper methods:

### `ModalWindow::panel_container()`

Creates a styled panel container for content sections:

```rust
ModalWindow::panel_container(
    scrollable(content),
    dark_mode,
    Length::Fill,
    Length::Fixed(400.0),
)
```

## Usage in Your Application

### 1. Add to your Message enum:

```rust
pub enum Message {
    OpenMySettings,
    CloseMySettings,
    RefreshMySettings,
    ApplyMySettings,
    // ... other messages
}
```

### 2. Add modal to your app state:

```rust
pub struct MyApp {
    my_settings: MySettingsView,
    dark_mode: bool,
    // ... other fields
}
```

### 3. Handle messages in update():

```rust
fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::OpenMySettings => {
            self.my_settings.toggle();
        }
        Message::CloseMySettings => {
            self.my_settings.close();
        }
        Message::ApplyMySettings => {
            // Apply your settings
        }
        // ... handle other messages
    }
    Command::none()
}
```

### 4. Add to your view():

```rust
fn view(&self) -> Element<Message> {
    let main_content = column![
        // Your main UI
    ];

    // Layer the modal on top if open
    if let Some(modal) = self.my_settings.view(self.dark_mode) {
        stack![main_content, modal].into()
    } else {
        main_content.into()
    }
}
```

## Examples

### Two-Panel Layout (Like DLT Settings)

```rust
fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
    row![
        // Left panel
        ModalWindow::panel_container(
            scrollable(left_content),
            dark_mode,
            Length::FillPortion(2),
            Length::Fixed(400.0),
        ),
        
        Space::new(Length::Fixed(10.0), Length::Shrink),
        
        // Right panel
        ModalWindow::panel_container(
            scrollable(right_content),
            dark_mode,
            Length::FillPortion(3),
            Length::Fixed(400.0),
        ),
    ]
    .width(Length::Fill)
    .into()
}
```

### Single-Panel Layout (Like Chart Settings)

```rust
fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
    ModalWindow::panel_container(
        scrollable(content),
        dark_mode,
        Length::Fill,
        Length::Fixed(400.0),
    )
}
```

## Styling

All modals automatically support:
- Dark mode and light mode
- Backdrop overlay (semi-transparent black)
- Shadow effects
- Rounded corners
- Consistent border styling
- Hover effects on buttons

## Adding to Your Project

1. Copy `modal_window.rs` to your components directory
2. Create new modal files based on the examples
3. Update your `Message` enum with the necessary variants
4. Implement the `ModalContent` trait for your modal
5. Add the modal to your app state and view logic

## Common Patterns

### Adding Custom Cleanup on Close

```rust
pub fn close(&mut self) {
    self.modal_state.close();
    // Your custom cleanup
    self.reset_selection();
    self.clear_unsaved_changes();
}
```

### Conditional Opening

```rust
pub fn open(&mut self) {
    if self.is_data_loaded() {
        self.modal_state.open();
    } else {
        // Load data first, then open
    }
}
```