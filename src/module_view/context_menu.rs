use iced::widget::canvas::{self, Canvas};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme, keyboard, mouse};

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub position: Point,
    pub items: Vec<ContextMenuItem>,
    pub target_module: Option<usize>,  // Which module this menu is for
    pub width: f32,
    pub item_height: f32,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextMenuAction,
    pub separator_after: bool,  // Draw a separator line after this item
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextMenuAction {
    AddChart,
    AddGanttChart,
    AddInjectionWindow,
    AddMeterWindow,
    Delete,
    Duplicate,
    Settings,
}

impl ContextMenu {
    pub fn new(position: Point, target_module: Option<usize>) -> Self {
        let items = vec![
            ContextMenuItem {
                label: "Add Chart".to_string(),
                action: ContextMenuAction::AddChart,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Add Gantt Chart".to_string(),
                action: ContextMenuAction::AddGanttChart,
                separator_after: true,  // Separator after this item
            },
            ContextMenuItem {
                label: "Duplicate".to_string(),
                action: ContextMenuAction::Duplicate,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Settings".to_string(),
                action: ContextMenuAction::Settings,
                separator_after: true,
            },
            ContextMenuItem {
                label: "Delete".to_string(),
                action: ContextMenuAction::Delete,
                separator_after: false,
            },
        ];

        Self {
            position,
            items,
            target_module,
            width: 180.0,
            item_height: 32.0,
        }
    }

    pub fn get_action_at(&self, point: Point) -> Option<ContextMenuAction> {
        // Check if point is within menu bounds
        let menu_height = self.calculate_total_height();
        
        if point.x < self.position.x || point.x > self.position.x + self.width {
            return None;
        }
        
        if point.y < self.position.y || point.y > self.position.y + menu_height {
            return None;
        }

        // Calculate which item was clicked
        let mut current_y = self.position.y;
        
        for item in &self.items {
            if point.y >= current_y && point.y < current_y + self.item_height {
                return Some(item.action);
            }
            
            current_y += self.item_height;
            
            // Account for separator height
            if item.separator_after {
                current_y += 8.0;
            }
        }

        None
    }

    fn calculate_total_height(&self) -> f32 {
        let item_count = self.items.len() as f32;
        let separator_count = self.items.iter().filter(|i| i.separator_after).count() as f32;
        
        item_count * self.item_height + separator_count * 8.0
    }
}

pub fn draw_context_menu(
    frame: &mut canvas::Frame,
    menu: &ContextMenu,
    cursor_position: Option<Point>,
    dark_mode: bool,
) {
    let menu_height = menu.calculate_total_height();
    let padding = 4.0;
    let corner_radius = 8.0;

    // Menu background colors
    let bg_color = if dark_mode {
        Color::from_rgba(0.15, 0.15, 0.18, 0.98)
    } else {
        Color::from_rgba(0.98, 0.98, 0.99, 0.98)
    };

    // Draw shadow for depth (multiple layers for softer shadow)
    for i in 0..4 {
        let shadow_offset = (i + 1) as f32 * 0.5;
        let shadow_alpha = 0.08 / (i + 1) as f32;
        
        frame.fill(
            &canvas::Path::rectangle(
                Point::new(
                    menu.position.x + shadow_offset,
                    menu.position.y + shadow_offset,
                ),
                Size::new(menu.width, menu_height),
            ),
            Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
        );
    }

    // Draw main background with rounded corners
    let background_path = canvas::Path::rounded_rectangle(
        Point::new(menu.position.x, menu.position.y),
        Size::new(menu.width, menu_height),
        corner_radius.into(),
    );
    frame.fill(&background_path, bg_color);

    // Draw border
    let border_color = if dark_mode {
        Color::from_rgba(0.3, 0.3, 0.35, 0.5)
    } else {
        Color::from_rgba(0.7, 0.7, 0.75, 0.4)
    };
    
    frame.stroke(
        &background_path,
        canvas::Stroke::default()
            .with_color(border_color)
            .with_width(1.0),
    );

    // Draw menu items
    let mut current_y = menu.position.y;
    let hovered_action = cursor_position.and_then(|pos| menu.get_action_at(pos));

    for item in &menu.items {
        let is_hovered = hovered_action == Some(item.action);

        // Draw hover highlight
        if is_hovered {
            let hover_color = if dark_mode {
                Color::from_rgba(0.3, 0.4, 0.6, 0.4)
            } else {
                Color::from_rgba(0.5, 0.65, 0.85, 0.15)
            };

            let item_rect = canvas::Path::rectangle(
                Point::new(menu.position.x + padding, current_y + 2.0),
                Size::new(menu.width - padding * 2.0, menu.item_height - 4.0),
            );
            frame.fill(&item_rect, hover_color);
        }

        // Special styling for Delete action
        let is_delete = item.action == ContextMenuAction::Delete;
        
        let text_color = if is_delete {
            if dark_mode {
                Color::from_rgb(0.95, 0.4, 0.4)
            } else {
                Color::from_rgb(0.8, 0.2, 0.2)
            }
        } else if dark_mode {
            if is_hovered {
                Color::WHITE
            } else {
                Color::from_rgba(0.9, 0.9, 0.92, 0.95)
            }
        } else {
            if is_hovered {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::from_rgb(0.2, 0.2, 0.25)
            }
        };

        // Draw text label
        frame.fill_text(canvas::Text {
            content: item.label.clone(),
            position: Point::new(
                menu.position.x + padding + 12.0,
                current_y + menu.item_height / 2.0,
            ),
            color: text_color,
            size: 14.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Center,
            font: iced::Font::default(),
            ..canvas::Text::default()
        });

        current_y += menu.item_height;

        // Draw separator if needed
        if item.separator_after {
            let separator_y = current_y + 4.0;
            let separator_color = if dark_mode {
                Color::from_rgba(0.4, 0.4, 0.45, 0.3)
            } else {
                Color::from_rgba(0.6, 0.6, 0.65, 0.25)
            };

            let separator_path = canvas::Path::line(
                Point::new(menu.position.x + padding + 8.0, separator_y),
                Point::new(menu.position.x + menu.width - padding - 8.0, separator_y),
            );

            frame.stroke(
                &separator_path,
                canvas::Stroke::default()
                    .with_color(separator_color)
                    .with_width(1.0),
            );

            current_y += 8.0;
        }
    }
}