use crate::message::Message;
use iced::{
    Border, Color, Element, Length, Padding, Shadow, Theme,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{Space, button, checkbox, column, container, row, text, text_input},
};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn view<'a>(
    dark_mode: bool,
    tcp_ip: &'a str,
    tcp_port: &'a str,
    connection_status: &'a str,
) -> Element<'a, Message> {
    let page_header = container(
        column![
            text::<Theme, _>("Settings")
                .size(28)
                .style(|theme: &Theme| text::Style {
                    color: Some(if matches!(theme, Theme::Dark) {
                        Color::WHITE
                    } else {
                        Color::from_rgb(0.1, 0.1, 0.1)
                    }),
                }),
            Space::with_height(Length::Fixed(4.0)),
            text::<Theme, _>("Customize your application preferences")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(if matches!(theme, Theme::Dark) {
                        Color::from_rgb(0.7, 0.7, 0.7)
                    } else {
                        Color::from_rgb(0.5, 0.5, 0.5)
                    }),
                }),
        ]
        .spacing(0),
    )
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 24.0,
        left: 0.0,
    });

    let appearance_card = create_card(
        "Appearance",
        "Manage theme and display settings",
        column![setting_row(
            "Dark Mode",
            "Switch between light and dark theme",
            checkbox("", dark_mode)
                .on_toggle(|_| Message::ToggleTheme)
                .into()
        ),]
        .spacing(0)
        .into(), // Added .into()
    );

    let tcp_card = create_card(
        "Network Connection",
        "Configure TCP/IP connection settings",
        column![
            input_field_with_icon(
                "IP Address",
                "Enter server IP address",
                text_input("127.0.0.1", tcp_ip)
                    .on_input(Message::TcpIpChanged)
                    .padding(12)
                    .size(14)
                    .into() // Added .into()
            ),
            Space::with_height(Length::Fixed(16.0)),
            input_field_with_icon(
                "Port",
                "Enter server port number",
                text_input("8080", tcp_port)
                    .on_input(Message::TcpPortChanged)
                    .padding(12)
                    .size(14)
                    .into() // Added .into()
            ),
            Space::with_height(Length::Fixed(20.0)),
            connection_section(connection_status),
        ]
        .spacing(0)
        .into(), // Added .into()
    );

    let settings_content = column![page_header, appearance_card, tcp_card,]
        .spacing(16)
        .padding(32)
        .max_width(600);

    container(settings_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill) // Added Length::Fill parameter
        .style(|theme: &Theme| container::Style {
            background: Some(
                if matches!(theme, Theme::Dark) {
                    Color::from_rgb(0.08, 0.08, 0.08)
                } else {
                    Color::from_rgb(0.96, 0.96, 0.97)
                }
                .into(),
            ),
            ..Default::default()
        })
        .into()
}

fn create_card<'a>(
    title: &'static str,
    description: &'static str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            // Card header
            column![
                text::<Theme, _>(title).size(18),
                Space::with_height(Length::Fixed(4.0)),
                text::<Theme, _>(description)
                    .size(13)
                    .style(|theme: &Theme| text::Style {
                        color: Some(if matches!(theme, Theme::Dark) {
                            Color::from_rgb(0.6, 0.6, 0.6)
                        } else {
                            Color::from_rgb(0.5, 0.5, 0.5)
                        }),
                    }),
            ]
            .spacing(0)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 20.0,
                left: 0.0,
            }),
            // Card content
            content,
        ]
        .spacing(0),
    )
    .padding(24)
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(
            if matches!(theme, Theme::Dark) {
                Color::from_rgb(0.12, 0.12, 0.13)
            } else {
                Color::WHITE
            }
            .into(),
        ),
        border: Border {
            color: if matches!(theme, Theme::Dark) {
                Color::from_rgba(1.0, 1.0, 1.0, 0.05)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.08)
            },
            width: 1.0,
            radius: Radius::from(12.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    })
    .into()
}

fn setting_row<'a>(
    label: &'static str,
    description: &'static str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text::<Theme, _>(label).size(14),
                Space::with_height(Length::Fixed(4.0)),
                text::<Theme, _>(description)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(if matches!(theme, Theme::Dark) {
                            Color::from_rgb(0.6, 0.6, 0.6)
                        } else {
                            Color::from_rgb(0.5, 0.5, 0.5)
                        }),
                    }),
            ]
            .spacing(0),
            Space::with_width(Length::Fill),
            container(control).center_y(Length::Shrink), // Added Length::Shrink parameter
        ]
        .align_y(iced::alignment::Vertical::Center) // Changed from align_items to align_y
        .spacing(16),
    )
    .padding(Padding {
        top: 12.0,
        right: 0.0,
        bottom: 12.0,
        left: 0.0,
    })
    .width(Length::Fill)
    .into()
}

