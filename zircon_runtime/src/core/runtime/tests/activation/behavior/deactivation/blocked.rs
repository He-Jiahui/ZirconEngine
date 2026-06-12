use std::sync::Arc;

use super::super::super::super::super::*;
use super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind, StartupMode};

mod exact_five_dependency_matcher;

#[test]
fn deactivate_single_service_module_reports_external_dependent() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("SingleOwnerModule", ServiceKind::Driver, "ClockDriver");
    let dependent_manager_name = RegistryName::from_parts(
        "SingleDependentModule",
        ServiceKind::Manager,
        "ClockManager",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("SingleOwnerModule", "single owner").with_driver(
                DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SingleDependentModule", "single dependent").with_manager(
                ManagerDescriptor::new(
                    dependent_manager_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name.clone())],
                    Arc::new(|core| {
                        let _ = core
                            .resolve_driver::<TestDriver>("SingleOwnerModule.Driver.ClockDriver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();

    runtime.activate_module("SingleOwnerModule").unwrap();
    runtime.activate_module("SingleDependentModule").unwrap();

    let error = runtime.deactivate_module("SingleOwnerModule").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected single-service unload to report the running external dependent");
    };
    assert_eq!(blocked_service, driver_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);

    let handle = runtime.handle();
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(driver_name.as_str())
            .expect("blocked single-service deactivate should keep the owner service");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }

    runtime.deactivate_module("SingleDependentModule").unwrap();
    runtime.deactivate_module("SingleOwnerModule").unwrap();

    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(driver_name.as_str())
        .expect("successful single-service deactivate should keep the owner service entry");
    assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
    assert!(entry.instance.is_none());
}

#[test]
fn deactivate_blocks_when_external_dependents_are_alive() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts("ModuleA", ServiceKind::Driver, "ClockDriver");
    let local_manager_name =
        RegistryName::from_parts("ModuleA", ServiceKind::Manager, "LocalManager");
    let dependent_manager_name =
        RegistryName::from_parts("ModuleB", ServiceKind::Manager, "ClockManager");

    runtime
        .register_module(
            ModuleDescriptor::new("ModuleA", "a")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    local_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(ModuleDescriptor::new("ModuleB", "b").with_manager(
            ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![DependencySpec::named(driver_name.clone())],
                Arc::new(|core| {
                    let _ = core.resolve_driver::<TestDriver>("ModuleA.Driver.ClockDriver")?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            ),
        ))
        .unwrap();

    runtime.activate_module("ModuleA").unwrap();
    runtime.activate_module("ModuleB").unwrap();
    let error = runtime.deactivate_module("ModuleA").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected unload block when a running external dependent is alive");
    };
    assert_eq!(blocked_service, driver_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("ModuleA")
            .expect("blocked deactivate should keep module registered");
        assert_eq!(module.lifecycle, LifecycleState::Running);
    }
    {
        let services = handle.inner.services.lock().unwrap();
        for service_name in [&driver_name, &local_manager_name] {
            let entry = services
                .get(service_name.as_str())
                .expect("blocked deactivate should keep owner service registered");
            assert_eq!(entry.lifecycle, LifecycleState::Running);
            assert!(entry.instance.is_some());
        }
    }

    runtime.deactivate_module("ModuleB").unwrap();
    runtime.deactivate_module("ModuleA").unwrap();

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &local_manager_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("successful deactivate should keep owner service entry");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
    }
}

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
fn deactivate_reports_first_blocked_service_in_shutdown_order() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts("OrderedModule", ServiceKind::Driver, "Driver");
    let manager_name = RegistryName::from_parts("OrderedModule", ServiceKind::Manager, "Manager");
    let plugin_name = RegistryName::from_parts("OrderedModule", ServiceKind::Plugin, "Plugin");
    let driver_dependent_name =
        RegistryName::from_parts("DependentModule", ServiceKind::Manager, "DriverDependent");
    let plugin_dependent_name =
        RegistryName::from_parts("DependentModule", ServiceKind::Manager, "PluginDependent");

    runtime
        .register_module(
            ModuleDescriptor::new("OrderedModule", "ordered blocked unload")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name,
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
            ModuleDescriptor::new("DependentModule", "dependents")
                .with_manager(ManagerDescriptor::new(
                    driver_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name)],
                    Arc::new(|core| {
                        let _ = core.resolve_driver::<TestDriver>("OrderedModule.Driver.Driver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    plugin_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(plugin_name.clone())],
                    Arc::new(move |core| {
                        let _ = core.resolve_plugin::<TestManager>(plugin_name.as_str())?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("OrderedModule").unwrap();
    runtime.activate_module("DependentModule").unwrap();

    let error = runtime.deactivate_module("OrderedModule").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected unload block when multiple external dependents are alive");
    };
    assert_eq!(blocked_service, "OrderedModule.Plugin.Plugin");
    assert_eq!(dependents, vec![plugin_dependent_name.to_string()]);
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
