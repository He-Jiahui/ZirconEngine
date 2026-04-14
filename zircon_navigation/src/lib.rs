//! Navigation module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";

#[derive(Clone, Debug, Default)]
pub struct NavigationConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        NAVIGATION_MODULE_NAME,
        "Navigation, pathfinding, and avoidance",
        "NavigationDriver",
        "NavigationManager",
    )
}
