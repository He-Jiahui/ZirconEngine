use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::super::super::*;
use super::super::fixtures::{TestDriver, TestManager};
use crate::core::error::CoreError;
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn lazy_manager_is_created_on_first_resolve() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let manager_calls = calls.clone();

    runtime
        .register_module(ModuleDescriptor::new("LazyModule", "lazy").with_manager(
            ManagerDescriptor::new(
                RegistryName::from_parts("LazyModule", ServiceKind::Manager, "LazyManager"),
                StartupMode::Lazy,
                Vec::new(),
                Arc::new(move |_| {
                    manager_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            ),
        ))
        .unwrap();
    runtime.activate_module("LazyModule").unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let _ = runtime
        .resolve_manager::<TestManager>("LazyModule.Manager.LazyManager")
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_lazy_manager_initialization_resets_lifecycle_and_can_retry() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("RetryModule", ServiceKind::Manager, "FlakyManager");
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(ModuleDescriptor::new("RetryModule", "retry").with_manager(
            ManagerDescriptor::new(
                service_name.clone(),
                StartupMode::Lazy,
                Vec::new(),
                Arc::new(move |_| {
                    if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        return Err(CoreError::MissingConfig("retry.once".to_owned()));
                    }
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            ),
        ))
        .unwrap();
    runtime.activate_module("RetryModule").unwrap();

    let first_error = runtime
        .resolve_manager::<TestManager>(service_name.as_str())
        .unwrap_err();
    assert!(matches!(
        first_error,
        CoreError::Initialization(name, detail)
            if name == service_name.as_str() && detail.contains("retry.once")
    ));

    let handle = runtime.handle();
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(service_name.as_str())
            .expect("flaky manager should stay registered after a failed resolve");
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
    }

    let _ = runtime
        .resolve_manager::<TestManager>(service_name.as_str())
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(service_name.as_str())
        .expect("flaky manager should stay in the service table");
    assert_eq!(entry.lifecycle, LifecycleState::Running);
    assert!(entry.instance.is_some());
}

#[test]
fn failed_dependency_initialization_resets_dependent_service_and_can_retry() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("DependencyRetryModule", ServiceKind::Driver, "FlakyDriver");
    let manager_name = RegistryName::from_parts(
        "DependencyRetryModule",
        ServiceKind::Manager,
        "DependentManager",
    );
    let driver_calls = Arc::new(AtomicUsize::new(0));
    let manager_calls = Arc::new(AtomicUsize::new(0));
    let driver_factory_calls = Arc::clone(&driver_calls);
    let manager_factory_calls = Arc::clone(&manager_calls);

    runtime
        .register_module(
            ModuleDescriptor::new("DependencyRetryModule", "dependency retry")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        if driver_factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            return Err(CoreError::MissingConfig("retry.dependency".to_owned()));
                        }
                        Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(driver_name.clone())],
                    Arc::new(move |_| {
                        manager_factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime.activate_module("DependencyRetryModule").unwrap();

    let first_error = runtime
        .resolve_manager::<TestManager>(manager_name.as_str())
        .unwrap_err();
    assert!(matches!(
        first_error,
        CoreError::Initialization(name, detail)
            if name == driver_name.as_str() && detail.contains("retry.dependency")
    ));
    assert_eq!(driver_calls.load(Ordering::SeqCst), 1);
    assert_eq!(manager_calls.load(Ordering::SeqCst), 0);

    let handle = runtime.handle();
    {
        let services = handle.inner.services.lock().unwrap();
        for service_name in [&driver_name, &manager_name] {
            let entry = services
                .get(service_name.as_str())
                .expect("failed dependency chain should stay registered");
            assert_eq!(entry.lifecycle, LifecycleState::Registered);
            assert!(entry.instance.is_none());
        }
    }

    let _ = runtime
        .resolve_manager::<TestManager>(manager_name.as_str())
        .unwrap();
    assert_eq!(driver_calls.load(Ordering::SeqCst), 2);
    assert_eq!(manager_calls.load(Ordering::SeqCst), 1);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &manager_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("retried dependency chain should stay in the service table");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}

#[test]
fn resolve_exact_four_dependencies_initializes_cached_keys_directly() {
    let runtime = CoreRuntime::new();
    let first_driver_name = RegistryName::from_parts(
        "ExactFourDependencyModule",
        ServiceKind::Driver,
        "FirstDriver",
    );
    let second_driver_name = RegistryName::from_parts(
        "ExactFourDependencyModule",
        ServiceKind::Driver,
        "SecondDriver",
    );
    let third_driver_name = RegistryName::from_parts(
        "ExactFourDependencyModule",
        ServiceKind::Driver,
        "ThirdDriver",
    );
    let fourth_driver_name = RegistryName::from_parts(
        "ExactFourDependencyModule",
        ServiceKind::Driver,
        "FourthDriver",
    );
    let manager_name =
        RegistryName::from_parts("ExactFourDependencyModule", ServiceKind::Manager, "Manager");
    let dependency_calls = Arc::new(AtomicUsize::new(0));
    let first_driver_calls = Arc::clone(&dependency_calls);
    let second_driver_calls = Arc::clone(&dependency_calls);
    let third_driver_calls = Arc::clone(&dependency_calls);
    let fourth_driver_calls = Arc::clone(&dependency_calls);
    let manager_calls = Arc::new(AtomicUsize::new(0));
    let manager_factory_calls = Arc::clone(&manager_calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourDependencyModule", "exact four dependency")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        first_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        second_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        third_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        fourth_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)
                    }),
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
                    Arc::new(move |_| {
                        manager_factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime
        .activate_module("ExactFourDependencyModule")
        .unwrap();

    let _ = runtime
        .resolve_manager::<TestManager>(manager_name.as_str())
        .unwrap();

    assert_eq!(dependency_calls.load(Ordering::SeqCst), 4);
    assert_eq!(manager_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn resolve_exact_five_dependencies_initializes_cached_keys_directly() {
    let runtime = CoreRuntime::new();
    let first_driver_name = RegistryName::from_parts(
        "ExactFiveDependencyModule",
        ServiceKind::Driver,
        "FirstDriver",
    );
    let second_driver_name = RegistryName::from_parts(
        "ExactFiveDependencyModule",
        ServiceKind::Driver,
        "SecondDriver",
    );
    let third_driver_name = RegistryName::from_parts(
        "ExactFiveDependencyModule",
        ServiceKind::Driver,
        "ThirdDriver",
    );
    let fourth_driver_name = RegistryName::from_parts(
        "ExactFiveDependencyModule",
        ServiceKind::Driver,
        "FourthDriver",
    );
    let fifth_driver_name = RegistryName::from_parts(
        "ExactFiveDependencyModule",
        ServiceKind::Driver,
        "FifthDriver",
    );
    let manager_name =
        RegistryName::from_parts("ExactFiveDependencyModule", ServiceKind::Manager, "Manager");
    let dependency_calls = Arc::new(AtomicUsize::new(0));
    let first_driver_calls = Arc::clone(&dependency_calls);
    let second_driver_calls = Arc::clone(&dependency_calls);
    let third_driver_calls = Arc::clone(&dependency_calls);
    let fourth_driver_calls = Arc::clone(&dependency_calls);
    let fifth_driver_calls = Arc::clone(&dependency_calls);
    let manager_calls = Arc::new(AtomicUsize::new(0));
    let manager_factory_calls = Arc::clone(&manager_calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveDependencyModule", "exact five dependency")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        first_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        second_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        third_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        fourth_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)
                    }),
                ))
                .with_driver(DriverDescriptor::new(
                    fifth_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        fifth_driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 4 }) as ServiceObject)
                    }),
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
                    Arc::new(move |_| {
                        manager_factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime
        .activate_module("ExactFiveDependencyModule")
        .unwrap();

    let _ = runtime
        .resolve_manager::<TestManager>(manager_name.as_str())
        .unwrap();

    assert_eq!(dependency_calls.load(Ordering::SeqCst), 5);
    assert_eq!(manager_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn four_frame_resolution_cycle_reports_canonical_registry_key() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("FourFrameCycleModule", ServiceKind::Driver, "FirstDriver");
    let second_driver_name =
        RegistryName::from_parts("FourFrameCycleModule", ServiceKind::Driver, "SecondDriver");
    let third_driver_name =
        RegistryName::from_parts("FourFrameCycleModule", ServiceKind::Driver, "ThirdDriver");
    let fourth_driver_name =
        RegistryName::from_parts("FourFrameCycleModule", ServiceKind::Driver, "FourthDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FourFrameCycleModule", "four frame cycle")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(second_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(third_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(fourth_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(first_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime.activate_module("FourFrameCycleModule").unwrap();

    let error = runtime
        .resolve_driver::<TestDriver>(first_driver_name.as_str())
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DependencyCycle(name) if name == first_driver_name.as_str()
    ));
}

#[test]
fn five_frame_resolution_cycle_reports_canonical_registry_key() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("FiveFrameCycleModule", ServiceKind::Driver, "FirstDriver");
    let second_driver_name =
        RegistryName::from_parts("FiveFrameCycleModule", ServiceKind::Driver, "SecondDriver");
    let third_driver_name =
        RegistryName::from_parts("FiveFrameCycleModule", ServiceKind::Driver, "ThirdDriver");
    let fourth_driver_name =
        RegistryName::from_parts("FiveFrameCycleModule", ServiceKind::Driver, "FourthDriver");
    let fifth_driver_name =
        RegistryName::from_parts("FiveFrameCycleModule", ServiceKind::Driver, "FifthDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FiveFrameCycleModule", "five frame cycle")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(second_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(third_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(fourth_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fourth_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(fifth_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 3 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    fifth_driver_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(first_driver_name.clone())],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 4 }) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime.activate_module("FiveFrameCycleModule").unwrap();

    let error = runtime
        .resolve_driver::<TestDriver>(first_driver_name.as_str())
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::DependencyCycle(name) if name == first_driver_name.as_str()
    ));
}
