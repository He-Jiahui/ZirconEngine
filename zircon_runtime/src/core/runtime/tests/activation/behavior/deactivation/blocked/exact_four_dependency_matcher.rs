use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

#[test]
fn deactivate_exact_four_services_reports_first_shutdown_service_when_dependent_names_all() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts(
        "ExactFourDependencyMatcherModule",
        ServiceKind::Driver,
        "Driver",
    );
    let first_manager_name = RegistryName::from_parts(
        "ExactFourDependencyMatcherModule",
        ServiceKind::Manager,
        "Manager",
    );
    let second_manager_name = RegistryName::from_parts(
        "ExactFourDependencyMatcherModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let plugin_name = RegistryName::from_parts(
        "ExactFourDependencyMatcherModule",
        ServiceKind::Plugin,
        "Plugin",
    );
    let dependent_manager_name = RegistryName::from_parts(
        "ExactFourDependencyMatcherDependentModule",
        ServiceKind::Manager,
        "DependentManager",
    );
    let driver_name_for_factory = driver_name.clone();
    let first_manager_name_for_factory = first_manager_name.clone();
    let second_manager_name_for_factory = second_manager_name.clone();
    let plugin_name_for_factory = plugin_name.clone();

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourDependencyMatcherModule", "exact-four matcher")
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
        .register_module(
            ModuleDescriptor::new(
                "ExactFourDependencyMatcherDependentModule",
                "exact-four dependent",
            )
            .with_manager(ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(driver_name.clone()),
                    DependencySpec::named(first_manager_name.clone()),
                    DependencySpec::named(second_manager_name.clone()),
                    DependencySpec::named(plugin_name.clone()),
                ],
                Arc::new(move |core| {
                    let _ = core.resolve_driver::<TestDriver>(driver_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_manager::<TestManager>(first_manager_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_manager::<TestManager>(second_manager_name_for_factory.as_str())?;
                    let _ = core.resolve_plugin::<TestManager>(plugin_name_for_factory.as_str())?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            )),
        )
        .unwrap();

    runtime
        .activate_module("ExactFourDependencyMatcherModule")
        .unwrap();
    runtime
        .activate_module("ExactFourDependencyMatcherDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactFourDependencyMatcherModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-four unload to report the first shutdown service");
    };
    assert_eq!(blocked_service, plugin_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);
}

#[test]
fn deactivate_exact_four_services_reports_first_blocked_without_index_map() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFourUnloadModule", ServiceKind::Driver, "Driver");
    let first_manager_name =
        RegistryName::from_parts("ExactFourUnloadModule", ServiceKind::Manager, "Manager");
    let second_manager_name = RegistryName::from_parts(
        "ExactFourUnloadModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let plugin_name =
        RegistryName::from_parts("ExactFourUnloadModule", ServiceKind::Plugin, "Plugin");
    let second_manager_dependent_name = RegistryName::from_parts(
        "ExactFourDependentModule",
        ServiceKind::Manager,
        "AssetManagerDependent",
    );
    let driver_dependent_name = RegistryName::from_parts(
        "ExactFourDependentModule",
        ServiceKind::Manager,
        "DriverDependent",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourUnloadModule", "exact-four blocked unload")
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
                    plugin_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourDependentModule", "dependents")
                .with_manager(ManagerDescriptor::new(
                    second_manager_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(second_manager_name.clone())],
                    Arc::new(|core| {
                        let _ = core.resolve_manager::<TestManager>(
                            "ExactFourUnloadModule.Manager.AssetManager",
                        )?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    driver_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name.clone())],
                    Arc::new(|core| {
                        let _ = core
                            .resolve_driver::<TestDriver>("ExactFourUnloadModule.Driver.Driver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactFourUnloadModule").unwrap();
    runtime.activate_module("ExactFourDependentModule").unwrap();

    let error = runtime
        .deactivate_module("ExactFourUnloadModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-four unload to report the earliest running external dependent");
    };
    assert_eq!(blocked_service, second_manager_name.as_str());
    assert_eq!(dependents, vec![second_manager_dependent_name.to_string()]);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFourUnloadModule")
        .expect("blocked exact-four deactivate should keep the module registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &first_manager_name, &second_manager_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("blocked exact-four deactivate should keep owner service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}
