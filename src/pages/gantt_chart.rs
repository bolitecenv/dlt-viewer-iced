use crate::message::Message;
use chrono::Datelike;
use chrono::{Duration, NaiveDate};
use iced::{
    Element, Length, Point, Rectangle, Renderer, Size, Theme, mouse,
    widget::{
        Canvas, Column, Space, button, canvas, column, container, row, scrollable, text, text_input,
    },
};
use iced_aw::date_picker::{self, DatePicker};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub color: [f32; 3],
}

impl Task {
    pub fn duration_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }
}

pub struct GanttChartState {
    pub tasks: Vec<Task>,
    pub new_task_name: String,
    pub show_start_picker: bool,
    pub show_end_picker: bool,
    pub selected_start_date: NaiveDate,
    pub selected_end_date: NaiveDate,
    pub next_id: usize,
}

impl Default for GanttChartState {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();

        // Sample tasks
        let tasks = vec![
            Task {
                id: 0,
                name: "Project Planning".to_string(),
                start_date: today,
                end_date: today + Duration::days(5),
                color: [0.3, 0.6, 0.9],
            },
            Task {
                id: 1,
                name: "Design Phase".to_string(),
                start_date: today + Duration::days(6),
                end_date: today + Duration::days(15),
                color: [0.9, 0.5, 0.3],
            },
            Task {
                id: 2,
                name: "Development".to_string(),
                start_date: today + Duration::days(16),
                end_date: today + Duration::days(40),
                color: [0.5, 0.8, 0.4],
            },
            Task {
                id: 3,
                name: "Testing".to_string(),
                start_date: today + Duration::days(35),
                end_date: today + Duration::days(45),
                color: [0.9, 0.7, 0.3],
            },
            Task {
                id: 4,
                name: "Deployment".to_string(),
                start_date: today + Duration::days(46),
                end_date: today + Duration::days(50),
                color: [0.8, 0.3, 0.6],
            },
        ];

        Self {
            next_id: tasks.len(),
            tasks,
            new_task_name: String::new(),
            show_start_picker: false,
            show_end_picker: false,
            selected_start_date: today,
            selected_end_date: today + Duration::days(7),
        }
    }
}

pub fn view<'a>(state: &'a GanttChartState, _dark_mode: bool) -> Element<'a, Message> {
    // Control panel
    let mut controls = Column::new().padding(20).spacing(15);

    // Title
    let title = text("Project Timeline").size(24);

    controls = controls.push(title);

    // Add task section
    let add_section_title = text("Add New Task").size(18);
    controls = controls.push(add_section_title);

    let name_input = text_input("Task name...", &state.new_task_name)
        .on_input(Message::GanttTaskNameChanged)
        .padding(10)
        .width(Length::Fill);

    let start_date_text = text(format!("Start: {}", state.selected_start_date)).size(14);
    let start_button = button("Select Start Date")
        .on_press(Message::GanttShowStartPicker)
        .padding(8);

    let end_date_text = text(format!("End: {}", state.selected_end_date)).size(14);
    let end_button = button("Select End Date")
        .on_press(Message::GanttShowEndPicker)
        .padding(8);

    let add_button = button("Add Task")
        .on_press(Message::GanttAddTask)
        .padding(10);

    controls = controls
        .push(name_input)
        .push(row![start_date_text, Space::new(10, 0), start_button].spacing(10))
        .push(row![end_date_text, Space::new(10, 0), end_button].spacing(10))
        .push(add_button);

    // Task list
    let mut task_list = Column::new().spacing(8).push(text("Tasks:").size(18));

    for task in &state.tasks {
        let task_color = task.color;
        let color_indicator =
            container(Space::new(12, 12)).style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    task_color[0],
                    task_color[1],
                    task_color[2],
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(1.0, 1.0, 1.0),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            });

        let task_row = container(
            row![
                color_indicator,
                column![
                    text(&task.name).size(14),
                    text(format!(
                        "{} → {} ({} days)",
                        task.start_date.format("%m/%d"),
                        task.end_date.format("%m/%d"),
                        task.duration_days()
                    ))
                    .size(12)
                ]
                .spacing(2)
                .width(Length::Fill),
                button("Remove")
                    .on_press(Message::GanttRemoveTask(task.id))
                    .padding(6)
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding(10)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: iced::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

        task_list = task_list.push(task_row);
    }

    controls = controls.push(scrollable(task_list).height(Length::Fill));

    let controls_container = container(controls)
        .width(380)
        .height(Length::Fill)
        .padding(10)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: iced::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        });

    // Gantt chart canvas
    let gantt_canvas: Element<Message> = Canvas::new(GanttChartCanvas {
        tasks: &state.tasks,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let chart_container = container(gantt_canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: iced::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        });

    // Layout
    let content = row![controls_container, chart_container]
        .spacing(15)
        .height(Length::Fill);

    let main_content: Element<Message> = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Date pickers overlay
    if state.show_start_picker {
        let date = date_picker::Date {
            year: state.selected_start_date.year() as i32,
            month: state.selected_start_date.month(),
            day: state.selected_start_date.day(),
        };
        DatePicker::new(
            state.show_start_picker,
            date,
            main_content,
            Message::GanttCancelStartPicker,
            Message::GanttStartDateSelected,
        )
        .into()
    } else if state.show_end_picker {
        let date = date_picker::Date {
            year: state.selected_end_date.year() as i32,
            month: state.selected_end_date.month(),
            day: state.selected_end_date.day(),
        };
        DatePicker::new(
            state.show_end_picker,
            date,
            main_content,
            Message::GanttCancelEndPicker,
            Message::GanttEndDateSelected,
        )
        .into()
    } else {
        main_content
    }
}

