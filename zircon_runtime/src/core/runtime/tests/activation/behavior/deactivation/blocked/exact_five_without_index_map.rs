use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

#[test]
fn deactivate_exact_five_services_reports_first_blocked_without_index_map() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFiveUnloadModule", ServiceKind::Driver, "Driver");
    let first_manager_name =
        RegistryName::from_parts("ExactFiveUnloadModule", ServiceKind::Manager, "Manager");
    let second_manager_name = RegistryName::from_parts(
        "ExactFiveUnloadModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let first_plugin_name =
        RegistryName::from_parts("ExactFiveUnloadModule", ServiceKind::Plugin, "Plugin");
    let second_plugin_name =
        RegistryName::from_parts("ExactFiveUnloadModule", ServiceKind::Plugin, "RenderPlugin");
    let second_manager_dependent_name = RegistryName::from_parts(
        "ExactFiveDependentModule",
        ServiceKind::Manager,
        "AssetManagerDependent",
    );
    let driver_dependent_name = RegistryName::from_parts(
        "ExactFiveDriverDependentModule",
        ServiceKind::Manager,
        "DriverDependent",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveUnloadModule", "exact-five blocked unload")
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
        .register_module(
            ModuleDescriptor::new("ExactFiveDependentModule", "dependents").with_manager(
                ManagerDescriptor::new(
                    second_manager_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(second_manager_name.clone())],
                    Arc::new(|core| {
                        let _ = core.resolve_manager::<TestManager>(
                            "ExactFiveUnloadModule.Manager.AssetManager",
                        )?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveDriverDependentModule", "driver dependent")
                .with_manager(ManagerDescriptor::new(
                    driver_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name.clone())],
                    Arc::new(|core| {
                        let _ = core
                            .resolve_driver::<TestDriver>("ExactFiveUnloadModule.Driver.Driver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactFiveUnloadModule").unwrap();
    runtime.activate_module("ExactFiveDependentModule").unwrap();
    runtime
        .activate_module("ExactFiveDriverDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactFiveUnloadModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-five unload to report the earliest running external dependent");
    };
    assert_eq!(blocked_service, second_manager_name.as_str());
    assert_eq!(dependents, vec![second_manager_dependent_name.to_string()]);

    runtime
        .deactivate_module("ExactFiveDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactFiveUnloadModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!(
            "expected exact-five unload to report the later driver dependent after manager clears"
        );
    };
    assert_eq!(blocked_service, driver_name.as_str());
    assert_eq!(dependents, vec![driver_dependent_name.to_string()]);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFiveUnloadModule")
        .expect("blocked exact-five deactivate should keep the module registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
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
            .expect("blocked exact-five deactivate should keep owner service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}
