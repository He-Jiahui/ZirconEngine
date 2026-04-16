use std::fmt;

use crate::lifecycle::StartupMode;

use super::{DependencySpec, RegistryName, ServiceFactory};

#[derive(Clone)]
pub struct PluginDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Vec<DependencySpec>,
    pub factory: ServiceFactory,
}

impl PluginDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies,
            factory,
        }
    }
}

impl fmt::Debug for PluginDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}
