use crate::core::manager::{ManagerResolver, NAVIGATION_MANAGER_NAME};
use crate::core::runtime::CoreRuntime;
use crate::core::ServiceKind;
use crate::scene::{SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME};

use super::{module_descriptor, BuiltinNavigationManager, BUILTIN_NAVIGATION_RUNTIME_DRIVER_NAME};

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
    let resolver = ManagerResolver::new(runtime.handle());
    resolver
        .resolve(
            resolver
                .navigation_handle()
                .expect("public navigation manager handle"),
        )
        .expect("public navigation manager facade must resolve");
}
