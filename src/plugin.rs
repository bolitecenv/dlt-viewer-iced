// src/plugin.rs
use iced::{Element, Task};

use crate::{pages::table::DltMessageRow, types::FrontDltEcuItem};

#[derive(Debug, Clone)]
pub struct DashboardContext<'a> {
    pub ecu_list: &'a Vec<FrontDltEcuItem>,
    pub dlt_buffer: &'a Vec<DltMessageRow>,
}

pub trait Plugin: Send + Sync {
    /// Plugin identifier/name shown in navigation
    fn name(&self) -> &str;

    /// Initialize the plugin
    fn new() -> Self where Self: Sized;
    
    /// Handle plugin-specific messages
    fn update(&mut self, message: PluginMessage, context: &DashboardContext) -> Task<PluginMessage>;

    /// Render plugin view
    fn view(&self, context: &DashboardContext) -> Element<'_, PluginMessage>;
}

#[derive(Debug, Clone)]
pub enum PluginMessage {
    Tick(u64),
    Custom(String, Vec<u8>),
}