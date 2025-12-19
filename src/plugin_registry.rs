use std::collections::HashMap;
use iced::Task;

use crate::plugin::{DashboardContext, Plugin, PluginMessage};

// Auto Generated: use
use crate::plugins::example;
use crate::plugins::timer;

// End Auto Generated: use


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

// Auto Generated: register_all
    #[allow(unused_mut)]
    fn register_all(&mut self) {
        self.register::<example::ExamplePlugin>();
        self.register::<timer::TimerPlugin>();
    }
// End Auto Generated: register_all
    
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

    pub fn update_all(&mut self,
        message: PluginMessage,
        context: &DashboardContext,
    ) -> Vec<Task<PluginMessage>> {
        self.plugins.iter_mut().map(|(_name, plugin)| {
            plugin.update(message.clone(), context)
        }).collect()
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