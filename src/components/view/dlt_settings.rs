// components/dlt_settings.rs
// DLT Settings modal implementation

use crate::message::Message;
use crate::types::{FrontDltEcuItem, FrontDltAppIdItem, FrontDltCtxIdItem};
use crate::components::view::modal_window::{ModalWindow, ModalContent, ModalConfig, ModalState};
use iced::{
    Color, Element, Length, Theme,
    alignment::Vertical,
    widget::{
        button, column, container, row, text, scrollable, Space, text_input
    }
};

// Selection state enum
#[derive(Debug, Clone, PartialEq)]
pub enum DltSelection {
    None,
    Ecu(String),
    App(String, String), // (ecu_id, app_id)
    Context(String, String, String), // (ecu_id, app_id, context_id)
}

// Editing state for context parameters
#[derive(Debug, Clone)]
pub struct ContextEditState {
    pub log_level_input: String,
    pub trace_status_input: String,
}

impl Default for ContextEditState {
    fn default() -> Self {
        Self {
            log_level_input: String::new(),
            trace_status_input: String::new(),
        }
    }
}

pub struct DltSettingsView {
    pub modal_state: ModalState,
    pub selected_item: DltSelection,
    pub expanded_ecus: Vec<String>,
    pub expanded_apps: Vec<(String, String)>, // (ecu_id, app_id)
    pub edit_state: ContextEditState,
    pub is_editing: bool,
    pub dlt_items: Vec<FrontDltEcuItem>,
}

impl DltSettingsView {
    pub fn new() -> Self {
        Self {
            modal_state: ModalState::new(),
            selected_item: DltSelection::None,
            expanded_ecus: Vec::new(),
            expanded_apps: Vec::new(),
            edit_state: ContextEditState::default(),
            is_editing: true,
            dlt_items: Vec::new(),
        }
    }

    pub fn open(&mut self) {
        self.modal_state.open();
    }

    pub fn close(&mut self) {
        self.modal_state.close();
        self.is_editing = false;
    }

    pub fn toggle(&mut self) {
        self.modal_state.toggle();
    }

    pub fn is_open(&self) -> bool {
        self.modal_state.is_open()
    }

    pub fn set_dlt_items(&mut self, items: Vec<FrontDltEcuItem>) {
        self.dlt_items = items;
    }

    pub fn toggle_ecu(&mut self, ecu_id: String) {
        if let Some(pos) = self.expanded_ecus.iter().position(|x| x == &ecu_id) {
            self.expanded_ecus.remove(pos);
        } else {
            self.expanded_ecus.push(ecu_id);
        }
    }

    pub fn toggle_app(&mut self, ecu_id: String, app_id: String) {
        let key = (ecu_id, app_id);
        if let Some(pos) = self.expanded_apps.iter().position(|x| x == &key) {
            self.expanded_apps.remove(pos);
        } else {
            self.expanded_apps.push(key);
        }
    }

    pub fn select_item(&mut self, selection: DltSelection) {
        self.selected_item = selection;
        self.is_editing = false;
    }

    pub fn start_editing(&mut self, log_level: i8, trace_status: i8) {
        self.is_editing = true;
        self.edit_state.log_level_input = log_level.to_string();
        self.edit_state.trace_status_input = trace_status.to_string();
    }

    pub fn update_log_level(&mut self, value: String) {
        self.edit_state.log_level_input = value;
    }

    pub fn update_trace_status(&mut self, value: String) {
        self.edit_state.trace_status_input = value;
    }

    pub fn view<'a>(&self, dark_mode: bool) -> Option<Element<'a, Message>> {
        ModalWindow::view(self, dark_mode, &self.modal_state)
    }

