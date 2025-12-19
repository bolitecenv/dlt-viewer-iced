use iced::{
    widget::{container, row, text, Row, Space},
    Element, Length, alignment,
};
use crate::message::{Message, Page};

pub fn view<'a>(
    connection_status: &str,
    message_count: usize,
    current_page: &Page,
    dark_mode: bool,
) -> Element<'a, Message> {
    let (bg_color, text_color, border_color) = if dark_mode {
        ([30, 30, 30], [200, 200, 200], [60, 60, 60])
    } else {
        ([240, 240, 240], [50, 50, 50], [200, 200, 200])
    };

    // Connection status indicator
    let status_color = match connection_status {
        "Connected" => [34, 197, 94],      // green
        "Disconnected" => [239, 68, 68],   // red
        "Connecting..." => [251, 191, 36], // amber
        _ => [156, 163, 175],              // gray
    };

    let connection_indicator = container(text("●").size(12))
        .style(move |_theme| container::Style {
            text_color: Some(iced::Color::from_rgb8(
                status_color[0],
                status_color[1],
                status_color[2],
            )),
            ..Default::default()
        });

    let status_text = text(format!("Status: {}", connection_status))
        .size(14)
        .style(move |_theme| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb8(text_color[0], text_color[1], text_color[2])),
        });

    // Message count
    let msg_count = text(format!("Messages: {}", message_count))
        .size(14)
        .style(move |_theme| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb8(text_color[0], text_color[1], text_color[2])),
        });

    // Current page
    let page_name = match current_page {
        Page::Reports => "Reports",
        Page::ECUSetting => "ECU Settings",
        Page::Settings => "Settings",
        Page::Table => "Message Table",
        Page::ChartCanvas => "Chart Canvas",
        Page::PluginPage(name) => name,
    };

    let page_info = text(format!("Page: {}", page_name))
        .size(14)
        .style(move |_theme| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb8(text_color[0], text_color[1], text_color[2])),
        });

    // System time (optional)
    let time_text = text(chrono::Local::now().format("%H:%M:%S").to_string())
        .size(14)
        .style(move |_theme| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb8(text_color[0], text_color[1], text_color[2])),
        });

    let footer_content = row![
        connection_indicator,
        Space::new().width(8),
        status_text,
        Space::new().width(20),
        msg_count,
        Space::new().width(20),
        page_info,
        Space::new().width(Length::Fill),
        time_text,
    ]
    .align_y(alignment::Vertical::Center)
    .padding([8, 16]);

    container(footer_content)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(
                bg_color[0],
                bg_color[1],
                bg_color[2],
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb8(border_color[0], border_color[1], border_color[2]),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}