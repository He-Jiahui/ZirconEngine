//! Navigation module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";
pub const NAVIGATION_DRIVER_NAME: &str = "NavigationModule.Driver.NavigationDriver";
pub const NAVIGATION_MANAGER_NAME: &str = "NavigationModule.Manager.NavigationManager";

#[derive(Clone, Debug, Default)]
pub struct NavigationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavigationModule;

#[derive(Clone, Debug, Default)]
pub struct NavigationDriver;

#[derive(Clone, Debug, Default)]
pub struct NavigationManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NAVIGATION_MODULE_NAME,
        "Navigation, pathfinding, and avoidance",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "NavigationDriver",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(NavigationDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Manager,
            "NavigationManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "NavigationDriver",
        )],
        factory(|_| Ok(Arc::new(NavigationManager::default()) as _)),
    ))
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
