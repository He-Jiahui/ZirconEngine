use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Barrier, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use super::super::super::*;
use super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind, StartupMode};

mod dependency_cycles;

#[derive(Debug)]
struct HeldService {
    calls: Arc<AtomicUsize>,
}

impl HeldService {
    fn enter_call(&self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn held_service_reference_cannot_enter_after_module_deactivation() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("HeldServiceModule", ServiceKind::Manager, "HeldManager");
    let calls = Arc::new(AtomicUsize::new(0));
    let service_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("HeldServiceModule", "M0 held service reference").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(HeldService {
                            calls: Arc::clone(&service_calls),
                        }) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("HeldServiceModule").unwrap();

    let held_service = runtime
        .resolve_manager_handle::<HeldService>(service_name.as_str())
        .unwrap();
    let in_flight_call = held_service.enter().unwrap();
    let deactivating_runtime = runtime.clone();
    let deactivation_start = Arc::new(Barrier::new(2));
    let deactivation_start_for_thread = Arc::clone(&deactivation_start);
    let (deactivation_complete, deactivation_result) = mpsc::channel();
    let deactivation = thread::spawn(move || {
        deactivation_start_for_thread.wait();
        deactivation_complete.send(deactivating_runtime.deactivate_module("HeldServiceModule"))
    });
    deactivation_start.wait();

    let mut admission_closed = false;
    for _ in 0..1_000 {
        admission_closed = !runtime
            .handle()
            .inner
            .services
            .lock()
            .unwrap()
            .get(service_name.as_str())
            .unwrap()
            .admission_open;
        if admission_closed {
            break;
        }
        thread::yield_now();
    }
    assert!(
        admission_closed,
        "deactivation should close service admission first"
    );
    assert!(matches!(
        deactivation_result.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));

    drop(in_flight_call);
    deactivation_result
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .unwrap();
    deactivation.join().unwrap();

    assert!(matches!(
        held_service.enter(),
        Err(CoreError::StaleServiceHandle { .. })
    ));

    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "an unloaded service must reject a new call through an already-held reference"
    );
}