    fn build_tree_view<'a>(&self, _dark_mode: bool) -> Element<'a, Message> {
        let mut tree_column = column![].spacing(2);

        for ecu in self.dlt_items.iter() {
            let is_expanded = self.expanded_ecus.contains(&ecu.ecuid);
            let is_selected = matches!(&self.selected_item, DltSelection::Ecu(id) if id == &ecu.ecuid);
            
            // ECU item
            let ecu_id_clone = ecu.ecuid.clone();
            let ecu_button = button(
                row![
                    text(if is_expanded { "▼" } else { "▶" }).size(12),
                    Space::new(Length::Fixed(5.0), Length::Shrink),
                    text(ecu.ecuid.clone()).size(14),
                ]
                .align_y(Vertical::Center)
            )
            .on_press(Message::SelectDltEcu(ecu_id_clone.clone()))
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
                        DltSelection::App(eid, aid) if eid == &ecu.ecuid && aid == &app.apid);
                    
                    let ecu_id_clone = ecu.ecuid.clone();
                    let app_id_clone = app.apid.clone();
                    
                    // App item (indented)
                    let app_button = button(
                        row![
                            Space::new(Length::Fixed(20.0), Length::Shrink),
                            text(if is_app_expanded { "▼" } else { "▶" }).size(12),
                            Space::new(Length::Fixed(5.0), Length::Shrink),
                            text(app.apid.clone()).size(14),
                        ]
                        .align_y(Vertical::Center)
                    )
                    .on_press(Message::SelectDltApp(ecu_id_clone.clone(), app_id_clone.clone()))
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
                                DltSelection::Context(eid, aid, cid) 
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
                                    text(ctx.context_id.clone()).size(14),
                                ]
                                .align_y(Vertical::Center)
                            )
                            .on_press(Message::SelectDltContext(
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

    fn build_detail_view<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        let text_color = if dark_mode { Color::WHITE } else { Color::BLACK };
        let label_color = if dark_mode { 
            Color::from_rgb(0.7, 0.7, 0.7) 
        } else { 
            Color::from_rgb(0.5, 0.5, 0.5) 
        };

        match &self.selected_item {
            DltSelection::None => {
                column![
                    text("Select an item to view details").size(16).color(label_color),
                ]
                .into()
            }
            DltSelection::Ecu(ecu_id) => {
                if let Some(ecu) = self.dlt_items.iter().find(|e| &e.ecuid == ecu_id) {
                    column![
                        text("ECU Information").size(18).color(text_color),
                        Space::new(Length::Shrink, Length::Fixed(20.0)),
                        
                        row![
                            text("ECU ID:").size(14).color(label_color),
                            Space::new(Length::Fixed(10.0), Length::Shrink),
                            text(ecu.ecuid.clone()).size(14).color(text_color),
                        ],
                        
                        Space::new(Length::Shrink, Length::Fixed(10.0)),
                        
                        row![
                            text("Description:").size(14).color(label_color),
                            Space::new(Length::Fixed(10.0), Length::Shrink),
                            text(ecu.description.clone()).size(14).color(text_color),
                        ],
                        
                        Space::new(Length::Shrink, Length::Fixed(10.0)),
                        
                        row![
                            text("Applications:").size(14).color(label_color),
                            Space::new(Length::Fixed(10.0), Length::Shrink),
                            text(format!("{}", ecu.app_ids.len())).size(14).color(text_color),
                        ],
                    ]
                    .spacing(5)
                    .into()
                } else {
                    text("ECU not found").size(14).color(text_color).into()
                }
            }
            DltSelection::App(ecu_id, app_id) => {
                if let Some(ecu) = self.dlt_items.iter().find(|e| &e.ecuid == ecu_id) {
                    if let Some(app) = ecu.app_ids.iter().find(|a| &a.apid == app_id) {
                        column![
                            text("Application Information").size(18).color(text_color),
                            Space::new(Length::Shrink, Length::Fixed(20.0)),
                            
                            row![
                                text("App ID:").size(14).color(label_color),
                                Space::new(Length::Fixed(10.0), Length::Shrink),
                                text(app.apid.clone()).size(14).color(text_color),
                            ],
                            
                            Space::new(Length::Shrink, Length::Fixed(10.0)),
                            
                            row![
                                text("Description:").size(14).color(label_color),
                                Space::new(Length::Fixed(10.0), Length::Shrink),
                                text(app.description.clone()).size(14).color(text_color),
                            ],
                            
                            Space::new(Length::Shrink, Length::Fixed(10.0)),
                            
                            row![
                                text("Parent ECU:").size(14).color(label_color),
                                Space::new(Length::Fixed(10.0), Length::Shrink),
                                text(ecu_id.clone()).size(14).color(text_color),
                            ],
                            
                            Space::new(Length::Shrink, Length::Fixed(10.0)),
                            
                            row![
                                text("Contexts:").size(14).color(label_color),
                                Space::new(Length::Fixed(10.0), Length::Shrink),
                                text(format!("{}", app.ctx_ids.len())).size(14).color(text_color),
                            ],
                        ]
                        .spacing(5)
                        .into()
                    } else {
                        text("Application not found").size(14).color(text_color).into()
                    }
                } else {
                    text("ECU not found").size(14).color(text_color).into()
                }
            }
            DltSelection::Context(ecu_id, app_id, ctx_id) => {
                if let Some(ecu) = self.dlt_items.iter().find(|e| &e.ecuid == ecu_id) {
                    if let Some(app) = ecu.app_ids.iter().find(|a| &a.apid == app_id) {
                        if let Some(ctx) = app.ctx_ids.iter().find(|c| &c.context_id == ctx_id) {
                            self.build_context_detail_view(dark_mode, text_color, label_color, ecu_id.clone(), app_id.clone(), ctx)
                        } else {
                            text("Context not found").size(14).color(text_color).into()
                        }
                    } else {
                        text("Application not found").size(14).color(text_color).into()
                    }
                } else {
                    text("ECU not found").size(14).color(text_color).into()
                }
            }
        }
    }

    fn build_context_detail_view<'a>(
        &self,
        dark_mode: bool,
        text_color: Color,
        label_color: Color,
        ecu_id: String,
        app_id: String,
        ctx: &FrontDltCtxIdItem,
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

        let mut detail_column = column![
            text("Context Information").size(18).color(text_color),
            Space::new(Length::Shrink, Length::Fixed(20.0)),
            
            row![
                text("Context ID:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text(ctx.context_id.clone()).size(14).color(text_color),
            ],
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),
            
            row![
                text("Description:").size(14).color(label_color).width(Length::Fixed(120.0)),
                text(ctx.description.clone()).size(14).color(text_color),
            ],
            
            Space::new(Length::Shrink, Length::Fixed(10.0)),
        ]
        .spacing(5);

        // Log Level - editable or display-only
        if self.is_editing {
            detail_column = detail_column.push(
                row![
                    text("Log Level:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text_input("0-6", &self.edit_state.log_level_input)
                        .on_input(Message::UpdateLogLevel)
                        .size(14)
                        .width(Length::Fixed(100.0))
                        .style(move |theme: &Theme, status| {
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
                        }),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    text(format!("({})", log_level_str)).size(14).color(label_color),
                ]
            );
        } else {
            detail_column = detail_column.push(
                row![
                    text("Log Level:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text(format!("{} ({})", log_level_str, ctx.log_level))
                        .size(14)
                        .color(text_color),
                ]
            );
        }

        detail_column = detail_column.push(Space::new(Length::Shrink, Length::Fixed(10.0)));

        // Trace Status - editable or display-only
        if self.is_editing {
            detail_column = detail_column.push(
                row![
                    text("Trace Status:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text_input("0 or 1", &self.edit_state.trace_status_input)
                        .on_input(Message::UpdateTraceStatus)
                        .size(14)
                        .width(Length::Fixed(100.0))
                        .style(move |theme: &Theme, status| {
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
                        }),
                    Space::new(Length::Fixed(10.0), Length::Shrink),
                    text(format!("({})", trace_status_str)).size(14).color(label_color),
                ]
            );
        } else {
            detail_column = detail_column.push(
                row![
                    text("Trace Status:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text(format!("{} ({})", trace_status_str, ctx.trace_status))
                        .size(14)
                        .color(text_color),
                ]
            );
        }

        detail_column = detail_column
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(
                row![
                    text("Parent App:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text(app_id.clone()).size(14).color(text_color),
                ]
            )
            .push(Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(
                row![
                    text("Parent ECU:").size(14).color(label_color).width(Length::Fixed(120.0)),
                    text(ecu_id.clone()).size(14).color(text_color),
                ]
            );

        // Edit/Save button
        detail_column = detail_column
            .push(Space::new(Length::Shrink, Length::Fixed(30.0)))
            .push(
                if self.is_editing {
                    row![
                        button(text("Save").size(14))
                            .on_press(Message::SaveContextSettings)
                            .padding(10)
                            .style(move |theme: &Theme, status| {
                                let base_style = button::primary(theme, status);
                                button::Style {
                                    background: Some(Color::from_rgb(0.2, 0.6, 0.2).into()),
                                    text_color: Color::WHITE,
                                    ..base_style
                                }
                            }),
                        Space::new(Length::Fixed(10.0), Length::Shrink),
                        button(text("Cancel").size(14))
                            .on_press(Message::CancelEditContext)
                            .padding(10),
                    ]
                } else {
                    row![
                        button(text("Edit").size(14))
                            .on_press(Message::EditContext(ctx.log_level, ctx.trace_status))
                            .padding(10),
                    ]
                }
            );

        detail_column.into()
    }
}

impl ModalContent<Message> for DltSettingsView {
    fn build_content<'a>(&self, dark_mode: bool) -> Element<'a, Message> {
        // Build the tree view (left panel)
        let tree_view = self.build_tree_view(dark_mode);
        
        // Build the detail view (right panel)
        let detail_view = self.build_detail_view(dark_mode);

        // Main content area with two panels
        row![
            // Left panel - Tree view
            ModalWindow::panel_container(
                scrollable(tree_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dark_mode,
                Length::FillPortion(2),
                Length::Fixed(400.0),
            ),
            
            Space::new(Length::Fixed(10.0), Length::Shrink),
            
            // Right panel - Detail view
            ModalWindow::panel_container(
                scrollable(detail_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dark_mode,
                Length::FillPortion(3),
                Length::Fixed(400.0),
            ),
        ]
        .width(Length::Fill)
        .into()
    }

    fn close_message(&self) -> Message {
        Message::CloseDltSettings
    }

    fn refresh_message(&self) -> Option<Message> {
        Some(Message::RefreshDltItems)
    }

    fn apply_message(&self) -> Option<Message> {
        Some(Message::ApplyDltSettings)
    }

    fn config(&self) -> ModalConfig {
        ModalConfig {
            width: 900.0,
            height: 600.0,
            title: "DLT Settings".to_string(),
            show_refresh: true,
            show_apply: true,
        }
    }
}