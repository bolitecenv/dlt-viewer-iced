// src/plugin_registry.rs
use std::collections::HashMap;
use iced::Task;

use crate::plugin::{DashboardContext, Plugin, PluginMessage};
use crate::plugins::{example};

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
        };
        
        // Auto-register plugins
        registry.register_all();
        registry
    }
    
    #[allow(unused_mut)]
    fn register_all(&mut self) {
        self.register::<example::ExamplePlugin>();
    }
    
    pub fn register<P: Plugin + 'static>(&mut self) {
        let plugin = P::new();
        let name = plugin.name().to_string();
        self.plugins.insert(name, Box::new(plugin));
    }

    pub fn update(&mut self,
        name: &str,
        message: PluginMessage,
        context: &DashboardContext,
    ) -> Task<PluginMessage> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.update(message, context)
        } else {
            Task::none()
        }
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }
    
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Plugin>> {
        self.plugins.get_mut(name)
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }

    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}