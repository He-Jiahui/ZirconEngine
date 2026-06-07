use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::lifecycle::{ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn service_table_is_keyed_by_canonical_registry_names() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("KeyedModule", ServiceKind::Manager, "KeyedManager");

    runtime
        .register_module(
            ModuleDescriptor::new("KeyedModule", "typed service key").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ),
            ),
        )
        .unwrap();

    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let stored_key = services
        .keys()
        .next()
        .expect("registered module should contribute one service");

    assert!(services.contains_key(service_name.as_str()));
    assert_eq!(stored_key.as_str(), service_name.as_str());
    assert_eq!(stored_key.module_name(), "KeyedModule");
    assert_eq!(stored_key.service_kind(), ServiceKind::Manager);
    assert_eq!(stored_key.service_name(), "KeyedManager");
    let service_entry = services
        .get(service_name.as_str())
        .expect("registered service should keep an entry");
    assert!(service_entry.dependencies.is_empty());
    drop(services);

    let modules = handle.inner.modules.lock().unwrap();
    let module_entry = modules
        .get("KeyedModule")
        .expect("registered module should keep its owner service list");
    assert_eq!(module_entry.service_names.len(), 1);
    assert_eq!(module_entry.service_names[0], service_name);
    assert!(module_entry.startup_service_names.is_empty());
    assert_eq!(module_entry.shutdown_service_names.len(), 1);
    assert_eq!(module_entry.shutdown_service_names[0], service_name);
}

#[test]
fn register_exact_three_dependencies_keeps_direct_dependency_name_cache() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("ThreeDependencyModule", ServiceKind::Driver, "ClockDriver");
    let second_driver_name =
        RegistryName::from_parts("ThreeDependencyModule", ServiceKind::Driver, "AudioDriver");
    let third_driver_name =
        RegistryName::from_parts("ThreeDependencyModule", ServiceKind::Driver, "InputDriver");
    let manager_name = RegistryName::from_parts(
        "ThreeDependencyModule",
        ServiceKind::Manager,
        "SceneManager",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("ThreeDependencyModule", "three dependency name cache")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    vec![
                        DependencySpec::named(first_driver_name.clone()),
                        DependencySpec::named(second_driver_name.clone()),
                        DependencySpec::named(third_driver_name.clone()),
                    ],
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let manager_entry = services
        .get(manager_name.as_str())
        .expect("manager should be registered with dependency names");
    assert_eq!(
        manager_entry.dependencies.as_ref(),
        [
            first_driver_name.clone(),
            second_driver_name.clone(),
            third_driver_name.clone()
        ]
    );
}

#[test]
fn register_exact_four_dependencies_keeps_direct_dependency_name_cache() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("FourDependencyModule", ServiceKind::Driver, "ClockDriver");
    let second_driver_name =
        RegistryName::from_parts("FourDependencyModule", ServiceKind::Driver, "AudioDriver");
    let third_driver_name =
        RegistryName::from_parts("FourDependencyModule", ServiceKind::Driver, "InputDriver");
    let fourth_driver_name =
        RegistryName::from_parts("FourDependencyModule", ServiceKind::Driver, "RenderDriver");
    let manager_name =
        RegistryName::from_parts("FourDependencyModule", ServiceKind::Manager, "SceneManager");

    runtime
        .register_module(
            ModuleDescriptor::new("FourDependencyModule", "four dependency name cache")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    vec![
                        DependencySpec::named(first_driver_name.clone()),
                        DependencySpec::named(second_driver_name.clone()),
                        DependencySpec::named(third_driver_name.clone()),
                        DependencySpec::named(fourth_driver_name.clone()),
                    ],
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let manager_entry = services
        .get(manager_name.as_str())
        .expect("manager should be registered with dependency names");
    assert_eq!(
        manager_entry.dependencies.as_ref(),
        [
            first_driver_name.clone(),
            second_driver_name.clone(),
            third_driver_name.clone(),
            fourth_driver_name.clone()
        ]
    );
}

#[test]
fn register_exact_five_dependencies_keeps_direct_dependency_name_cache() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Driver, "ClockDriver");
    let second_driver_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Driver, "AudioDriver");
    let third_driver_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Driver, "InputDriver");
    let fourth_driver_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Driver, "RenderDriver");
    let fifth_driver_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Driver, "PhysicsDriver");
    let manager_name =
        RegistryName::from_parts("FiveDependencyModule", ServiceKind::Manager, "SceneManager");

    runtime
        .register_module(
            ModuleDescriptor::new("FiveDependencyModule", "five dependency name cache")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fifth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 4 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    vec![
                        DependencySpec::named(first_driver_name.clone()),
                        DependencySpec::named(second_driver_name.clone()),
                        DependencySpec::named(third_driver_name.clone()),
                        DependencySpec::named(fourth_driver_name.clone()),
                        DependencySpec::named(fifth_driver_name.clone()),
                    ],
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let manager_entry = services
        .get(manager_name.as_str())
        .expect("manager should be registered with dependency names");
    assert_eq!(
        manager_entry.dependencies.as_ref(),
        [
            first_driver_name.clone(),
            second_driver_name.clone(),
            third_driver_name.clone(),
            fourth_driver_name.clone(),
            fifth_driver_name.clone()
        ]
    );
}
