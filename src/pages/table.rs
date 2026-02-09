use crate::message::Message;
use crate::components::dlt_parser::ParsedDltMessage;
use iced::{
    Border, Color, Element, Length, Padding, Shadow, Theme,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{Space, button, column, container, row, scrollable, text},
};
use iced::widget::lazy;

// Number of rows to render at once (adjust based on performance needs)
const VISIBLE_ROWS: usize = 100;
const ROW_HEIGHT: f32 = 42.0; // Approximate height of each row

#[derive(Debug, Clone)]
pub struct DltMessageRow {
    pub index: u32,
    pub timestamp: String,
    pub ecu_id: String,
    pub app_id: String,
    pub context_id: String,
    pub message_type: String,
    pub payload: String,
    pub length: usize,
}

impl DltMessageRow {
    pub fn from_parsed_message(
        parsed_msg: &ParsedDltMessage,
    ) -> Self {
        Self {
            index: 0,
            timestamp: parsed_msg.get_timestamp_string(),
            ecu_id: parsed_msg.get_ecu_id(),
            app_id: parsed_msg.get_app_id(),
            context_id: parsed_msg.get_context_id(),
            message_type: "Log".to_string(), // TODO: parse actual message type
            payload: parsed_msg.parse_payload().unwrap_or_else(|| "".to_string()),
            length: parsed_msg.payload.len(),
        }
    }
}

pub fn view<'a>(
    dark_mode: bool,
    messages: &'a [DltMessageRow],
    scroll_offset: f32, // Add this parameter to track scroll position
) -> Element<'a, Message> {
    let page_header = container(
        column![
            text::<Theme, _>("DLT Messages")
                .size(28)
                .style(|theme: &Theme| text::Style {
                    color: Some(if matches!(theme, Theme::Dark) {
                        Color::WHITE
                    } else {
                        Color::from_rgb(0.1, 0.1, 0.1)
                    }),
                }),
            Space::new().height(Length::Fixed(4.0)),
            text::<Theme, _>(format!("Total messages: {}", messages.len()))
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

    let controls = container(
        row![
            button(
                container(text::<Theme, _>("Clear All").size(14))
                    .padding(Padding {
                        top: 0.0,
                        right: 8.0,
                        bottom: 0.0,
                        left: 8.0,
                    })
                    .center_x(Length::Fill)
            )
            .on_press(Message::ClearMessages)
            .padding(Padding {
                top: 10.0,
                right: 20.0,
                bottom: 10.0,
                left: 20.0,
            })
            .style(|theme: &Theme, status| {
                let base_style = button::Style {
                    background: Some(
                        if matches!(theme, Theme::Dark) {
                            Color::from_rgb(0.9, 0.3, 0.3)
                        } else {
                            Color::from_rgb(0.95, 0.2, 0.2)
                        }
                        .into(),
                    ),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(8.0),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.95, 0.2, 0.2, 0.3),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                };

                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(
                            if matches!(theme, Theme::Dark) {
                                Color::from_rgb(0.95, 0.35, 0.35)
                            } else {
                                Color::from_rgb(0.98, 0.25, 0.25)
                            }
                            .into(),
                        ),
                        ..base_style
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(
                            if matches!(theme, Theme::Dark) {
                                Color::from_rgb(0.85, 0.25, 0.25)
                            } else {
                                Color::from_rgb(0.9, 0.15, 0.15)
                            }
                            .into(),
                        ),
                        ..base_style
                    },
                    _ => base_style,
                }
            }),
        ]
        .spacing(12)
        .align_y(Vertical::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 16.0,
        left: 0.0,
    });

    let scroll_bucket = (scroll_offset / ROW_HEIGHT) as usize;


    let table_card = lazy(
        (messages.len(), dark_mode, scroll_bucket),
        move |_| create_table_card(dark_mode, messages.to_owned(), scroll_offset)
    );

    let content = column![
        page_header,
        controls,
        table_card,
    ]
    .spacing(16)
    .padding(32)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
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

