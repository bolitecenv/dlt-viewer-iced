use iced::{Background, Border, Color, Element, Length, Shadow, Theme};
use iced::widget::{Button, Text, button, column, container, Container, row, scrollable, text, text_input};
use crate::components::tcp_handler::{TCPClientsHandler, TCPClient};
use crate::message::Message;

/// The main view function for this page
pub fn view<'a>(
    clients_handler: &'a TCPClientsHandler,
    current_ip: &'a str,
    current_port: &'a str,
) -> Element<'a, Message> {
    
    // 1. The Input Row (IP, Port, + Button)
    let ip_input = text_input("IP Address", current_ip)
        .on_input(Message::IpChanged)
        .padding(10)
        .width(Length::Fixed(200.0));

    let port_input = text_input("Port", current_port)
        .on_input(Message::PortChanged)
        .padding(10)
        .width(Length::Fixed(100.0));

    let add_button = button(text("+ Add Daemon"))
        .on_press(Message::ConnectTcp) // Sends the Message when clicked
        .padding(10)
        .style(button::primary);

    let controls = row![ip_input, port_input, add_button]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // 2. The List of Cards
    // We iterate over the clients in the HashMap and create a card for each
    let client_cards = clients_handler.clients.values().fold(
        column![].spacing(10), 
        |col, client| col.push(view_client_card(client))
    );

    // 3. Assemble the Page
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
    ].spacing(5).align_y(iced::Alignment::Center);

    // Card Content
    let card_content = column![
        text(&client.name).size(18).color(Color::WHITE),
        row![
            text(format!("IP: {}", client.config.ip)).size(12),
            text("|").size(12),
            text(format!("Port: {}", client.config.port)).size(12),
        ].spacing(10),
        status_indicator
    ].spacing(5);

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