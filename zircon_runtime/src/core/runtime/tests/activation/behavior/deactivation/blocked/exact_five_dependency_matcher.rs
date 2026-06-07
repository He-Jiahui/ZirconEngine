use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::error::CoreError;
use crate::core::lifecycle::{ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn deactivate_exact_five_services_reports_first_shutdown_service_when_dependent_names_all() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherModule",
        ServiceKind::Driver,
        "Driver",
    );
    let first_manager_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherModule",
        ServiceKind::Manager,
        "Manager",
    );
    let second_manager_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let first_plugin_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherModule",
        ServiceKind::Plugin,
        "Plugin",
    );
    let second_plugin_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let dependent_manager_name = RegistryName::from_parts(
        "ExactFiveDependencyMatcherDependentModule",
        ServiceKind::Manager,
        "DependentManager",
    );
    let driver_name_for_factory = driver_name.clone();
    let first_manager_name_for_factory = first_manager_name.clone();
    let second_manager_name_for_factory = second_manager_name.clone();
    let first_plugin_name_for_factory = first_plugin_name.clone();
    let second_plugin_name_for_factory = second_plugin_name.clone();

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveDependencyMatcherModule", "exact-five matcher")
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
            ModuleDescriptor::new(
                "ExactFiveDependencyMatcherDependentModule",
                "exact-five dependent",
            )
            .with_manager(ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(driver_name.clone()),
                    DependencySpec::named(first_manager_name.clone()),
                    DependencySpec::named(second_manager_name.clone()),
                    DependencySpec::named(first_plugin_name.clone()),
                    DependencySpec::named(second_plugin_name.clone()),
                ],
                Arc::new(move |core| {
                    let _ = core.resolve_driver::<TestDriver>(driver_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_manager::<TestManager>(first_manager_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_manager::<TestManager>(second_manager_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_plugin::<TestManager>(first_plugin_name_for_factory.as_str())?;
                    let _ = core
                        .resolve_plugin::<TestManager>(second_plugin_name_for_factory.as_str())?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            )),
        )
        .unwrap();

    runtime
        .activate_module("ExactFiveDependencyMatcherModule")
        .unwrap();
    runtime
        .activate_module("ExactFiveDependencyMatcherDependentModule")
        .unwrap();

    let error = runtime
        .deactivate_module("ExactFiveDependencyMatcherModule")
        .unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected exact-five unload to report the first shutdown service");
    };
    assert_eq!(blocked_service, first_plugin_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);
}
