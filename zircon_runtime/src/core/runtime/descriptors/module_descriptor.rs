use std::fmt;
use std::sync::Arc;

use super::{DriverDescriptor, ManagerDescriptor, ModuleDependencySpec, PluginDescriptor};
use crate::core::runtime::lifecycle::{InitLevel, ModuleLifecycle, NoopModuleLifecycle};

#[derive(Clone)]
pub struct ModuleDescriptor {
    pub name: String,
    pub description: String,
    pub init_level: InitLevel,
    pub module_dependencies: Vec<ModuleDependencySpec>,
    pub lifecycle: Arc<dyn ModuleLifecycle>,
    pub drivers: Vec<DriverDescriptor>,
    pub managers: Vec<ManagerDescriptor>,
    pub plugins: Vec<PluginDescriptor>,
}

impl ModuleDescriptor {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            lifecycle: Arc::new(NoopModuleLifecycle),
            drivers: Vec::new(),
            managers: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.init_level = init_level;
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module_dependencies.push(dependency);
        self
    }

    pub fn with_lifecycle(mut self, lifecycle: Arc<dyn ModuleLifecycle>) -> Self {
        self.lifecycle = lifecycle;
        self
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

impl fmt::Debug for ModuleDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleDescriptor")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("init_level", &self.init_level)
            .field("module_dependencies", &self.module_dependencies)
            .field("lifecycle", &"ModuleLifecycle")
            .field("drivers", &self.drivers)
            .field("managers", &self.managers)
            .field("plugins", &self.plugins)
            .finish()
    }
}
