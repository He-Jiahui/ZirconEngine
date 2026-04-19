use super::{DriverDescriptor, ManagerDescriptor, PluginDescriptor};

#[derive(Clone, Debug)]
pub struct ModuleDescriptor {
    pub name: String,
    pub description: String,
    pub drivers: Vec<DriverDescriptor>,
    pub managers: Vec<ManagerDescriptor>,
    pub plugins: Vec<PluginDescriptor>,
}

impl ModuleDescriptor {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            drivers: Vec::new(),
            managers: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn with_driver(mut self, descriptor: DriverDescriptor) -> Self {
        self.drivers.push(descriptor);
        self
    }

    pub fn with_manager(mut self, descriptor: ManagerDescriptor) -> Self {
        self.managers.push(descriptor);
        self
    }

    pub fn with_plugin(mut self, descriptor: PluginDescriptor) -> Self {
        self.plugins.push(descriptor);
        self
    }
}
