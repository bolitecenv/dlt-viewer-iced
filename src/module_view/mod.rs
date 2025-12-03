// mod.rs - Chart module organization

pub mod canvas;
pub mod chart_widget;
pub mod module_widget;
// pub mod ganttchart_renderer;

// Re-export commonly used items
pub use canvas::{ModuleCanvas};
pub use chart_widget::ChartWidget;
pub use module_widget::{MIN_CHART_HEIGHT, MIN_CHART_WIDTH, ModuleWidget, RESIZE_HANDLE_SIZE};
