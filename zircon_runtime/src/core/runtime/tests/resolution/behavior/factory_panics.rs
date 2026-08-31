use super::*;
use std::time::Instant;

fn assert_factory_panicked(error: CoreError, service_name: &RegistryName) {
    assert!(matches!(
        error,
        CoreError::ServiceFactoryPanicked { service }
            if service == service_name.as_str()
    ));
}

fn assert_service_is_registered(runtime: &CoreRuntime, service_name: &RegistryName) {
    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(service_name)
        .expect("panicked factory service should remain registered");
    assert_eq!(entry.lifecycle, LifecycleState::Registered);
    assert!(entry.initialization_owner.is_none());
    assert!(entry.instance.is_none());
}

#[test]
fn lazy_manager_factory_panic_is_typed_and_retryable() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "LazyManagerPanicModule",
        ServiceKind::Manager,
        "LazyManager",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("LazyManagerPanicModule", "lazy manager panic").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            panic!("intentional lazy manager factory panic");
                        }
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("LazyManagerPanicModule").unwrap();

    let error = runtime
        .resolve_manager::<TestManager>(service_name.as_str())
        .unwrap_err();
    assert_factory_panicked(error, &service_name);
    assert_service_is_registered(&runtime, &service_name);

    runtime
        .resolve_manager::<TestManager>(service_name.as_str())
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    runtime.deactivate_module("LazyManagerPanicModule").unwrap();
    drop(runtime);
}

#[test]
fn lazy_plugin_factory_panic_is_typed_and_retryable() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("LazyPluginPanicModule", ServiceKind::Plugin, "LazyPlugin");
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("LazyPluginPanicModule", "lazy plugin panic").with_plugin(
                PluginDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            panic!("intentional lazy plugin factory panic");
                        }
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("LazyPluginPanicModule").unwrap();

    let error = runtime
        .resolve_plugin::<TestManager>(service_name.as_str())
        .unwrap_err();
    assert_factory_panicked(error, &service_name);
    assert_service_is_registered(&runtime, &service_name);

    runtime
        .resolve_plugin::<TestManager>(service_name.as_str())
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn immediate_factory_panic_preserves_the_typed_error_and_can_reactivate() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "ImmediateFactoryPanicModule",
        ServiceKind::Manager,
        "ImmediateManager",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ImmediateFactoryPanicModule", "immediate factory panic")
                .with_manager(ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            panic!("intentional immediate manager factory panic");
                        }
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    let error = runtime
        .activate_module("ImmediateFactoryPanicModule")
        .unwrap_err();
    assert_factory_panicked(error, &service_name);
    assert_service_is_registered(&runtime, &service_name);

    runtime
        .activate_module("ImmediateFactoryPanicModule")
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn panicked_factory_resets_the_claim_and_wakes_a_waiting_resolver() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "FactoryPanicWaiterModule",
        ServiceKind::Manager,
        "LazyManager",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let factory_release = Arc::clone(&release);
    let (factory_entered_tx, factory_entered_rx) = mpsc::channel();

    runtime
        .register_module(
            ModuleDescriptor::new("FactoryPanicWaiterModule", "factory panic waiter").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            factory_entered_tx.send(()).unwrap();
                            let (released, changed) = factory_release.as_ref();
                            let guard = released.lock().unwrap();
                            drop(changed.wait_while(guard, |released| !*released).unwrap());
                            panic!("intentional concurrent factory panic");
                        }
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("FactoryPanicWaiterModule").unwrap();

    let first_runtime = runtime.clone();
    let first_name = service_name.clone();
    let (result_tx, result_rx) = mpsc::channel();
    let first_result_tx = result_tx.clone();
    let first = thread::spawn(move || {
        first_result_tx
            .send((
                "first",
                first_runtime.resolve_manager::<TestManager>(first_name.as_str()),
            ))
            .unwrap();
    });
    factory_entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("first resolver should enter the factory");

    let second_runtime = runtime.clone();
    let second_name = service_name.clone();
    let second = thread::spawn(move || {
        result_tx
            .send((
                "second",
                second_runtime.resolve_manager::<TestManager>(second_name.as_str()),
            ))
            .unwrap();
    });

    let handle = runtime.handle();
    let mut waiter_registered = false;
    let waiter_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < waiter_deadline {
        waiter_registered = !handle
            .inner
            .service_resolution_waits
            .lock()
            .unwrap()
            .is_empty();
        if waiter_registered {
            break;
        }
        thread::yield_now();
    }
    assert!(
        waiter_registered,
        "second resolver should register as a waiter"
    );

    let (released, changed) = release.as_ref();
    *released.lock().unwrap() = true;
    changed.notify_all();

    let mut first_result = None;
    let mut second_result = None;
    for _ in 0..2 {
        let (resolver, result) = result_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("both resolvers must reach a terminal result");
        match resolver {
            "first" => first_result = Some(result),
            "second" => second_result = Some(result),
            _ => unreachable!("unknown resolver identity"),
        }
    }
    let first_error = first_result.unwrap().unwrap_err();
    assert_factory_panicked(first_error, &service_name);
    second_result.unwrap().unwrap();
    first.join().unwrap();
    second.join().unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(handle
        .inner
        .service_resolution_waits
        .lock()
        .unwrap()
        .is_empty());
}
