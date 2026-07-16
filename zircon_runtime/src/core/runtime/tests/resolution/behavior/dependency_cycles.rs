use super::*;

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
