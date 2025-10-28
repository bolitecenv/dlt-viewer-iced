use crate::message::Message;
use iced::widget::canvas::{self, Canvas, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use rand::Rng;

pub fn view(data: Vec<f32>) -> Element<'static, Message> {
    Canvas::new(LineChart { data })
        .width(Length::Fill)
        .height(Length::Fixed(300.0))
        .into()
}

struct LineChart {
    data: Vec<f32>,
}

impl canvas::Program<Message> for LineChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let padding = 40.0;
        let chart_width = bounds.width - 2.0 * padding;
        let chart_height = bounds.height - 2.0 * padding;

        // Draw axes
        let axis_color = Color::from_rgb(0.7, 0.7, 0.7);
        frame.stroke(
            &Path::line(
                Point::new(padding, padding),
                Point::new(padding, bounds.height - padding),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );
        frame.stroke(
            &Path::line(
                Point::new(padding, bounds.height - padding),
                Point::new(bounds.width - padding, bounds.height - padding),
            ),
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );

        if self.data.is_empty() {
            return vec![frame.into_geometry()];
        }

        let max_value = self.data.iter().cloned().fold(0.0f32, f32::max);
        let min_value = self.data.iter().cloned().fold(f32::MAX, f32::min);
        let value_range = max_value - min_value;

        // Build the line path
        let mut path_builder = canvas::path::Builder::new();

        for (i, &value) in self.data.iter().enumerate() {
            let x = padding + (i as f32 / (self.data.len() - 1) as f32) * chart_width;
            let normalized_value = if value_range > 0.0 {
                (value - min_value) / value_range
            } else {
                0.5
            };
            let y = bounds.height - padding - normalized_value * chart_height;

            if i == 0 {
                path_builder.move_to(Point::new(x, y));
            } else {
                path_builder.line_to(Point::new(x, y));
            }
        }

        let line_path = path_builder.build();
        frame.stroke(
            &line_path,
            canvas::Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.6, 0.9))
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }
}

// Helper function to generate 5000 random points
pub fn generate_random_data() -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..5000).map(|_| rng.gen_range(0.0..100.0)).collect()
}
