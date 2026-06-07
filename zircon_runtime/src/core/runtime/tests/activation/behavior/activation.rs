use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::error::CoreError;
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn immediate_services_activate_in_dependency_order() {
    let runtime = CoreRuntime::new();
    let order = Arc::new(AtomicUsize::new(0));

    let driver_order = order.clone();
    let driver = DriverDescriptor::new(
        RegistryName::from_parts("TestModule", ServiceKind::Driver, "ClockDriver"),
        StartupMode::Immediate,
        Vec::new(),
        Arc::new(move |_| {
            let order = driver_order.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(TestDriver { order }) as ServiceObject)
        }),
    );
    let manager = ManagerDescriptor::new(
        RegistryName::from_parts("TestModule", ServiceKind::Manager, "ClockManager"),
        StartupMode::Immediate,
        vec![DependencySpec::named(RegistryName::from_parts(
            "TestModule",
            ServiceKind::Driver,
            "ClockDriver",
        ))],
        Arc::new(move |core| {
            let _driver = core.resolve_driver::<TestDriver>("TestModule.Driver.ClockDriver")?;
            Ok(Arc::new(TestManager) as ServiceObject)
        }),
    );

    runtime
        .register_module(
            ModuleDescriptor::new("TestModule", "test")
                .with_driver(driver)
                .with_manager(manager),
        )
        .unwrap();
    runtime.activate_module("TestModule").unwrap();

    let driver = runtime
        .resolve_driver::<TestDriver>("TestModule.Driver.ClockDriver")
        .unwrap();
    assert_eq!(driver.order, 0);
}

#[test]
fn failed_immediate_activation_resets_module_and_service_lifecycle_for_retry() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "ActivationRetryModule",
        ServiceKind::Driver,
        "FlakyImmediateDriver",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ActivationRetryModule", "activation retry").with_driver(
                DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        if factory_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            return Err(CoreError::MissingConfig("retry.activation".to_owned()));
                        }
                        Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();

    let first_error = runtime
        .activate_module("ActivationRetryModule")
        .unwrap_err();
    assert!(matches!(
        first_error,
        CoreError::Initialization(name, detail)
            if name == service_name.as_str() && detail.contains("retry.activation")
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("ActivationRetryModule")
            .expect("failed activation should keep the module registered");
        assert_eq!(module.lifecycle, LifecycleState::Registered);
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(service_name.as_str())
            .expect("failed activation should keep the service registered");
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
    }

    runtime.activate_module("ActivationRetryModule").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ActivationRetryModule")
        .expect("retried activation should keep the module registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);

    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(service_name.as_str())
        .expect("retried activation should keep the service registered");
    assert_eq!(entry.lifecycle, LifecycleState::Running);
    assert!(entry.instance.is_some());
}

#[test]
fn resolving_immediate_service_uses_activation_instance_without_reinitializing() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "DirectImmediateModule",
        ServiceKind::Driver,
        "DirectImmediateDriver",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("DirectImmediateModule", "direct immediate resolve").with_driver(
                DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        let order = factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order }) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();

    let driver = runtime
        .resolve_driver::<TestDriver>(service_name.as_str())
        .unwrap();
    assert_eq!(driver.order, 0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("DirectImmediateModule")
        .expect("direct resolve should activate the owner module");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(service_name.as_str())
        .expect("direct resolve should keep the immediate service registered");
    assert_eq!(entry.lifecycle, LifecycleState::Running);
    assert!(entry.instance.is_some());
}