#[test]
fn service_call_guard_keeps_its_runtime_slot_alive_until_release() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "GuardRuntimeRetentionModule",
        ServiceKind::Manager,
        "GuardedManager",
    );
    runtime
        .register_module(
            ModuleDescriptor::new("GuardRuntimeRetentionModule", "M3 guard ownership")
                .with_manager(ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .activate_module("GuardRuntimeRetentionModule")
        .unwrap();

    let held_service = runtime
        .resolve_manager_handle::<TestManager>(service_name.as_str())
        .unwrap();
    let first_call = held_service.enter().unwrap();
    drop(runtime);

    let second_call = held_service
        .enter()
        .expect("an in-flight guard must retain the runtime slot for its release");
    drop(second_call);
    drop(first_call);

    assert!(matches!(
        held_service.enter(),
        Err(CoreError::RuntimeUnavailable)
    ));
}

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
fn concurrent_lazy_manager_resolve_executes_factory_once() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "ConcurrentLazyModule",
        ServiceKind::Manager,
        "ConcurrentLazyManager",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);
    let factory_release = Arc::new((Mutex::new(false), Condvar::new()));
    let factory_release_for_call = Arc::clone(&factory_release);
    let (factory_invoked_tx, factory_invoked_rx) = mpsc::channel();

    runtime
        .register_module(
            ModuleDescriptor::new("ConcurrentLazyModule", "concurrent lazy manager").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        factory_calls.fetch_add(1, Ordering::SeqCst);
                        factory_invoked_tx
                            .send(())
                            .expect("factory invocation should remain observable");
                        let (release, released) = factory_release_for_call.as_ref();
                        let guard = release.lock().unwrap();
                        drop(released.wait_while(guard, |release| !*release).unwrap());
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("ConcurrentLazyModule").unwrap();

    let first_runtime = runtime.clone();
    let first_service_name = service_name.clone();
    let first = thread::spawn(move || {
        first_runtime.resolve_manager::<TestManager>(first_service_name.as_str())
    });
    factory_invoked_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("first resolver should enter the lazy factory");

    let second_runtime = runtime.clone();
    let second_service_name = service_name.clone();
    let second_started = Arc::new(Barrier::new(2));
    let second_started_in_thread = Arc::clone(&second_started);
    let second = thread::spawn(move || {
        second_started_in_thread.wait();
        second_runtime.resolve_manager::<TestManager>(second_service_name.as_str())
    });
    second_started.wait();
    assert!(
        factory_invoked_rx
            .recv_timeout(Duration::from_millis(100))
            .is_err(),
        "a concurrent lazy resolve must wait for the in-flight factory",
    );

    let (release, released) = factory_release.as_ref();
    *release.lock().unwrap() = true;
    released.notify_all();

    let first = first.join().unwrap().unwrap();
    let second = second.join().unwrap().unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn concurrent_cyclic_lazy_manager_dependencies_return_without_deadlock() {
    let runtime = CoreRuntime::new();
    let first_name = RegistryName::from_parts(
        "ConcurrentCycleModule",
        ServiceKind::Manager,
        "FirstManager",
    );
    let second_name = RegistryName::from_parts(
        "ConcurrentCycleModule",
        ServiceKind::Manager,
        "SecondManager",
    );
    let factory_calls = Arc::new(AtomicUsize::new(0));
    let first_factory_calls = Arc::clone(&factory_calls);
    let second_factory_calls = Arc::clone(&factory_calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ConcurrentCycleModule", "concurrent dependency cycle")
                .with_manager(ManagerDescriptor::new(
                    first_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(second_name.clone())],
                    Arc::new(move |_| {
                        first_factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_name.clone(),
                    StartupMode::Lazy,
                    vec![DependencySpec::named(first_name.clone())],
                    Arc::new(move |_| {
                        second_factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime.activate_module("ConcurrentCycleModule").unwrap();

    runtime
        .handle()
        .install_service_resolution_claim_barrier(2, Arc::new(Barrier::new(2)));
    let (result_tx, result_rx) = mpsc::channel();

    let first_runtime = runtime.clone();
    let first_service_name = first_name.clone();
    let first_result_tx = result_tx.clone();
    let first = thread::spawn(move || {
        let result = first_runtime
            .resolve_manager::<TestManager>(first_service_name.as_str())
            .map(|_| ());
        first_result_tx.send(result).unwrap();
    });

    let second_runtime = runtime.clone();
    let second_service_name = second_name.clone();
    let second = thread::spawn(move || {
        let result = second_runtime
            .resolve_manager::<TestManager>(second_service_name.as_str())
            .map(|_| ());
        result_tx.send(result).unwrap();
    });

    for _ in 0..2 {
        let result = result_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("cyclic concurrent resolves must terminate within the bounded budget");
        match result {
            Err(CoreError::DependencyCycle(name)) => {
                assert!(name == first_name.as_str() || name == second_name.as_str());
            }
            other => panic!("expected dependency cycle, got {other:?}"),
        }
    }

    first.join().unwrap();
    second.join().unwrap();
    assert_eq!(factory_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn registered_manager_identity_is_unique_and_resolves_the_registered_generation() {
    let service_name =
        RegistryName::from_parts("IdentityModule", ServiceKind::Manager, "IdentityManager");
    let first_runtime = CoreRuntime::new();
    let second_runtime = CoreRuntime::new();

    for runtime in [&first_runtime, &second_runtime] {
        runtime
            .register_module(
                ModuleDescriptor::new("IdentityModule", "identity").with_manager(
                    ManagerDescriptor::new(
                        service_name.clone(),
                        StartupMode::Lazy,
                        Vec::new(),
                        Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                    ),
                ),
            )
            .unwrap();
        runtime.activate_module("IdentityModule").unwrap();
    }

    let first_handle = first_runtime.handle();
    let first_identity = first_handle
        .registered_manager_identity(service_name.as_str())
        .unwrap();
    let second_identity = second_runtime
        .handle()
        .registered_manager_identity(service_name.as_str())
        .unwrap();

    assert_ne!(first_identity.index(), second_identity.index());
    assert_eq!(first_identity.generation(), 1);
    assert_eq!(first_identity.service(), &service_name);
    first_handle
        .resolve_registered_manager::<TestManager>(&first_identity)
        .unwrap();
}

#[test]
fn deactivation_invalidates_registered_manager_identity_before_reactivation() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "StaleIdentityModule",
        ServiceKind::Manager,
        "IdentityManager",
    );
    runtime
        .register_module(
            ModuleDescriptor::new("StaleIdentityModule", "stale identity").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("StaleIdentityModule").unwrap();

    let handle = runtime.handle();
    let stale_identity = handle
        .registered_manager_identity(service_name.as_str())
        .unwrap();
    runtime.deactivate_module("StaleIdentityModule").unwrap();

    assert!(matches!(
        handle.registered_manager_identity(service_name.as_str()),
        Err(CoreError::ServiceUnavailable(name)) if name == service_name.as_str()
    ));
    assert!(matches!(
        handle.resolve_manager::<TestManager>(service_name.as_str()),
        Err(CoreError::ServiceUnavailable(name)) if name == service_name.as_str()
    ));

    let stale_error = handle
        .resolve_registered_manager::<TestManager>(&stale_identity)
        .unwrap_err();
    assert!(matches!(
        stale_error,
        CoreError::StaleServiceHandle {
            name,
            expected_index,
            expected_generation,
            actual_index,
            actual_generation,
        } if name == service_name.as_str()
            && expected_index == stale_identity.index()
            && expected_generation == stale_identity.generation()
            && actual_index == stale_identity.index()
            && actual_generation == stale_identity.generation() + 1
    ));

    runtime.activate_module("StaleIdentityModule").unwrap();
    let current_identity = handle
        .registered_manager_identity(service_name.as_str())
        .unwrap();
    assert_eq!(current_identity.index(), stale_identity.index());
    assert_eq!(
        current_identity.generation(),
        stale_identity.generation() + 1
    );
    handle
        .resolve_registered_manager::<TestManager>(&current_identity)
        .unwrap();
}

#[test]
fn lazy_factory_cannot_restore_service_after_concurrent_module_unload() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "ConcurrentUnloadModule",
        ServiceKind::Manager,
        "LazyManager",
    );
    let factory_entered = Arc::new(Barrier::new(2));
    let factory_release = Arc::new(Barrier::new(2));
    let entered = Arc::clone(&factory_entered);
    let release = Arc::clone(&factory_release);

    runtime
        .register_module(
            ModuleDescriptor::new("ConcurrentUnloadModule", "concurrent unload").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        entered.wait();
                        release.wait();
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("ConcurrentUnloadModule").unwrap();

    let resolver_runtime = runtime.clone();
    let resolved_service_name = service_name.clone();
    let resolve_thread = thread::spawn(move || {
        resolver_runtime.resolve_manager::<TestManager>(resolved_service_name.as_str())
    });
    factory_entered.wait();
    runtime.deactivate_module("ConcurrentUnloadModule").unwrap();
    factory_release.wait();

    assert!(matches!(
        resolve_thread.join().unwrap(),
        Err(CoreError::ServiceUnavailable(name)) if name == service_name.as_str()
    ));
    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(&service_name).unwrap();
    assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
    assert!(entry.instance.is_none());
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
