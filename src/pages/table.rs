use crate::message::Message;
use iced::{
    Border, Color, Element, Length, Padding, Shadow, Theme,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{Space, button, column, container, row, scrollable, text},
};
use dlt_format_parser::DltFormat;

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
    pub fn new(
        index: u32,
        timestamp: String,
        ecu_id: String,
        app_id: String,
        context_id: String,
        message_type: String,
        payload: String,
        length: usize,
    ) -> Self {
        Self {
            index,
            timestamp,
            ecu_id,
            app_id,
            context_id,
            message_type,
            payload,
            length,
        }
    }

    pub fn from_dlt_format(
        dlt_format: &DltFormat,
    ) -> Self {
        Self {
            index: 0, // Will be set later
            timestamp: dlt_format.get_timestamp_string(),
            ecu_id: dlt_format.standard_header_extra.get_ecu().trim_end_matches('\0').to_string(),
            app_id: dlt_format.extended_header.get_apid().trim_end_matches('\0').to_string(),
            context_id: dlt_format.extended_header.get_ctid().trim_end_matches('\0').to_string(),
            message_type: "Unknown".to_string(),
            payload: dlt_format_parser::MessageList::parse(&dlt_format.payload, dlt_format.payload.len()).get_entire_string(),
            length: dlt_format.payload.len(),
        }
    }
}

pub fn view<'a>(
    dark_mode: bool,
    messages: &'a [DltMessageRow],
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
            Space::with_height(Length::Fixed(4.0)),
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

    let table_card = create_table_card(dark_mode, messages);

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

fn create_table_card<'a>(
    dark_mode: bool,
    messages: &'a [DltMessageRow],
) -> Element<'a, Message> {
    let theme = if dark_mode { Theme::Dark } else { Theme::Light };
    
    // Table header
    let header = container(
        row![
            header_cell("#", 60.0),
            header_cell("Timestamp", 140.0),
            header_cell("ECU", 70.0),
            header_cell("App ID", 70.0),
            header_cell("Context", 70.0),
            header_cell("Type", 90.0),
            header_cell("Length", 70.0),
            header_cell("Payload", 400.0),
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

    // Table rows
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
                Space::with_height(Length::Fixed(8.0)),
                text::<Theme, _>("Connect to a TCP server to receive DLT messages")
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
        for msg in messages.iter() {
            let row_widget = create_table_row(msg, dark_mode);
            table_rows = table_rows.push(row_widget);
        }
    }

    let scrollable_content = scrollable(
        column![
            header,
            table_rows,
        ]
        .spacing(0)
    )
    .height(Length::Fill);

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

fn header_cell<'a>(label: &'static str, width: f32) -> Element<'a, Message> {
    container(
        text::<Theme, _>(label)
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(if matches!(theme, Theme::Dark) {
                    Color::from_rgb(0.8, 0.8, 0.8)
                } else {
                    Color::from_rgb(0.3, 0.3, 0.3)
                }),
            }),
    )
    .width(Length::Fixed(width))
    .into()
}

fn create_table_row<'a>(
    msg: &'a DltMessageRow,
    dark_mode: bool,
) -> Element<'a, Message> {
    let is_even = msg.index % 2 == 0;
    
    // Create owned strings for numeric values
    let index_str = msg.index.to_string();
    let length_str = msg.length.to_string();
    
    container(
        row![
            table_cell_owned(index_str, 60.0),
            table_cell(&msg.timestamp, 140.0),
            table_cell(&msg.ecu_id, 70.0),
            table_cell(&msg.app_id, 70.0),
            table_cell(&msg.context_id, 70.0),
            table_cell(&msg.message_type, 90.0),
            table_cell_owned(length_str, 70.0),
            table_cell(&msg.payload, 400.0),
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
    .style(move |theme: &Theme| {
        let bg_color = if matches!(theme, Theme::Dark) {
            if is_even {
                Color::from_rgb(0.12, 0.12, 0.13)
            } else {
                Color::from_rgb(0.14, 0.14, 0.15)
            }
        } else {
            if is_even {
                Color::WHITE
            } else {
                Color::from_rgb(0.98, 0.98, 0.99)
            }
        };

        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                color: if matches!(theme, Theme::Dark) {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.03)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.03)
                },
                width: 0.0,
                radius: Radius::from(0.0),
            },
            ..Default::default()
        }
    })
    .into()
}

fn table_cell<'a>(content: &'a str, width: f32) -> Element<'a, Message> {
    container(
        text::<Theme, _>(content)
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(if matches!(theme, Theme::Dark) {
                    Color::from_rgb(0.85, 0.85, 0.85)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.2)
                }),
            }),
    )
    .width(Length::Fixed(width))
    .into()
}

fn table_cell_owned<'a>(content: String, width: f32) -> Element<'a, Message> {
    container(
        text::<Theme, _>(content)
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(if matches!(theme, Theme::Dark) {
                    Color::from_rgb(0.85, 0.85, 0.85)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.2)
                }),
            }),
    )
    .width(Length::Fixed(width))
    .into()
}