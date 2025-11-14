// mod.rs - Chart module organization

pub mod module_widget;
pub mod chart_renderer;
pub mod canvas;
pub mod ganttchart_renderer;

// Re-export commonly used items
pub use module_widget::{ModuleWidget, RESIZE_HANDLE_SIZE, MIN_CHART_WIDTH, MIN_CHART_HEIGHT};
pub use chart_renderer::ChartRenderer;
pub use canvas::{ModuleCanvas, DragState, view};