fn input_field_with_icon<'a>(
    label: &'static str,
    _placeholder: &'static str, // Added underscore prefix
    input: Element<'a, Message>,
) -> Element<'a, Message> {
    column![
        text::<Theme, _>(label)
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(if matches!(theme, Theme::Dark) {
                    Color::from_rgb(0.85, 0.85, 0.85)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.2)
                }),
            }),
        Space::with_height(Length::Fixed(8.0)),
        container(input)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(
                    if matches!(theme, Theme::Dark) {
                        Color::from_rgb(0.08, 0.08, 0.09)
                    } else {
                        Color::from_rgb(0.97, 0.97, 0.98)
                    }
                    .into(),
                ),
                border: Border {
                    color: if matches!(theme, Theme::Dark) {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.12)
                    },
                    width: 1.0,
                    radius: Radius::from(8.0),
                },
                ..Default::default()
            }),
    ]
    .spacing(0)
    .into()
}

fn connection_section(status: &str) -> Element<'_, Message> {
    container(
        row![
            container(
                button(
                    container(text::<Theme, _>("Connect").size(14))
                        .padding(Padding {
                            top: 0.0,
                            right: 8.0,
                            bottom: 0.0,
                            left: 8.0,
                        })
                        .center_x(Length::Fill) // Added Length::Fill parameter
                )
                .on_press(Message::ConnectTcp)
                .padding(Padding {
                    top: 12.0,
                    right: 24.0,
                    bottom: 12.0,
                    left: 24.0,
                })
                .style(|theme: &Theme, status| {
                    let base_style = button::Style {
                        background: Some(
                            if matches!(theme, Theme::Dark) {
                                Color::from_rgb(0.3, 0.45, 0.9)
                            } else {
                                Color::from_rgb(0.2, 0.4, 0.95)
                            }
                            .into(),
                        ),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(8.0),
                            ..Default::default()
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.2, 0.4, 0.95, 0.3),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 8.0,
                        },
                    };

                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(
                                if matches!(theme, Theme::Dark) {
                                    Color::from_rgb(0.35, 0.5, 0.95)
                                } else {
                                    Color::from_rgb(0.25, 0.45, 0.98)
                                }
                                .into(),
                            ),
                            ..base_style
                        },
                        button::Status::Pressed => button::Style {
                            background: Some(
                                if matches!(theme, Theme::Dark) {
                                    Color::from_rgb(0.25, 0.4, 0.85)
                                } else {
                                    Color::from_rgb(0.15, 0.35, 0.9)
                                }
                                .into(),
                            ),
                            ..base_style
                        },
                        _ => base_style,
                    }
                })
            ),
            Space::with_width(Length::Fixed(16.0)),
            container(
                row![
                    connection_status_indicator(status),
                    Space::with_width(Length::Fixed(8.0)),
                    text::<Theme, _>(status)
                        .size(13)
                        .style(|theme: &Theme| text::Style {
                            color: Some(if matches!(theme, Theme::Dark) {
                                Color::from_rgb(0.7, 0.7, 0.7)
                            } else {
                                Color::from_rgb(0.4, 0.4, 0.4)
                            }),
                        }),
                ]
                .align_y(iced::alignment::Vertical::Center) // Changed from align_items to align_y
            )
            .center_y(Length::Shrink) // Added Length::Shrink parameter
        ]
        .align_y(iced::alignment::Vertical::Center), // Changed from align_items to align_y
    )
    .into()
}

fn connection_status_indicator(status: &str) -> Element<'_, Message> {
    let color = if status.contains("Connected") {
        Color::from_rgb(0.2, 0.8, 0.4)
    } else if status.contains("failed") || status.contains("Failed") {
        Color::from_rgb(0.95, 0.3, 0.3)
    } else {
        Color::from_rgb(0.6, 0.6, 0.6)
    };

    container(Space::with_width(Length::Fixed(0.0)))
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: Radius::from(4.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// Simple TCP connection handler
pub fn handle_tcp_connection(ip: &str, port: &str) -> Result<String, String> {
    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            println!("Successfully connected to {}", address);

            // Send a simple message
            let message = b"Hello from client";
            stream
                .write_all(message)
                .map_err(|e| format!("Failed to send data: {}", e))?;

            // Read response
            let mut buffer = [0; 512];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    println!("Received: {}", response);
                    Ok(format!("Connected to {}", address))
                }
                Err(e) => {
                    println!("Failed to read response: {}", e);
                    Ok(format!("Connected to {}", address))
                }
            }
        }
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}
