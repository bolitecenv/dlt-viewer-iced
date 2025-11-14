// components/modal_window.rs
// Reusable modal window component

use iced::widget::{center, mouse_area, opaque};
use iced::{
    Color, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, text, Space
    }
};

use crate::app::ICON_FONT;

/// Configuration for modal window appearance
#[derive(Debug, Clone)]
pub struct ModalConfig {
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub show_refresh: bool,
    pub show_apply: bool,
}

impl Default for ModalConfig {
    fn default() -> Self {
        Self {
            width: 900.0,
            height: 600.0,
            title: "Modal Window".to_string(),
            show_refresh: false,
            show_apply: true,
        }
    }
}

/// Base modal state - contains common functionality for all modals
#[derive(Debug, Clone)]
pub struct ModalState {
    pub is_open: bool,
}

impl ModalState {
    pub fn new() -> Self {
        Self {
            is_open: false,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

impl Default for ModalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for modal window content
/// Implement this trait to create custom modal windows
pub trait ModalContent<Message: Clone> {
    /// Build the main content area of the modal
    fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message>;
    
    /// Message to send when close button is pressed
    fn close_message(&self) -> Message;
    
    /// Optional: Message to send when refresh button is pressed
    fn refresh_message(&self) -> Option<Message> {
        None
    }
    
    /// Optional: Message to send when apply button is pressed
    fn apply_message(&self) -> Option<Message> {
        None
    }
    
    /// Get modal configuration
    fn config(&self) -> ModalConfig {
        ModalConfig::default()
    }
}

/// Base modal window structure
pub struct ModalWindow;

impl ModalWindow {
    /// Create a modal window with custom content
    pub fn view<'a, Message: Clone + 'static>(
        content: &impl ModalContent<Message>,
        dark_mode: bool,
        state: &ModalState,
    ) -> Option<Element<'a, Message>> {
        if !state.is_open() {
            return None;
        }

        let config = content.config();
        let main_content = content.build_content(dark_mode);

        // Build footer buttons
        let mut footer_row = row![].spacing(10).width(Length::Fill);
        
        if config.show_refresh {
            if let Some(refresh_msg) = content.refresh_message() {
                footer_row = footer_row.push(
                    button(text("Refresh").size(14))
                        .on_press(refresh_msg)
                        .padding(10)
                );
            }
        }
        
        footer_row = footer_row.push(Space::new(Length::Fill, Length::Shrink));
        
        if config.show_apply {
            if let Some(apply_msg) = content.apply_message() {
                footer_row = footer_row.push(
                    button(text("Apply").size(14))
                        .on_press(apply_msg)
                        .padding(10)
                        .style(move |theme: &Theme, status| {
                            let base_style = button::primary(theme, status);
                            button::Style {
                                background: Some(Color::from_rgb(0.2, 0.6, 0.2).into()),
                                text_color: Color::WHITE,
                                ..base_style
                            }
                        })
                );
            }
        }

        // Main popup content
        let popup_content = container(
            column![
                // Header
                row![
                    text(config.title.clone()).size(20).color(if dark_mode {
                        Color::WHITE
                    } else {
                        Color::BLACK
                    }),
                    Space::new(Length::Fill, Length::Shrink),
                    button(text("\u{f057}").size(16).font(ICON_FONT))
                        .on_press(content.close_message())
                        .padding([4, 8])
                        .style(move |theme: &Theme, status| {
                            let base_style = button::primary(theme, status);
                            button::Style {
                                background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
                                text_color: Color::WHITE,
                                ..base_style
                            }
                        })
                ]
                .spacing(20)
                .align_y(Vertical::Center),
                
                // Divider
                Self::divider::<Message>(dark_mode),
                
                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                // Main content (provided by implementation)
                main_content,
                
                Space::new(Length::Shrink, Length::Fixed(20.0)),
                
                // Footer buttons
                footer_row
            ]
            .spacing(10)
            .padding(20),
        )
        .width(Length::Fixed(config.width))
        .height(Length::Fixed(config.height))
        .style(move |_theme: &Theme| container::Style {
            background: Some(if dark_mode {
                Color::from_rgb(0.15, 0.15, 0.15)
            } else {
                Color::from_rgb(0.98, 0.98, 0.98)
            }.into()),
            text_color: Some(if dark_mode {
                Color::WHITE
            } else {
                Color::BLACK
            }),
            border: iced::Border {
                width: 1.0,
                color: if dark_mode {
                    Color::from_rgb(0.3, 0.3, 0.3)
                } else {
                    Color::from_rgb(0.7, 0.7, 0.7)
                },
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            },
        });

        // Center the popup with backdrop
        let centered_popup = container(popup_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

        Some(
            opaque(
                mouse_area(
                    container(centered_popup)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                            text_color: None,
                            ..Default::default()
                        })
                )
            ).into()
        )
    }

    /// Helper: Create a divider line
    fn divider<'a, Message: 'a>(dark_mode: bool) -> Element<'a, Message> {
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(if dark_mode {
                    Color::from_rgb(0.3, 0.3, 0.3)
                } else {
                    Color::from_rgb(0.8, 0.8, 0.8)
                }.into()),
                text_color: None,
                ..Default::default()
            })
            .into()
    }

    /// Helper: Create a styled panel container
    pub fn panel_container<'a, Message: 'a>(
        content: impl Into<Element<'a, Message>>,
        dark_mode: bool,
        width: Length,
        height: Length,
    ) -> Element<'a, Message> {
        container(content)
            .width(width)
            .height(height)
            .style(move |_theme: &Theme| container::Style {
                background: Some(if dark_mode {
                    Color::from_rgb(0.1, 0.1, 0.1)
                } else {
                    Color::from_rgb(0.95, 0.95, 0.95)
                }.into()),
                border: iced::Border {
                    width: 1.0,
                    color: if dark_mode {
                        Color::from_rgb(0.3, 0.3, 0.3)
                    } else {
                        Color::from_rgb(0.7, 0.7, 0.7)
                    },
                    radius: 4.0.into(),
                },
                text_color: None,
                shadow: Default::default(),
            })
            .padding(10)
            .into()
    }
}