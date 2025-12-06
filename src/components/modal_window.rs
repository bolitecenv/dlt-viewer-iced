use iced::widget::{center, mouse_area, opaque, scrollable};
use iced::{
    Color, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, text, Space
    },
    Font,
};
use crate::app::ICON_FONT;
use crate::message::Message;

pub struct ModalConfig {
    pub width: f32,
    pub height: f32,
    pub can_close: bool,
    pub can_apply: bool,
    pub show_refresh: bool,
    pub show_apply: bool,
    pub title: String,
}

impl Default for ModalConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            can_close: true,
            can_apply: true,
            show_refresh: false,
            show_apply: true,
            title: "Modal Window".to_string(),
        }
    }
}

pub trait ModalWindowView {
    fn title(&self) -> String;
    fn get_config(&self) -> ModalConfig;
    fn content(&self) -> Element<'_, Message>;
    fn close_message(&self) -> Message;
    fn refresh_message(&self) -> Option<Message>;
    fn apply_message(&self) -> Option<Message>;
    fn update(&self, message: String) -> Option<Message>;

    fn draw(&self, dark_mode: bool) -> Element<'_, Message> {
        self.modal_window_view(dark_mode)
    }

    fn divider(&self, dark_mode: bool) -> Element<'static, Message> {
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(if dark_mode {
                    Color::from_rgb(0.3, 0.3, 0.3)
                } else {
                    Color::from_rgb(0.7, 0.7, 0.7)
                }.into()),
                ..Default::default()
            })
            .into()
    }

    fn modal_window_view(&self, dark_mode: bool) -> Element<'_, Message> {
        let config = self.get_config();
        let content = self.content();

        let mut footer_row = row![].spacing(10).width(Length::Fill);
        
        if config.show_refresh {
            if let Some(refresh_msg) = self.refresh_message() {
                footer_row = footer_row.push(
                    button(text("Refresh").size(14))
                        .on_press(refresh_msg)
                        .padding(10)
                );
            }
        }
        
        footer_row = footer_row.push(Space::new(Length::Fill, Length::Shrink));
        
        if config.show_apply {
            if let Some(apply_msg) = self.apply_message() {
                let apply_button = button(text("Apply").size(14))
                    .padding(10)
                    .style(move |theme: &Theme, status| {
                        let base_style = button::primary(theme, status);
                        button::Style {
                            background: Some(Color::from_rgb(0.2, 0.6, 0.2).into()),
                            text_color: Color::WHITE,
                            ..base_style
                        }
                    });
                
                footer_row = footer_row.push(
                    if config.can_apply {
                        apply_button.on_press(apply_msg)
                    } else {
                        apply_button
                    }
                );
            }
        }

        // Build header row with conditional close button
        let mut header_row = row![
            text(config.title.clone()).size(20).color(if dark_mode {
                Color::WHITE
            } else {
                Color::BLACK
            }),
            Space::new(Length::Fill, Length::Shrink),
        ]
        .spacing(20)
        .align_y(Vertical::Center);

        if config.can_close {
            header_row = header_row.push(
                button(text("✕").font(ICON_FONT).size(16))
                    .on_press(self.close_message())
                    .padding([4, 8])
                    .style(move |theme: &Theme, status| {
                        let base_style = button::primary(theme, status);
                        button::Style {
                            background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
                            text_color: Color::WHITE,
                            ..base_style
                        }
                    })
            );
        }

        // Main popup content
        let popup_content = container(
            column![
                // Header
                header_row,
                
                // Divider
                self.divider(dark_mode),

                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                // Main content (provided by implementation) - takes up all available space
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill),

                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                // Divider before footer
                self.divider(dark_mode),
                
                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                // Footer buttons
                footer_row
            ]
            .spacing(0)
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
    }
}