// components/ecu_list_view.rs
// ECU List View with tree navigation and settings panel

use iced::{
    Color, Element, Length, Theme,
    alignment::Vertical,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input, Column, Row,
    }
};

// Import your types - adjust the path as needed for your project structure
use crate::types::{FrontDltEcuItem, FrontDltAppIdItem, FrontDltCtxIdItem};
use crate::message::Message;

/// Selection state for the ECU tree
#[derive(Debug, Clone, PartialEq)]
pub enum EcuSelection {
    None,
    Ecu(String),
    App(String, String), // (ecu_id, app_id)
    Context(String, String, String), // (ecu_id, app_id, context_id)
}

/// Editing state for context parameters and injection messages
#[derive(Debug, Clone)]
pub struct EcuEditState {
    pub log_level_input: String,
    pub trace_status_input: String,
    pub message_input: String,
    pub message_type_input: String,
}

impl Default for EcuEditState {
    fn default() -> Self {
        Self {
            log_level_input: String::new(),
            trace_status_input: String::new(),
            message_input: String::new(),
            message_type_input: "DLT_TYPE_LOG".to_string(),
        }
    }
}

/// Main ECU List View component
pub struct EcuListView {
    pub ecu_list: Vec<FrontDltEcuItem>,
    pub selected_item: EcuSelection,
    pub expanded_ecus: Vec<String>,
    pub expanded_apps: Vec<(String, String)>, // (ecu_id, app_id)
    pub edit_state: EcuEditState,
    pub is_editing: bool,
}

impl EcuListView {
    pub fn new(ecu_list: Vec<FrontDltEcuItem>) -> Self {
        Self {
            ecu_list,
            selected_item: EcuSelection::None,
            expanded_ecus: Vec::new(),
            expanded_apps: Vec::new(),
            edit_state: EcuEditState::default(),
            is_editing: false,
        }
    }

    /// Set or update the ECU list
    pub fn set_ecu_list(&mut self, ecu_list: Vec<FrontDltEcuItem>) {
        self.ecu_list = ecu_list;
    }

    /// Toggle ECU expansion state
    pub fn toggle_ecu(&mut self, ecu_id: String) {
        if let Some(pos) = self.expanded_ecus.iter().position(|x| x == &ecu_id) {
            self.expanded_ecus.remove(pos);
        } else {
            self.expanded_ecus.push(ecu_id);
        }
    }

    /// Toggle App expansion state
    pub fn toggle_app(&mut self, ecu_id: String, app_id: String) {
        let key = (ecu_id, app_id);
        if let Some(pos) = self.expanded_apps.iter().position(|x| x == &key) {
            self.expanded_apps.remove(pos);
        } else {
            self.expanded_apps.push(key);
        }
    }

    /// Select an item in the tree
    pub fn select_item(&mut self, selection: EcuSelection) {
        self.selected_item = selection;
        self.is_editing = false;
    }

    /// Start editing context settings
    pub fn start_editing(&mut self, log_level: i8, trace_status: i8) {
        self.is_editing = true;
        self.edit_state.log_level_input = log_level.to_string();
        self.edit_state.trace_status_input = trace_status.to_string();
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.is_editing = false;
        self.edit_state = EcuEditState::default();
    }

    /// Update log level input
    pub fn update_log_level(&mut self, value: String) {
        self.edit_state.log_level_input = value;
    }

    /// Update trace status input
    pub fn update_trace_status(&mut self, value: String) {
        self.edit_state.trace_status_input = value;
    }

    pub fn update_context_settings(&mut self, ecu_id: String,
                                     app_id: String, 
                                     ctx_id: String,
                                     log_level: i8,
                                     trace_status: i8) {
        self.edit_state.log_level_input = log_level.to_string();
        self.edit_state.trace_status_input = trace_status.to_string();
    }

    /// Update injection message input
    pub fn update_message(&mut self, value: String) {
        self.edit_state.message_input = value;
    }

    /// Update message type input
    pub fn update_message_type(&mut self, value: String) {
        self.edit_state.message_type_input = value;
    }

    /// Clear injection message
    pub fn clear_message(&mut self) {
        self.edit_state.message_input.clear();
    }

