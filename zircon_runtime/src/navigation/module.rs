use std::sync::Arc;

use crate::core::framework::navigation::NavigationManager;
use crate::core::manager::NAVIGATION_MANAGER_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind,
    StartupMode,
};
use crate::engine_module::{dependency_on, factory, EngineModule};
use crate::scene::{SceneNavigationRuntime, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME};

use super::BuiltinNavigationManager;

pub const BUILTIN_NAVIGATION_MODULE_NAME: &str = "navigation.runtime";
pub const BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME: &str =
    "navigation.runtime.Driver.BuiltinNavigationRuntime";

#[derive(Debug, Default)]
pub struct BuiltinNavigationModule;

impl EngineModule for BuiltinNavigationModule {
    fn module_name(&self) -> &'static str {
        BUILTIN_NAVIGATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Built-in baked navmesh pathfinding and lightweight agent avoidance"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        BUILTIN_NAVIGATION_MODULE_NAME,
        "Built-in baked navmesh pathfinding and lightweight agent avoidance",
    )
    .with_driver(DriverDescriptor::new(
        RegistryName::new(BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("built-in navigation implementation driver name must be valid"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(Arc::new(BuiltinNavigationManager::new()) as ServiceObject)),
    ))
    .with_driver(DriverDescriptor::new(
        RegistryName::new(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("scene navigation runtime driver name must be valid"),
        StartupMode::Lazy,
        vec![dependency_on(
            BUILTIN_NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "BuiltinNavigationRuntime",
        )],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            core.register_service_alias::<BuiltinNavigationManager, dyn SceneNavigationRuntime>(
                BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                |runtime| runtime,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        RegistryName::new(NAVIGATION_MANAGER_NAME)
            .expect("built-in navigation manager name must be valid"),
        StartupMode::Lazy,
        vec![dependency_on(
            BUILTIN_NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "BuiltinNavigationRuntime",
        )],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            core.register_service_alias::<BuiltinNavigationManager, dyn NavigationManager>(
                BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                NAVIGATION_MANAGER_NAME,
                ServiceKind::Manager,
                |runtime| runtime,
            )
        }),
    ))
}

#[cfg(test)]
#[path = "tests/module.rs"]
mod tests;
