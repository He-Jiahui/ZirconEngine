use std::sync::Arc;

use super::super::super::super::state::{ServiceEntry, ServiceEntryFactory};
use super::super::super::super::*;
use super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind, StartupMode};

#[test]
fn register_single_service_reports_existing_service_table_key() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("OrphanServiceModule", ServiceKind::Driver, "ClockDriver");
    let handle = runtime.handle();

    {
        let mut services = handle.inner.services.lock().unwrap();
        services.insert(
            service_name.clone(),
            ServiceEntry {
                index: 1,
                generation: ServiceEntry::initial_generation(),
                startup_mode: StartupMode::Lazy,
                dependencies: Arc::default(),
                factory: ServiceEntryFactory::Service(Arc::new(|_| {
                    Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                })),
                lifecycle: LifecycleState::Registered,
                initialization_owner: None,
                instance: None,
            },
        );
    }

    let error = runtime
        .register_module(
            ModuleDescriptor::new("OrphanServiceModule", "orphaned service duplicate").with_driver(
                DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DuplicateService(name) if name == service_name.as_str()
    ));

    let modules = handle.inner.modules.lock().unwrap();
    assert!(!modules.contains_key("OrphanServiceModule"));
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    assert_eq!(services.len(), 1);
    assert!(services.contains_key(service_name.as_str()));
}

#[test]
fn register_exact_three_services_reports_existing_third_key_without_partial_commit() {
    let runtime = CoreRuntime::new();
    let first_name = RegistryName::from_parts(
        "ExactThreeDuplicateModule",
        ServiceKind::Driver,
        "ClockDriver",
    );
    let second_name = RegistryName::from_parts(
        "ExactThreeDuplicateModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let third_name = RegistryName::from_parts(
        "ExactThreeDuplicateModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let handle = runtime.handle();

    {
        let mut services = handle.inner.services.lock().unwrap();
        services.insert(
            third_name.clone(),
            ServiceEntry {
                index: 1,
                generation: ServiceEntry::initial_generation(),
                startup_mode: StartupMode::Lazy,
                dependencies: Arc::default(),
                factory: ServiceEntryFactory::Plugin(Arc::new(|_| {
                    Ok(Arc::new(TestManager) as ServiceObject)
                })),
                lifecycle: LifecycleState::Registered,
                initialization_owner: None,
                instance: None,
            },
        );
    }

    let error = runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeDuplicateModule", "exact-three duplicate")
                .with_driver(DriverDescriptor::new(
                    first_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    third_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DuplicateService(name) if name == third_name.as_str()
    ));

    let modules = handle.inner.modules.lock().unwrap();
    assert!(!modules.contains_key("ExactThreeDuplicateModule"));
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    assert_eq!(services.len(), 1);
    assert!(!services.contains_key(first_name.as_str()));
    assert!(!services.contains_key(second_name.as_str()));
    assert!(services.contains_key(third_name.as_str()));
}

#[test]
fn register_exact_four_services_reports_existing_fourth_key_without_partial_commit() {
    let runtime = CoreRuntime::new();
    let first_name = RegistryName::from_parts(
        "ExactFourDuplicateModule",
        ServiceKind::Driver,
        "ClockDriver",
    );
    let second_name = RegistryName::from_parts(
        "ExactFourDuplicateModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let third_name = RegistryName::from_parts(
        "ExactFourDuplicateModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let fourth_name = RegistryName::from_parts(
        "ExactFourDuplicateModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let handle = runtime.handle();

    {
        let mut services = handle.inner.services.lock().unwrap();
        services.insert(
            fourth_name.clone(),
            ServiceEntry {
                index: 1,
                generation: ServiceEntry::initial_generation(),
                startup_mode: StartupMode::Lazy,
                dependencies: Arc::default(),
                factory: ServiceEntryFactory::Plugin(Arc::new(|_| {
                    Ok(Arc::new(TestManager) as ServiceObject)
                })),
                lifecycle: LifecycleState::Registered,
                initialization_owner: None,
                instance: None,
            },
        );
    }

    let error = runtime
        .register_module(
            ModuleDescriptor::new("ExactFourDuplicateModule", "exact-four duplicate")
                .with_driver(DriverDescriptor::new(
                    first_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    third_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    fourth_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DuplicateService(name) if name == fourth_name.as_str()
    ));

    let modules = handle.inner.modules.lock().unwrap();
    assert!(!modules.contains_key("ExactFourDuplicateModule"));
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    assert_eq!(services.len(), 1);
    assert!(!services.contains_key(first_name.as_str()));
    assert!(!services.contains_key(second_name.as_str()));
    assert!(!services.contains_key(third_name.as_str()));
    assert!(services.contains_key(fourth_name.as_str()));
}

#[test]
fn register_exact_five_services_reports_existing_fifth_key_without_partial_commit() {
    let runtime = CoreRuntime::new();
    let first_name = RegistryName::from_parts(
        "ExactFiveDuplicateModule",
        ServiceKind::Driver,
        "ClockDriver",
    );
    let second_name = RegistryName::from_parts(
        "ExactFiveDuplicateModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let third_name = RegistryName::from_parts(
        "ExactFiveDuplicateModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let fourth_name = RegistryName::from_parts(
        "ExactFiveDuplicateModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let fifth_name = RegistryName::from_parts(
        "ExactFiveDuplicateModule",
        ServiceKind::Plugin,
        "EditorPlugin",
    );
    let handle = runtime.handle();

    {
        let mut services = handle.inner.services.lock().unwrap();
        services.insert(
            fifth_name.clone(),
            ServiceEntry {
                index: 1,
                generation: ServiceEntry::initial_generation(),
                startup_mode: StartupMode::Lazy,
                dependencies: Arc::default(),
                factory: ServiceEntryFactory::Plugin(Arc::new(|_| {
                    Ok(Arc::new(TestManager) as ServiceObject)
                })),
                lifecycle: LifecycleState::Registered,
                initialization_owner: None,
                instance: None,
            },
        );
    }

    let error = runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveDuplicateModule", "exact-five duplicate")
                .with_driver(DriverDescriptor::new(
                    first_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    third_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    fourth_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    fifth_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DuplicateService(name) if name == fifth_name.as_str()
    ));

    let modules = handle.inner.modules.lock().unwrap();
    assert!(!modules.contains_key("ExactFiveDuplicateModule"));
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    assert_eq!(services.len(), 1);
    assert!(!services.contains_key(first_name.as_str()));
    assert!(!services.contains_key(second_name.as_str()));
    assert!(!services.contains_key(third_name.as_str()));
    assert!(!services.contains_key(fourth_name.as_str()));
    assert!(services.contains_key(fifth_name.as_str()));
}
