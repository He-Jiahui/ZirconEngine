use super::*;

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
