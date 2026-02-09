use iced::{Background, Border, Color, Element, Length, Shadow, Theme};
use iced::widget::{Button, Text, button, column, container, Container, row, scrollable, text, text_input, pick_list};
use crate::components::tcp_handler::{TCPClientsHandler, TCPClient};
use crate::message::Message;

/// Connection type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    TCP,
    Serial,
}

impl ConnectionType {
    pub const ALL: [ConnectionType; 2] = [ConnectionType::TCP, ConnectionType::Serial];
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConnectionType::TCP => "TCP/IP",
                ConnectionType::Serial => "TTY/USB Serial",
            }
        )
    }
}

/// The main view function for this page
pub fn view<'a>(
    clients_handler: &'a TCPClientsHandler,
    current_tcp_client_name: &'a str,
    current_ip: &'a str,
    current_port: &'a str,
    current_connection_type: Option<ConnectionType>,
    current_serial_port: &'a str,
    current_baud_rate: &'a str,
) -> Element<'a, Message> {
    
    // Connection type selector
    let connection_type_picker = pick_list(
        &ConnectionType::ALL[..],
        current_connection_type,
        Message::ConnectionTypeSelected,
    )
    .placeholder("Select Connection Type")
    .padding(10)
    .width(Length::Fixed(200.0));

    let client_name = text_input("Client Name", current_tcp_client_name)
        .on_input(Message::TcpClientNameChanged)
        .padding(10)
        .width(Length::Fixed(200.0));

    // Build controls based on selected connection type
    let connection_controls = match current_connection_type {
        Some(ConnectionType::TCP) => {
            let ip_input = text_input("IP Address", current_ip)
                .on_input(Message::TcpIpChanged)
                .padding(10)
                .width(Length::Fixed(200.0));

            let port_input = text_input("Port", current_port)
                .on_input(Message::TcpPortChanged)
                .padding(10)
                .width(Length::Fixed(100.0));

            let add_button = button(text("+ Add TCP Client"))
                .on_press(Message::ConnectTcp)
                .padding(10)
                .style(button::primary);

            row![ip_input, port_input, add_button]
                .spacing(10)
                .align_y(iced::Alignment::Center)
        }
        Some(ConnectionType::Serial) => {
            let serial_port_input = text_input("Serial Port (e.g., /dev/ttyUSB0)", current_serial_port)
                .on_input(Message::SerialPortChanged)
                .padding(10)
                .width(Length::Fixed(250.0));

            let baud_rate_input = text_input("Baud Rate", current_baud_rate)
                .on_input(Message::BaudRateChanged)
                .padding(10)
                .width(Length::Fixed(120.0));

            let add_button = button(text("+ Add Serial Client"))
                .on_press(Message::ConnectSerial)
                .padding(10)
                .style(button::primary);

            row![serial_port_input, baud_rate_input, add_button]
                .spacing(10)
                .align_y(iced::Alignment::Center)
        }
        None => {
            row![text("← Select a connection type").size(14).color(Color::from_rgb(0.6, 0.6, 0.6))]
                .spacing(10)
                .align_y(iced::Alignment::Center)
        }
    };

    let controls = column![
        row![connection_type_picker, client_name]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        connection_controls
    ]
    .spacing(10);

    // The List of Cards
    let client_cards = clients_handler.clients.values().fold(
        column![].spacing(10),
        |col, client| col.push(view_client_card(client))
    );

    // Assemble the Page
    let content = column![
        text("DLT Daemon Connections").size(24),
        controls,
        text("Active Connections:").size(18),
        scrollable(client_cards).height(Length::Fill)
    ]
    .spacing(20)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Helper function to render a specific Client Card
fn view_client_card<'a>(client: &'a TCPClient) -> Element<'a, Message> {
    // Determine connection status logic
    let (status_text, status_color) = if client.stream.is_some() {
        ("Connected", Color::from_rgb(0.0, 0.8, 0.0)) // Green
    } else {
        ("Disconnected", Color::from_rgb(0.8, 0.0, 0.0)) // Red
    };

    let status_indicator = row![
        text("●").size(20).color(status_color),
        text(status_text).size(14)
    ]
    .spacing(5)
    .align_y(iced::Alignment::Center);

    // Display connection details based on type
    let connection_info = if client.config.is_serial {
        row![
            text(format!("Port: {}", client.config.serial_port)).size(12),
            text("|").size(12),
            text(format!("Baud: {}", client.config.baud_rate)).size(12),
        ]
        .spacing(10)
    } else {
        row![
            text(format!("IP: {}", client.config.ip)).size(12),
            text("|").size(12),
            text(format!("Port: {}", client.config.port)).size(12),
        ]
        .spacing(10)
    };

    // Connection type badge
    let conn_type_badge = container(
        text(if client.config.is_serial { "SERIAL" } else { "TCP" })
            .size(10)
            .color(Color::WHITE)
    )
    .padding(4)
    .style(badge_style);

    // Card Content
    let card_content = column![
        row![
            text(&client.name).size(18).color(Color::WHITE),
            conn_type_badge
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        connection_info,
        status_indicator
    ]
    .spacing(5);

    // Apply the custom Card Style
    container(card_content)
        .width(Length::Fill)
        .padding(15)
        .style(card_style)
        .into()
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
        border: Border {
            color: Color::from_rgb(0.3, 0.3, 0.35),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

fn badge_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgb(0.4, 0.4, 0.5))),
        border: Border {
            color: Color::from_rgb(0.5, 0.5, 0.6),
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}