struct GanttChartCanvas<'a> {
    tasks: &'a [Task],
}

impl<'a> canvas::Program<Message> for GanttChartCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let palette = theme.extended_palette();
        let text_color = palette.background.base.text;
        let grid_color = palette.background.strong.color;

        if self.tasks.is_empty() {
            frame.fill_text(canvas::Text {
                content: "No tasks yet. Add some tasks to see the Gantt chart!".to_string(),
                position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                color: text_color,
                size: 16.0.into(),
                ..canvas::Text::default()
            });
            return vec![frame.into_geometry()];
        }

        // Calculate date range
        let min_date = self
            .tasks
            .iter()
            .map(|t| t.start_date)
            .min()
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        let max_date = self
            .tasks
            .iter()
            .map(|t| t.end_date)
            .max()
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let total_days = (max_date - min_date).num_days() + 1;

        // Layout parameters
        let margin_top = 60.0;
        let margin_left = 200.0;
        let chart_width = (bounds.width - margin_left - 40.0).max(400.0);
        let chart_height = bounds.height - margin_top - 40.0;
        let row_height = (chart_height / self.tasks.len() as f32).min(80.0).max(40.0);
        let day_width = chart_width / total_days as f32;

        // Draw title
        frame.fill_text(canvas::Text {
            content: "Project Gantt Chart".to_string(),
            position: Point::new(bounds.width / 2.0, 25.0),
            color: text_color,
            size: 22.0.into(),
            ..canvas::Text::default()
        });

        // Draw timeline header
        let date_label_interval = if total_days > 60 {
            14
        } else if total_days > 30 {
            7
        } else {
            3
        };

        for i in 0..=total_days {
            let date = min_date + Duration::days(i);
            let x = margin_left + i as f32 * day_width;

            // Draw vertical grid lines
            if i % date_label_interval == 0 {
                frame.stroke(
                    &canvas::Path::line(
                        Point::new(x, margin_top),
                        Point::new(x, margin_top + chart_height),
                    ),
                    canvas::Stroke::default()
                        .with_color(grid_color)
                        .with_width(1.0),
                );

                // Draw date labels
                frame.fill_text(canvas::Text {
                    content: date.format("%m/%d").to_string(),
                    position: Point::new(x, margin_top - 15.0),
                    color: text_color,
                    size: 11.0.into(),
                    ..canvas::Text::default()
                });
            }
        }

        // Draw tasks
        for (idx, task) in self.tasks.iter().enumerate() {
            let y = margin_top + idx as f32 * row_height;

            // Draw task name
            frame.fill_text(canvas::Text {
                content: task.name.clone(),
                position: Point::new(10.0, y + row_height / 2.0),
                color: text_color,
                size: 13.0.into(),
                ..canvas::Text::default()
            });

            // Calculate bar position
            let start_offset = (task.start_date - min_date).num_days();
            let bar_x = margin_left + start_offset as f32 * day_width;
            let bar_width = (task.duration_days() as f32 * day_width).max(2.0);
            let bar_y = y + row_height * 0.25;
            let bar_height = row_height * 0.5;

            // Draw task bar shadow
            frame.fill_rectangle(
                Point::new(bar_x + 2.0, bar_y + 2.0),
                Size::new(bar_width, bar_height),
                iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            );

            // Draw task bar
            frame.fill_rectangle(
                Point::new(bar_x, bar_y),
                Size::new(bar_width, bar_height),
                iced::Color::from_rgb(task.color[0], task.color[1], task.color[2]),
            );

            // Draw task bar border - FIXED: use correct stroke_rectangle signature
            frame.stroke_rectangle(
                Point::new(bar_x, bar_y),
                Size::new(bar_width, bar_height),
                canvas::Stroke::default()
                    .with_color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                    .with_width(1.5),
            );

            // Draw duration text on bar
            if bar_width > 50.0 {
                frame.fill_text(canvas::Text {
                    content: format!("{} days", task.duration_days()),
                    position: Point::new(bar_x + bar_width / 2.0, bar_y + bar_height / 2.0),
                    color: iced::Color::WHITE,
                    size: 11.0.into(),
                    ..canvas::Text::default()
                });
            }

            // Draw horizontal grid line
            frame.stroke(
                &canvas::Path::line(
                    Point::new(margin_left, y + row_height),
                    Point::new(margin_left + chart_width, y + row_height),
                ),
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(0.5),
            );
        }

        vec![frame.into_geometry()]
    }
}
