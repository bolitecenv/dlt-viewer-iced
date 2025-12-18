use crate::{app::ICON_FONT, message::Message};
use iced::{
    Alignment, Color, Element, Length, Theme,
    widget::{Space, button, container, row, text},
};

pub fn view(dark_mode: bool) -> Element<'static, Message> {
    let theme_color = if dark_mode {
        Color::from_rgb(0.1, 0.1, 0.1)
    } else {
        Color::from_rgb(0.95, 0.95, 0.95)
    };

    let theme_text = if dark_mode {
        row![text("\u{f186}")
            .font(ICON_FONT)
            .size(16),
            text(" Dark")
            .size(14),
            ]
    } else {
        row![text("\u{f185}")
            .font(ICON_FONT)
            .size(16),
            text(" Light")
            .size(14),
            ]
    };

    let theme_button = button(theme_text)
        .on_press(Message::ToggleTheme)
        .padding(8);

    
    let settings_text = row![text("\u{f013}")
        .font(ICON_FONT)
        .size(16),
        text(" Settings")
        .size(14),
        ];
        
    // Settings button that triggers the popup
    let settings_button = button(settings_text)
        .on_press(Message::OpenDltSettings)
        .padding(8);
    
    container(
        row![
            text("Dashboard Application").size(24).color(if dark_mode {
                Color::WHITE
            } else {
                Color::BLACK
            }),
            Space::new().width(Length::Fill).height(Length::Shrink),
            theme_button,
            settings_button,
        ]
        .spacing(10)
        .padding(15)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(move |_theme: &Theme| container::Style {
        background: Some(theme_color.into()),
        text_color: None,
        border: iced::Border {
            width: 1.0,
            color: if dark_mode {
                Color::from_rgb(0.2, 0.2, 0.2)
            } else {
                Color::from_rgb(0.85, 0.85, 0.85)
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}