//! Navigation module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";

#[derive(Clone, Debug, Default)]
pub struct NavigationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavigationModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        NAVIGATION_MODULE_NAME,
        "Navigation, pathfinding, and avoidance",
        "NavigationDriver",
        "NavigationManager",
    )
}

impl EngineModule for NavigationModule {
    fn module_name(&self) -> &'static str {
        NAVIGATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Navigation, pathfinding, and avoidance"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
