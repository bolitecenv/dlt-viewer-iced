// src/plugin.rs
use iced::{Element, Task};

use crate::{pages::table::DltMessageRow, types::FrontDltEcuItem};

#[derive(Debug, Clone)]
pub struct DashboardContext {
    pub ecu_list: Vec<FrontDltEcuItem>,
    pub dlt_buffer: Vec<DltMessageRow>,
}

pub trait Plugin: Send + Sync {
    /// Plugin identifier/name shown in navigation
    fn name(&self) -> &str;

    fn nav_name(&self) -> &str;
    
    /// Initialize the plugin
    fn new() -> Self where Self: Sized;
    
    /// Handle plugin-specific messages
    fn update(&mut self, message: PluginMessage, context: &DashboardContext) -> Task<PluginMessage>;

    /// Render plugin view
    fn view(&self, context: &DashboardContext) -> Element<PluginMessage>;
    
    /// Optional: plugin icon or description
    fn description(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum PluginMessage {
    // Each plugin defines its own message variants
    Custom(String, Vec<u8>), // (plugin_name, serialized_data)
}