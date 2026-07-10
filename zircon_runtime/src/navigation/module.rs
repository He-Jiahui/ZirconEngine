use std::sync::Arc;

use crate::core::manager::{NavigationManagerHandle, NAVIGATION_MANAGER_NAME};
use crate::core::runtime::ServiceObject;
use crate::core::{ManagerDescriptor, ModuleDescriptor, RegistryName, StartupMode};
use crate::engine_module::{factory, EngineModule};

use super::BuiltinNavigationManager;

pub const BUILTIN_NAVIGATION_MODULE_NAME: &str = "navigation.runtime";

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
    .with_manager(ManagerDescriptor::new(
        RegistryName::new(NAVIGATION_MANAGER_NAME)
            .expect("built-in navigation manager name must be valid"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| {
            Ok(Arc::new(NavigationManagerHandle::new(Arc::new(
                BuiltinNavigationManager::new(),
            ))) as ServiceObject)
        }),
    ))
}