#[test]
fn activate_module_with_no_immediate_services_skips_startup_resolution_loop() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("LazyStartupModule", ServiceKind::Manager, "LazyManager");
    let calls = Arc::new(AtomicUsize::new(0));
    let factory_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("LazyStartupModule", "lazy startup").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        factory_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("LazyStartupModule")
            .expect("registered lazy module should be in module table");
        assert!(module.startup_service_names.is_empty());
        assert_eq!(module.lifecycle, LifecycleState::Registered);
    }

    runtime.activate_module("LazyStartupModule").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("LazyStartupModule")
            .expect("activated lazy module should remain in module table");
        assert_eq!(module.lifecycle, LifecycleState::Running);
    }

    let _ = runtime
        .resolve_manager::<TestManager>(service_name.as_str())
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn activate_exact_three_immediate_services_initializes_each_cached_startup_entry_once() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactThreeStartupModule", ServiceKind::Driver, "Driver");
    let manager_name =
        RegistryName::from_parts("ExactThreeStartupModule", ServiceKind::Manager, "Manager");
    let plugin_name =
        RegistryName::from_parts("ExactThreeStartupModule", ServiceKind::Plugin, "Plugin");
    let calls = Arc::new(AtomicUsize::new(0));
    let driver_calls = Arc::clone(&calls);
    let manager_calls = Arc::clone(&calls);
    let plugin_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeStartupModule", "exact-three startup")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        plugin_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactThreeStartupModule").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactThreeStartupModule")
        .expect("activated exact-three module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    assert_eq!(
        module.startup_service_names.as_ref(),
        [
            driver_name.clone(),
            manager_name.clone(),
            plugin_name.clone()
        ]
    );
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &manager_name, &plugin_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("exact-three startup should keep service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}

#[test]
fn activate_exact_four_immediate_services_initializes_each_cached_startup_entry_once() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFourStartupModule", ServiceKind::Driver, "Driver");
    let first_manager_name =
        RegistryName::from_parts("ExactFourStartupModule", ServiceKind::Manager, "Manager");
    let second_manager_name = RegistryName::from_parts(
        "ExactFourStartupModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let plugin_name =
        RegistryName::from_parts("ExactFourStartupModule", ServiceKind::Plugin, "Plugin");
    let calls = Arc::new(AtomicUsize::new(0));
    let driver_calls = Arc::clone(&calls);
    let first_manager_calls = Arc::clone(&calls);
    let second_manager_calls = Arc::clone(&calls);
    let plugin_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFourStartupModule", "exact-four startup")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        first_manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        second_manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        plugin_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactFourStartupModule").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 4);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFourStartupModule")
        .expect("activated exact-four module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    assert_eq!(
        module.startup_service_names.as_ref(),
        [
            driver_name.clone(),
            first_manager_name.clone(),
            second_manager_name.clone(),
            plugin_name.clone()
        ]
    );
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [
        &driver_name,
        &first_manager_name,
        &second_manager_name,
        &plugin_name,
    ] {
        let entry = services
            .get(service_name.as_str())
            .expect("exact-four startup should keep service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}

#[test]
fn activate_exact_five_immediate_services_initializes_each_cached_startup_entry_once() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactFiveStartupModule", ServiceKind::Driver, "Driver");
    let first_manager_name =
        RegistryName::from_parts("ExactFiveStartupModule", ServiceKind::Manager, "Manager");
    let second_manager_name = RegistryName::from_parts(
        "ExactFiveStartupModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let first_plugin_name =
        RegistryName::from_parts("ExactFiveStartupModule", ServiceKind::Plugin, "Plugin");
    let second_plugin_name = RegistryName::from_parts(
        "ExactFiveStartupModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let driver_calls = Arc::clone(&calls);
    let first_manager_calls = Arc::clone(&calls);
    let second_manager_calls = Arc::clone(&calls);
    let first_plugin_calls = Arc::clone(&calls);
    let second_plugin_calls = Arc::clone(&calls);

    runtime
        .register_module(
            ModuleDescriptor::new("ExactFiveStartupModule", "exact-five startup")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        driver_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        first_manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        second_manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_plugin(PluginDescriptor::new(
                    first_plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        first_plugin_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_plugin(PluginDescriptor::new(
                    second_plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        second_plugin_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("ExactFiveStartupModule").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 5);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactFiveStartupModule")
        .expect("activated exact-five module should remain in the module table");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    assert_eq!(
        module.startup_service_names.as_ref(),
        [
            driver_name.clone(),
            first_manager_name.clone(),
            second_manager_name.clone(),
            first_plugin_name.clone(),
            second_plugin_name.clone()
        ]
    );
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
            .expect("exact-five startup should keep service entries");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }
}
