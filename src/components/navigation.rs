use crate::{app::ICON_FONT, message::{Message, Page}, plugin_registry::PluginRegistry};
use iced::{
    Alignment, Border, Color, Element, Length, Theme, widget::{Button, button, column, container, row, text}
};


// Font Awesome Unicode characters
const ICON_OVERVIEW: &str = "\u{f080}"; // chart-bar
const ICON_ANALYTICS: &str = "\u{f015}"; // chart-line  
const ICON_REPORTS: &str = "\u{f15c}"; // file
const ICON_SETTINGS: &str = "\u{f013}"; // gear/cog

pub fn view(current_page: Page, plugin: &PluginRegistry, dark_mode: bool) -> Element<'static, Message> {
    let theme_color = if dark_mode {
        Color::from_rgb(0.1, 0.1, 0.1)
    } else {
        Color::from_rgb(0.95, 0.95, 0.95)
    };

    // Use owned Strings for labels so they don't borrow from a temporary collection.
    let base_nav_items: Vec<(&'static str, String, Page)> = vec![
        (ICON_OVERVIEW, String::from("Overview"), Page::Overview),
        (ICON_REPORTS, String::from("Reports"), Page::Reports),
        (ICON_SETTINGS, String::from("Settings"), Page::Settings),
        ("\u{f0ce}", String::from("Table"), Page::Table),
        ("\u{f43c}", String::from("Chart Canvas"), Page::ChartCanvas),
    ];

    let plugin_nav_items: Vec<(&'static str, String, Page)> = plugin.plugin_names().iter().map(|name| {
        // For simplicity, use the same name for icon and label, own the label String
        (ICON_ANALYTICS, name.clone(), Page::PluginPage(name.clone()))
    }).collect();

    let nav_items: Vec<(&'static str, String, Page)> = [base_nav_items, plugin_nav_items].concat();

    let mut nav_buttons = column![].spacing(5).padding(10).align_x(Alignment::Start);

    for (icon, label, page) in nav_items {
        let is_active = current_page == page;
        let background = if is_active {
            Color::from_rgb(0.2, 0.4, 0.8)
        } else if dark_mode {
            Color::from_rgb(0.15, 0.15, 0.15)
        } else {
            Color::from_rgb(0.9, 0.9, 0.9)
        };

        let text_color = if is_active {
            Color::WHITE
        } else if dark_mode {
            Color::from_rgb(0.7, 0.7, 0.7)
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };

        let btn: button::Button<'_, Message> = button(
            row![
                text(icon)
                    .font(ICON_FONT)
                    .size(16)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(text_color),
                    }),
                text(label)
                    .size(16)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(text_color),
                    }),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .on_press(Message::NavigateTo(page))
        .padding(12)
        .width(Length::Fill)
        .style(
            move |_theme: &Theme, _status: button::Status| button::Style {
                background: Some(background.into()),
                border: Border {
                    radius: 5.0.into(),
                    ..Default::default()
                },
                text_color,
                ..Default::default()
            },
        );

        nav_buttons = nav_buttons.push(btn);
    }

    container(nav_buttons)
        .width(200)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(theme_color.into()),
            border: Border {
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