fn create_table_card(
    dark_mode: bool,
    messages: Vec<DltMessageRow>,
    scroll_offset: f32,
) -> Element<'static, Message> {
    let header = container(
        row![
            header_cell("#", 60.0, dark_mode),
            header_cell("Timestamp", 140.0, dark_mode),
            header_cell("ECU", 70.0, dark_mode),
            header_cell("App ID", 70.0, dark_mode),
            header_cell("Context", 70.0, dark_mode),
            header_cell("Type", 90.0, dark_mode),
            header_cell("Length", 70.0, dark_mode),
            header_cell("Payload", 400.0, dark_mode),
        ]
        .spacing(8)
        .padding(Padding {
            top: 12.0,
            right: 16.0,
            bottom: 12.0,
            left: 16.0,
        }),
    )
    .width(Length::Fill)
    .style(move |theme: &Theme| container::Style {
        background: Some(
            if matches!(theme, Theme::Dark) {
                Color::from_rgb(0.15, 0.15, 0.16)
            } else {
                Color::from_rgb(0.94, 0.94, 0.95)
            }
            .into(),
        ),
        border: Border {
            color: if matches!(theme, Theme::Dark) {
                Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.08)
            },
            width: 0.0,
            radius: Radius {
                top_left: 12.0,
                top_right: 12.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
        },
        ..Default::default()
    });

    let mut table_rows = column![].spacing(0);
    
    if messages.is_empty() {
        let empty_state = container(
            column![
                text::<Theme, _>("No messages yet")
                    .size(16)
                    .style(|theme: &Theme| text::Style {
                        color: Some(if matches!(theme, Theme::Dark) {
                            Color::from_rgb(0.5, 0.5, 0.5)
                        } else {
                            Color::from_rgb(0.6, 0.6, 0.6)
                        }),
                    }),
                Space::new().height(Length::Fixed(8.0)),
                text::<Theme, _>("Connect to a DLT Daemon to receive DLT messages")
                    .size(13)
                    .style(|theme: &Theme| text::Style {
                        color: Some(if matches!(theme, Theme::Dark) {
                            Color::from_rgb(0.4, 0.4, 0.4)
                        } else {
                            Color::from_rgb(0.5, 0.5, 0.5)
                        }),
                    }),
            ]
            .spacing(0)
            .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 60.0,
            right: 20.0,
            bottom: 60.0,
            left: 20.0,
        })
        .center_x(Length::Fill);
        
        table_rows = table_rows.push(empty_state);
    } else {
        // Calculate which rows should be visible based on scroll position
        let total_messages = messages.len();
        let start_idx = (scroll_offset / ROW_HEIGHT).floor() as usize;
        let start_idx = start_idx.min(total_messages.saturating_sub(1));
        let end_idx = (start_idx + VISIBLE_ROWS).min(total_messages);
        
        // Add spacer for rows before the visible range
        if start_idx > 0 {
            let spacer_height = start_idx as f32 * ROW_HEIGHT;
            table_rows = table_rows.push(
                Space::new()
                    .width(Length::Fill)
                    .height(Length::Fixed(spacer_height))
            );
        }
        
        // Pre-compute colors for better performance
        let text_color = if dark_mode {
            Color::from_rgb(0.85, 0.85, 0.85)
        } else {
            Color::from_rgb(0.2, 0.2, 0.2)
        };
        
        let even_bg = if dark_mode {
            Color::from_rgb(0.12, 0.12, 0.13)
        } else {
            Color::WHITE
        };
        
        let odd_bg = if dark_mode {
            Color::from_rgb(0.14, 0.14, 0.15)
        } else {
            Color::from_rgb(0.98, 0.98, 0.99)
        };
        
        let border_color = if dark_mode {
            Color::from_rgba(1.0, 1.0, 1.0, 0.03)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.03)
        };
        
        // Render only visible messages
        for msg in messages.iter().skip(start_idx).take(end_idx - start_idx) {
            let row_widget = create_table_row(msg, text_color, even_bg, odd_bg, border_color);
            table_rows = table_rows.push(row_widget);
        }
        
        // Add spacer for rows after the visible range
        if end_idx < total_messages {
            let spacer_height = (total_messages - end_idx) as f32 * ROW_HEIGHT;
            table_rows = table_rows.push(
                Space::new()
                    .width(Length::Fill)
                    .height(Length::Fixed(spacer_height))
            );
        }
    }

    let scrollable_content = scrollable(
        column![
            header,
            table_rows,
        ]
        .spacing(0)
    )
    .height(Length::Fill)
    .on_scroll(Message::ScrollChanged); // Add scroll event handler

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme: &Theme| container::Style {
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

fn header_cell(label: &'static str, width: f32, dark_mode: bool) -> Element<'static, Message> {
    let color = if dark_mode {
        Color::from_rgb(0.8, 0.8, 0.8)
    } else {
        Color::from_rgb(0.3, 0.3, 0.3)
    };
    
    container(
        text::<Theme, _>(label)
            .size(13)
            .color(color)
    )
    .width(Length::Fixed(width))
    .into()
}

fn create_table_row(
    msg: &DltMessageRow,
    text_color: Color,
    even_bg: Color,
    odd_bg: Color,
    border_color: Color,
) -> Element<'static, Message> {
    let is_even = msg.index % 2 == 0;
    
    let index_str = msg.index.to_string();
    let length_str = msg.length.to_string();
    let timestamp = msg.timestamp.clone();
    let ecu_id = msg.ecu_id.clone();
    let app_id = msg.app_id.clone();
    let context_id = msg.context_id.clone();
    let message_type = msg.message_type.clone();
    let payload = msg.payload.clone();
    
    let bg_color = if is_even { even_bg } else { odd_bg };
    
    container(
        row![
            table_cell_optimized(index_str, 60.0, text_color),
            table_cell_optimized(timestamp, 140.0, text_color),
            table_cell_optimized(ecu_id, 70.0, text_color),
            table_cell_optimized(app_id, 70.0, text_color),
            table_cell_optimized(context_id, 70.0, text_color),
            table_cell_optimized(message_type, 90.0, text_color),
            table_cell_optimized(length_str, 70.0, text_color),
            table_cell_optimized(payload, 400.0, text_color),
        ]
        .spacing(8)
        .padding(Padding {
            top: 10.0,
            right: 16.0,
            bottom: 10.0,
            left: 16.0,
        }),
    )
    .width(Length::Fill)
    .style(move |_theme: &Theme| {
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                color: border_color,
                width: 0.0,
                radius: Radius::from(0.0),
            },
            ..Default::default()
        }
    })
    .into()
}

fn table_cell_optimized(content: String, width: f32, text_color: Color) -> Element<'static, Message> {
    container(
        text::<Theme, _>(content)
            .size(12)
            .color(text_color)
    )
    .width(Length::Fixed(width))
    .into()
}