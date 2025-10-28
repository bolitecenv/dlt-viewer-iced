use crate::chart::line_chart;
use crate::components::{navigation, top_bar};
use crate::message::{Message, Page};
use crate::pages;
use crate::pages::gantt_chart::GanttChartState;
use iced::futures::{self, StreamExt};
use iced::{
    Element, Length, Subscription, Task, Theme,
    widget::{column, container, row},
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tokio::time::sleep;

pub struct Dashboard {
    pub metric1: i32,
    pub metric2: i32,
    pub total_users: u32,
    pub active_sessions: u32,
    pub current_page: Page,
    pub dark_mode: bool,
    pub chart_data: Vec<f32>,
    pub tcp_ip: String,
    pub tcp_port: String,
    pub connection_status: String,
    pub gantt_chart_state: GanttChartState,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            metric1: 42,
            metric2: 78,
            total_users: 1247,
            active_sessions: 89,
            current_page: Page::Overview,
            dark_mode: true,
            chart_data: line_chart::generate_random_data(),
            tcp_ip: "127.0.0.1".to_string(),
            tcp_port: "8080".to_string(),
            connection_status: "Disconnected".to_string(),
            gantt_chart_state: GanttChartState::default(),
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IncrementMetric1 => self.metric1 += 1,
            Message::DecrementMetric1 => self.metric1 -= 1,
            Message::IncrementMetric2 => self.metric2 += 5,
            Message::RefreshData => {
                self.total_users += 1;
                self.active_sessions = (self.active_sessions + 3) % 150;
            }
            Message::ToggleTheme => self.dark_mode = !self.dark_mode,
            Message::NavigateTo(page) => self.current_page = page,
            Message::Tick => {
                // Update chart data every tick
                self.chart_data = line_chart::generate_random_data();
            }
            Message::TcpIpChanged(ip) => self.tcp_ip = ip,
            Message::TcpPortChanged(port) => self.tcp_port = port,
            Message::ConnectTcp => {
                let ip = self.tcp_ip.clone();
                let port = self.tcp_port.clone();
                return Task::perform(
                    async move {
                        match TcpStream::connect(format!("{}:{}", ip, port)) {
                            Ok(mut stream) => {
                                let mut buffer = [0; 512];
                                match stream.read(&mut buffer) {
                                    Ok(_) => Ok("Connected successfully".to_string()),
                                    Err(e) => Err(format!("Read error: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Connection error: {}", e)),
                        }
                    },
                    Message::TcpConnectionResult,
                );
            }
            Message::TcpConnectionResult(result) => {
                self.connection_status = match result {
                    Ok(msg) => msg,
                    Err(err) => err,
                };
            }
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            futures::stream::unfold((), |_| async {
                sleep(Duration::from_millis(100)).await;
                Some((Message::Tick, ()))
            })
        })
    }

    pub fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn view(&self) -> Element<Message> {
        let top = top_bar::view(self.dark_mode);
        let nav = navigation::view(self.current_page, self.dark_mode);

        let main_content = match self.current_page {
            Page::Overview => pages::overview::view(self),
            Page::Analytics => pages::analytics::view(self.dark_mode, &self.chart_data),
            Page::Reports => pages::placeholder::view("Reports", "📋", self.dark_mode),
            Page::Settings => pages::settings::view(
                self.dark_mode,
                &self.tcp_ip,
                &self.tcp_port,
                &self.connection_status,
            ), // Updated to use the new settings page
            Page::GanttChart => pages::gantt_chart::view(&self.gantt_chart_state, self.dark_mode),
        };

        let content_area = container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        let main_layout = column![top, row![nav, content_area].height(Length::Fill)];

        container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
