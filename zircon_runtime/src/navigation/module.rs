use std::sync::Arc;

use crate::core::manager::{NavigationManagerHandle, NAVIGATION_MANAGER_NAME};
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, EngineModule};
use crate::scene::{SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME};

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
            let runtime = core.resolve_driver::<BuiltinNavigationManager>(
                BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME,
            )?;
            Ok(Arc::new(SceneNavigationRuntimeHandle::new(runtime)) as ServiceObject)
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
            let runtime = core.resolve_driver::<BuiltinNavigationManager>(
                BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME,
            )?;
            Ok(Arc::new(NavigationManagerHandle::new(runtime)) as ServiceObject)
        }),
    ))
}

#[cfg(test)]
mod tests {
    use crate::core::manager::{NavigationManagerHandle, NAVIGATION_MANAGER_NAME};
    use crate::core::runtime::CoreRuntime;
    use crate::core::ServiceKind;
    use crate::scene::{SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME};

    use super::{
        module_descriptor, BuiltinNavigationManager, BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME,
    };

    #[test]
    fn builtin_navigation_module_obeys_driver_manager_dependency_layers() {
        let descriptor = module_descriptor();

        let implementation = descriptor
            .drivers
            .iter()
            .find(|driver| driver.name.as_str() == BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("built-in navigation implementation must be a driver");
        assert!(implementation.dependencies.is_empty());

        let scene_driver = descriptor
            .drivers
            .iter()
            .find(|driver| driver.name.as_str() == SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("scene navigation runtime must be a driver");
        assert_eq!(scene_driver.dependencies.len(), 1);
        assert_eq!(
            scene_driver.dependencies[0].name.as_str(),
            BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME
        );
        assert_eq!(
            scene_driver.dependencies[0].name.service_kind(),
            ServiceKind::Driver
        );

        let public_manager = descriptor
            .managers
            .iter()
            .find(|manager| manager.name.as_str() == NAVIGATION_MANAGER_NAME)
            .expect("public navigation facade must be a manager");
        assert_eq!(public_manager.dependencies.len(), 1);
        assert_eq!(
            public_manager.dependencies[0].name.as_str(),
            BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME
        );

        let runtime = CoreRuntime::new();
        runtime
            .register_module(descriptor)
            .expect("navigation service dependency layering must be valid");
        runtime
            .resolve_driver::<BuiltinNavigationManager>(BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("internal navigation runtime driver must resolve");
        runtime
            .resolve_driver::<SceneNavigationRuntimeHandle>(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
            .expect("scene navigation runtime driver must resolve");
        runtime
            .resolve_manager::<NavigationManagerHandle>(NAVIGATION_MANAGER_NAME)
            .expect("public navigation manager facade must resolve");
    }
}
