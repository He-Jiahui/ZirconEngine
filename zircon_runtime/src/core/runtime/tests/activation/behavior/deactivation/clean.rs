use std::sync::Arc;

use super::super::super::super::super::*;
use super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn deactivate_empty_module_skips_service_unload_precheck() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new(
            "EmptyShutdownModule",
            "empty shutdown",
        ))
        .unwrap();

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("EmptyShutdownModule")
            .expect("registered empty module should be in module table");
        assert!(module.shutdown_service_names.is_empty());
        assert_eq!(module.lifecycle, LifecycleState::Registered);
    }

    runtime.activate_module("EmptyShutdownModule").unwrap();
    runtime.deactivate_module("EmptyShutdownModule").unwrap();

    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("EmptyShutdownModule")
        .expect("deactivated empty module should remain in module table");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
}

#[test]
fn deactivate_exact_three_services_unloads_all_cached_entries_directly() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactThreeCleanUnloadModule", ServiceKind::Driver, "Driver");
    let manager_name = RegistryName::from_parts(
        "ExactThreeCleanUnloadModule",
        ServiceKind::Manager,
        "Manager",
    );
    let plugin_name =
        RegistryName::from_parts("ExactThreeCleanUnloadModule", ServiceKind::Plugin, "Plugin");

    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeCleanUnloadModule", "exact-three clean unload")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    runtime
        .activate_module("ExactThreeCleanUnloadModule")
        .unwrap();
    runtime
        .deactivate_module("ExactThreeCleanUnloadModule")
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactThreeCleanUnloadModule")
        .expect("deactivated exact-three module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &manager_name, &plugin_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("clean exact-three deactivate should keep cached service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
    }
}

#[test]
fn deactivate_exact_four_services_unloads_all_cached_entries_directly() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFourCleanUnloadModule", ServiceKind::Driver, "Driver");
    let first_manager_name = RegistryName::from_parts(
        "ExactFourCleanUnloadModule",
        ServiceKind::Manager,
        "Manager",
    );
    let second_manager_name = RegistryName::from_parts(
        "ExactFourCleanUnloadModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let plugin_name =
        RegistryName::from_parts("ExactFourCleanUnloadModule", ServiceKind::Plugin, "Plugin");

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourCleanUnloadModule", "exact-four clean unload")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    runtime
        .activate_module("ExactFourCleanUnloadModule")
        .unwrap();
    runtime
        .deactivate_module("ExactFourCleanUnloadModule")
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFourCleanUnloadModule")
        .expect("deactivated exact-four module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [
        &driver_name,
        &first_manager_name,
        &second_manager_name,
        &plugin_name,
    ] {
        let entry = services
            .get(service_name.as_str())
            .expect("clean exact-four deactivate should keep cached service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
    }
}

#[test]
fn deactivate_exact_five_services_unloads_all_cached_entries_directly() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFiveCleanUnloadModule", ServiceKind::Driver, "Driver");
    let first_manager_name = RegistryName::from_parts(
        "ExactFiveCleanUnloadModule",
        ServiceKind::Manager,
        "Manager",
    );
    let second_manager_name = RegistryName::from_parts(
        "ExactFiveCleanUnloadModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let first_plugin_name =
        RegistryName::from_parts("ExactFiveCleanUnloadModule", ServiceKind::Plugin, "Plugin");
    let second_plugin_name = RegistryName::from_parts(
        "ExactFiveCleanUnloadModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveCleanUnloadModule", "exact-five clean unload")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    first_plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    second_plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    runtime
        .activate_module("ExactFiveCleanUnloadModule")
        .unwrap();
    runtime
        .deactivate_module("ExactFiveCleanUnloadModule")
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFiveCleanUnloadModule")
        .expect("deactivated exact-five module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [
        &driver_name,
        &first_manager_name,
        &second_manager_name,
        &first_plugin_name,
        &second_plugin_name,
    ] {
        let entry = services
            .get(service_name.as_str())
            .expect("clean exact-five deactivate should keep cached service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
    }
}
