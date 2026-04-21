use crate::engine_module::{EngineModule, ModuleDescriptor};

use super::{NavigationDriver, NavigationManager};

pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";
pub const NAVIGATION_DRIVER_NAME: &str = "NavigationModule.Driver.NavigationDriver";
pub const NAVIGATION_MANAGER_NAME: &str = "NavigationModule.Manager.NavigationManager";

#[derive(Clone, Copy, Debug, Default)]
pub struct NavigationModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::super::module_descriptor_with_driver_and_manager::<NavigationDriver, NavigationManager>(
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
