use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

#[test]
fn deactivate_exact_two_services_reports_first_shutdown_service_when_dependent_names_both() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts(
        "ExactTwoDependencyMatcherModule",
        ServiceKind::Driver,
        "Driver",
    );
    let manager_name = RegistryName::from_parts(
        "ExactTwoDependencyMatcherModule",
        ServiceKind::Manager,
        "Manager",
    );
    let dependent_manager_name = RegistryName::from_parts(
        "ExactTwoDependencyMatcherDependentModule",
        ServiceKind::Manager,
        "DependentManager",
    );
    let driver_name_for_factory = driver_name.clone();
    let manager_name_for_factory = manager_name.clone();

    runtime
        .register_module(
            ModuleDescriptor::new("ExactTwoDependencyMatcherModule", "exact-two matcher")
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
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new(
                "ExactTwoDependencyMatcherDependentModule",
                "exact-two dependent",
            )
            .with_manager(ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(driver_name.clone()),
                    DependencySpec::named(manager_name.clone()),
                ],
                Arc::new(move |core| {
                    let _ = core.resolve_driver::<TestDriver>(driver_name_for_factory.as_str())?;
                    let _ =
                        core.resolve_manager::<TestManager>(manager_name_for_factory.as_str())?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            )),
        )
        .unwrap();

    runtime
        .activate_module("ExactTwoDependencyMatcherModule")
        .unwrap();
    runtime
        .activate_module("ExactTwoDependencyMatcherDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactTwoDependencyMatcherModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-two unload to report the first shutdown service");
    };
    assert_eq!(blocked_service, manager_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);
}

#[test]
fn deactivate_exact_three_services_reports_first_shutdown_service_when_dependent_names_all() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts(
        "ExactThreeDependencyMatcherModule",
        ServiceKind::Driver,
        "Driver",
    );
    let manager_name = RegistryName::from_parts(
        "ExactThreeDependencyMatcherModule",
        ServiceKind::Manager,
        "Manager",
    );
    let plugin_name = RegistryName::from_parts(
        "ExactThreeDependencyMatcherModule",
        ServiceKind::Plugin,
        "Plugin",
    );
    let dependent_manager_name = RegistryName::from_parts(
        "ExactThreeDependencyMatcherDependentModule",
        ServiceKind::Manager,
        "DependentManager",
    );
    let driver_name_for_factory = driver_name.clone();
    let manager_name_for_factory = manager_name.clone();
    let plugin_name_for_factory = plugin_name.clone();

    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeDependencyMatcherModule", "exact-three matcher")
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
        .register_module(
            ModuleDescriptor::new(
                "ExactThreeDependencyMatcherDependentModule",
                "exact-three dependent",
            )
            .with_manager(ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(driver_name.clone()),
                    DependencySpec::named(manager_name.clone()),
                    DependencySpec::named(plugin_name.clone()),
                ],
                Arc::new(move |core| {
                    let _ = core.resolve_driver::<TestDriver>(driver_name_for_factory.as_str())?;
                    let _ =
                        core.resolve_manager::<TestManager>(manager_name_for_factory.as_str())?;
                    let _ = core.resolve_plugin::<TestManager>(plugin_name_for_factory.as_str())?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            )),
        )
        .unwrap();

    runtime
        .activate_module("ExactThreeDependencyMatcherModule")
        .unwrap();
    runtime
        .activate_module("ExactThreeDependencyMatcherDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactThreeDependencyMatcherModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-three unload to report the first shutdown service");
    };
    assert_eq!(blocked_service, plugin_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);
}

#[test]
fn deactivate_exact_three_services_reports_first_blocked_without_index_map() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactThreeUnloadModule", ServiceKind::Driver, "Driver");
    let manager_name =
        RegistryName::from_parts("ExactThreeUnloadModule", ServiceKind::Manager, "Manager");
    let plugin_name =
        RegistryName::from_parts("ExactThreeUnloadModule", ServiceKind::Plugin, "Plugin");
    let manager_dependent_name = RegistryName::from_parts(
        "ExactThreeDependentModule",
        ServiceKind::Manager,
        "ManagerDependent",
    );
    let driver_dependent_name = RegistryName::from_parts(
        "ExactThreeDependentModule",
        ServiceKind::Manager,
        "DriverDependent",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeUnloadModule", "exact-three blocked unload")
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
                    plugin_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeDependentModule", "dependents")
                .with_manager(ManagerDescriptor::new(
                    manager_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(manager_name.clone())],
                    Arc::new(|core| {
                        let _ = core.resolve_manager::<TestManager>(
                            "ExactThreeUnloadModule.Manager.Manager",
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
                            .resolve_driver::<TestDriver>("ExactThreeUnloadModule.Driver.Driver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactThreeUnloadModule").unwrap();
    runtime
        .activate_module("ExactThreeDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactThreeUnloadModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-three unload to report the earliest running external dependent");
    };
    assert_eq!(blocked_service, manager_name.as_str());
    assert_eq!(dependents, vec![manager_dependent_name.to_string()]);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactThreeUnloadModule")
        .expect("blocked exact-three deactivate should keep the module registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &manager_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("blocked exact-three deactivate should keep owner service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}