    /// Main view function
    pub fn view<'a>(&'a self, dark_mode: bool) -> Element<'a, Message> {
        let tree_view = self.build_tree_view(dark_mode);
        let detail_view = self.build_detail_view(dark_mode);

        row![
            // Left panel - Tree view
            self.create_panel(
                scrollable(tree_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dark_mode,
                Length::FillPortion(2),
            ),
            
            Space::new(Length::Fixed(10.0), Length::Shrink),
            
            // Right panel - Detail view
            self.create_panel(
                scrollable(detail_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dark_mode,
                Length::FillPortion(3),
            ),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Create a styled panel container
    fn create_panel<'a>(
        &'a self,
        content: impl Into<Element<'a, Message>>,
        dark_mode: bool,
        width: Length,
    ) -> Element<'a, Message> {
        container(content)
            .width(width)
            .height(Length::Fill)
            .padding(15)
            .style(move |_theme: &Theme| {
                container::Style {
                    background: Some(if dark_mode {
                        Color::from_rgb(0.15, 0.15, 0.15).into()
                    } else {
                        Color::from_rgb(0.95, 0.95, 0.95).into()
                    }),
                    border: iced::Border {
                        width: 1.0,
                        color: if dark_mode {
                            Color::from_rgb(0.3, 0.3, 0.3)
                        } else {
                            Color::from_rgb(0.8, 0.8, 0.8)
                        },
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    /// Build the tree view (left panel)
    fn build_tree_view<'a>(&'a self, _dark_mode: bool) -> Element<'a, Message> {
        let mut tree_column = column![].spacing(2);

        for ecu in self.ecu_list.iter() {
            let is_expanded = self.expanded_ecus.contains(&ecu.ecuid);
            let is_selected = matches!(&self.selected_item, EcuSelection::Ecu(id) if id == &ecu.ecuid);
            
            // ECU item
            let ecu_id_clone = ecu.ecuid.clone();
            let ecu_button = button(
                row![
                    text(if is_expanded { "▼" } else { "▶" }).size(12),
                    Space::new(Length::Fixed(5.0), Length::Shrink),
                    text(format!("ECU: {}", ecu.ecuid.clone())).size(14),
                ]
                .align_y(Vertical::Center)
            )
            .on_press(Message::SelectEcu(ecu_id_clone.clone()))
            .padding([4, 8])
            .width(Length::Fill)
            .style(move |theme: &Theme, status| {
                let base_style = button::secondary(theme, status);
                button::Style {
                    background: if is_selected {
                        Some(Color::from_rgba(0.3, 0.5, 0.8, 0.3).into())
                    } else {
                        base_style.background
                    },
                    ..base_style
                }
            });

            tree_column = tree_column.push(ecu_button);

            // Show apps if ECU is expanded
            if is_expanded {
                for app in &ecu.app_ids {
                    let app_key = (ecu.ecuid.clone(), app.apid.clone());
                    let is_app_expanded = self.expanded_apps.contains(&app_key);
                    let is_app_selected = matches!(&self.selected_item, 
                        EcuSelection::App(eid, aid) if eid == &ecu.ecuid && aid == &app.apid);
                    
                    let ecu_id_clone = ecu.ecuid.clone();
                    let app_id_clone = app.apid.clone();
                    
                    // App item (indented)
                    let app_button = button(
                        row![
                            Space::new(Length::Fixed(20.0), Length::Shrink),
                            text(if is_app_expanded { "▼" } else { "▶" }).size(12),
                            Space::new(Length::Fixed(5.0), Length::Shrink),
                            text(format!("App: {}", app.apid.clone())).size(14),
                        ]
                        .align_y(Vertical::Center)
                    )
                    .on_press(Message::SelectApp(ecu_id_clone.clone(), app_id_clone.clone()))
                    .padding([4, 8])
                    .width(Length::Fill)
                    .style(move |theme: &Theme, status| {
                        let base_style = button::secondary(theme, status);
                        button::Style {
                            background: if is_app_selected {
                                Some(Color::from_rgba(0.3, 0.5, 0.8, 0.3).into())
                            } else {
                                base_style.background
                            },
                            ..base_style
                        }
                    });

                    tree_column = tree_column.push(app_button);

                    // Show contexts if App is expanded
                    if is_app_expanded {
                        for ctx in &app.ctx_ids {
                            let is_ctx_selected = matches!(&self.selected_item,
                                EcuSelection::Context(eid, aid, cid) 
                                if eid == &ecu.ecuid && aid == &app.apid && cid == &ctx.context_id);
                            
                            let ecu_id_clone = ecu.ecuid.clone();
                            let app_id_clone = app.apid.clone();
                            let ctx_id_clone = ctx.context_id.clone();
                            
                            // Context item (double indented)
                            let ctx_button = button(
                                row![
                                    Space::new(Length::Fixed(40.0), Length::Shrink),
                                    text("•").size(12),
                                    Space::new(Length::Fixed(5.0), Length::Shrink),
                                    text(format!("Ctx: {}", ctx.context_id.clone())).size(14),
                                ]
                                .align_y(Vertical::Center)
                            )
                            .on_press(Message::SelectContext(
                                ecu_id_clone, 
                                app_id_clone, 
                                ctx_id_clone
                            ))
                            .padding([4, 8])
                            .width(Length::Fill)
                            .style(move |theme: &Theme, status| {
                                let base_style = button::secondary(theme, status);
                                button::Style {
                                    background: if is_ctx_selected {
                                        Some(Color::from_rgba(0.3, 0.5, 0.8, 0.3).into())
                                    } else {
                                        base_style.background
                                    },
                                    ..base_style
                                }
                            });

                            tree_column = tree_column.push(ctx_button);
                        }
                    }
                }
            }
        }

        tree_column.into()
    }

    /// Build the detail view (right panel)
    fn build_detail_view<'a>(&'a self, dark_mode: bool) -> Element<'a, Message> {
        let text_color = if dark_mode { Color::WHITE } else { Color::BLACK };
        let label_color = if dark_mode { 
            Color::from_rgb(0.7, 0.7, 0.7) 
        } else { 
            Color::from_rgb(0.5, 0.5, 0.5) 
        };

        match &self.selected_item {
            EcuSelection::None => {
                column![
                    text("Select an item to view details").size(16).color(label_color),
                ]
                .into()
            }
            EcuSelection::Ecu(ecu_id) => {
                self.build_ecu_detail(ecu_id, text_color, label_color)
            }
            EcuSelection::App(ecu_id, app_id) => {
                self.build_app_detail(ecu_id, app_id, text_color, label_color)
            }
            EcuSelection::Context(ecu_id, app_id, ctx_id) => {
                self.build_context_detail(ecu_id, app_id, ctx_id, dark_mode, text_color, label_color)
            }
        }
    }

    /// Build ECU detail view
    fn build_ecu_detail<'a>(
        &'a self,
        ecu_id: &'a str,
        text_color: Color,
        label_color: Color,
    ) -> Element<'a, Message> {
        if let Some(ecu) = self.ecu_list.iter().find(|e| e.ecuid == ecu_id) {
            column![
                text("ECU Information").size(20).color(text_color),
                Space::new(Length::Shrink, Length::Fixed(20.0)),
                
                row![
                    text("ECU ID:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(&ecu.ecuid).size(14).color(text_color),
                ],
                
                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                row![
                    text("Description:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(&ecu.description).size(14).color(text_color),
                ],
                
                Space::new(Length::Shrink, Length::Fixed(10.0)),
                
                row![
                    text("Applications:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(format!("{}", ecu.app_ids.len())).size(14).color(text_color),
                ],
            ]
            .spacing(5)
            .into()
        } else {
            text("ECU not found").size(14).color(text_color).into()
        }
    }

    /// Build App detail view
    fn build_app_detail<'a>(
        &'a self,
        ecu_id: &'a str,
        app_id: &'a str,
        text_color: Color,
        label_color: Color,
    ) -> Element<'a, Message> {
        if let Some(ecu) = self.ecu_list.iter().find(|e| e.ecuid == ecu_id) {
            if let Some(app) = ecu.app_ids.iter().find(|a| a.apid == app_id) {
                return column![
                    text("Application Information").size(20).color(text_color),
                    Space::new(Length::Shrink, Length::Fixed(20.0)),
                    
                    row![
                        text("App ID:").size(14).color(label_color).width(Length::Fixed(140.0)),
                        text(&app.apid).size(14).color(text_color),
                    ],
                    
                    Space::new(Length::Shrink, Length::Fixed(10.0)),
                    
                    row![
                        text("Description:").size(14).color(label_color).width(Length::Fixed(140.0)),
                        text(&app.description).size(14).color(text_color),
                    ],
                    
                    Space::new(Length::Shrink, Length::Fixed(10.0)),
                    
                    row![
                        text("Parent ECU:").size(14).color(label_color).width(Length::Fixed(140.0)),
                        text(ecu_id).size(14).color(text_color),
                    ],
                    
                    Space::new(Length::Shrink, Length::Fixed(10.0)),
                    
                    row![
                        text("Contexts:").size(14).color(label_color).width(Length::Fixed(140.0)),
                        text(format!("{}", app.ctx_ids.len())).size(14).color(text_color),
                    ],
                ]
                .spacing(5)
                .into();
            }
        }
        text("Application not found").size(14).color(text_color).into()
    }

    /// Build Context detail view with settings and injection
    fn build_context_detail<'a>(
        &'a self,
        ecu_id: &'a str,
        app_id: &'a str,
        ctx_id: &'a str,
        dark_mode: bool,
        text_color: Color,
        label_color: Color,
    ) -> Element<'a, Message> {
        if let Some(ecu) = self.ecu_list.iter().find(|e| e.ecuid == ecu_id) {
            if let Some(app) = ecu.app_ids.iter().find(|a| a.apid == app_id) {
                if let Some(ctx) = app.ctx_ids.iter().find(|c| c.context_id == ctx_id) {
                    return self.build_context_settings_and_injection(
                        ecu_id,
                        app_id,
                        ctx,
                        dark_mode,
                        text_color,
                        label_color,
                    );
                }
            }
        }
        text("Context not found").size(14).color(text_color).into()
    }

    /// Build full context view with settings and injection message
    fn build_context_settings_and_injection<'a>(
        &'a self,
        ecu_id: &'a str,
        app_id: &'a str,
        ctx: &'a FrontDltCtxIdItem,
        dark_mode: bool,
        text_color: Color,
        label_color: Color,
    ) -> Element<'a, Message> {
        let log_level_str = match ctx.log_level {
            0 => "OFF",
            1 => "FATAL",
            2 => "ERROR",
            3 => "WARN",
            4 => "INFO",
            5 => "DEBUG",
            6 => "VERBOSE",
            _ => "UNKNOWN",
        };
        
        let trace_status_str = match ctx.trace_status {
            0 => "OFF",
            1 => "ON",
            _ => "UNKNOWN",
        };

        let mut content = column![
            text("Context Settings").size(20).color(text_color),
            Space::new(Length::Shrink, Length::Fixed(20.0)),
            
            row![
                text("Context ID:").size(14).color(label_color).width(Length::Fixed(140.0)),
                text(&ctx.context_id).size(14).color(text_color),
            ],
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),
            
            row![
                text("Description:").size(14).color(label_color).width(Length::Fixed(140.0)),
                text(&ctx.description).size(14).color(text_color),
            ],
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),
        ]
        .spacing(5);

        // Log Level - editable or display-only
        if self.is_editing {
            content = content.push(
                row![
                    text("Log Level:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text_input("0-6", &self.edit_state.log_level_input)
                        .on_input(Message::UpdateLogLevel)
                        .size(14)
                        .width(Length::Fixed(100.0))
                        .style(move |theme: &Theme, status| {
                            self.text_input_style(theme, status, dark_mode)
                        }),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    text(format!("({})", log_level_str)).size(14).color(label_color),
                ]
            );
        } else {
            content = content.push(
                row![
                    text("Log Level:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(format!("{} ({})", ctx.log_level, log_level_str))
                        .size(14)
                        .color(text_color),
                ]
            );
        }

        content = content.push(Space::new(Length::Shrink, Length::Fixed(10.0)));

        // Trace Status - editable or display-only
        if self.is_editing {
            content = content.push(
                row![
                    text("Trace Status:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text_input("0 or 1", &self.edit_state.trace_status_input)
                        .on_input(Message::UpdateTraceStatus)
                        .size(14)
                        .width(Length::Fixed(100.0))
                        .style(move |theme: &Theme, status| {
                            self.text_input_style(theme, status, dark_mode)
                        }),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    text(format!("({})", trace_status_str)).size(14).color(label_color),
                ]
            );
        } else {
            content = content.push(
                row![
                    text("Trace Status:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(format!("{} ({})", ctx.trace_status, trace_status_str))
                        .size(14)
                        .color(text_color),
                ]
            );
        }

        content = content
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(
                row![
                    text("Parent App:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(app_id).size(14).color(text_color),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(
                row![
                    text("Parent ECU:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(ecu_id).size(14).color(text_color),
                ]
            );

        // Edit/Save/Cancel buttons
        content = content
            .push(Space::new(Length::Shrink, Length::Fixed(20.0)))
            .push(
                if self.is_editing {
                    row![
                        button(text("Save").size(14))
                            .on_press(Message::SaveContextSettings)
                            .padding(10)
                            .style(move |theme: &Theme, status| {
                                button::Style {
                                    background: Some(Color::from_rgb(0.2, 0.6, 0.2).into()),
                                    text_color: Color::WHITE,
                                    ..button::primary(theme, status)
                                }
                            }),
                        Space::new(Length::Fixed(10.0), Length::Shrink),
                        button(text("Cancel").size(14))
                            .on_press(Message::CancelEditContext)
                            .padding(10),
                    ]
                } else {
                    row![
                        button(text("Edit Settings").size(14))
                            .on_press(Message::ECUViewEditContext(ctx.log_level, ctx.trace_status))
                            .padding(10),
                    ]
                }
            );

        // Divider
        content = content
            .push(Space::new(Length::Shrink, Length::Fixed(30.0)))
            .push(
                container(Space::new(Length::Fill, Length::Fixed(1.0)))
                    .style(move |_theme: &Theme| {
                        container::Style {
                            background: Some(if dark_mode {
                                Color::from_rgb(0.4, 0.4, 0.4).into()
                            } else {
                                Color::from_rgb(0.8, 0.8, 0.8).into()
                            }),
                            ..Default::default()
                        }
                    })
            )
            .push(Space::new(Length::Shrink, Length::Fixed(20.0)));

        // Injection Message Settings
        content = content
            .push(text("Injection Message Settings").size(20).color(text_color))
            .push(Space::new(Length::Shrink, Length::Fixed(15.0)))
            .push(
                row![
                    text("Target ECU:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(ecu_id).size(14).color(text_color),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(8.0)))
            .push(
                row![
                    text("Target App:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(app_id).size(14).color(text_color),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(8.0)))
            .push(
                row![
                    text("Target Context:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text(&ctx.context_id).size(14).color(text_color),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(15.0)))
            .push(
                row![
                    text("Message Type:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    text_input("DLT_TYPE_LOG", &self.edit_state.message_type_input)
                        .on_input(Message::UpdateMessageType)
                        .size(14)
                        .width(Length::Fixed(200.0))
                        .style(move |theme: &Theme, status| {
                            self.text_input_style(theme, status, dark_mode)
                        }),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(
                column![
                    row![
                        text("Message:").size(14).color(label_color).width(Length::Fixed(140.0)),
                    ],
                    text_input("Enter message to inject...", &self.edit_state.message_input)
                        .on_input(Message::UpdateInjectionMessage)
                        .size(14)
                        .width(Length::Fill)
                        .style(move |theme: &Theme, status| {
                            self.text_input_style(theme, status, dark_mode)
                        }),
                ]
                .spacing(5)
            )
            .push(Space::new(Length::Shrink, Length::Fixed(20.0)))
            .push(
                row![
                    button(text("Inject Message").size(14))
                        .on_press(Message::InjectMessage(
                            ecu_id.to_string(),
                            app_id.to_string(),
                            ctx.context_id.clone(),
                            self.edit_state.message_input.clone(),
                        ))
                        .padding(10)
                        .style(move |theme: &Theme, status| {
                            button::Style {
                                background: Some(Color::from_rgb(0.2, 0.5, 0.8).into()),
                                text_color: Color::WHITE,
                                ..button::primary(theme, status)
                            }
                        }),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    button(text("Clear").size(14))
                        .on_press(Message::ClearInjectionMessage)
                        .padding(10),
                ]
            );

        content.into()
    }

    /// Helper function for text input styling
    fn text_input_style(&self, theme: &Theme, status: iced::widget::text_input::Status, dark_mode: bool) -> iced::widget::text_input::Style {
        text_input::Style {
            background: if dark_mode {
                Color::from_rgb(0.2, 0.2, 0.2).into()
            } else {
                Color::WHITE.into()
            },
            border: iced::Border {
                width: 1.0,
                color: if dark_mode {
                    Color::from_rgb(0.4, 0.4, 0.4)
                } else {
                    Color::from_rgb(0.7, 0.7, 0.7)
                },
                radius: 4.0.into(),
            },
            icon: text_input::default(theme, status).icon,
            placeholder: text_input::default(theme, status).placeholder,
            value: if dark_mode { Color::WHITE } else { Color::BLACK },
            selection: text_input::default(theme, status).selection,
        }
    }
}