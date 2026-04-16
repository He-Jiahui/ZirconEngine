use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam_channel::unbounded;
use serde_json::Value;

use super::*;
use crate::channel_util::recv_latest;
use crate::error::CoreError;
use crate::lifecycle::{ServiceKind, StartupMode};
use crate::types::ServiceObject;

#[derive(Debug)]
struct TestDriver {
    order: usize,
}

#[derive(Debug)]
struct TestManager;

#[test]
fn recv_latest_keeps_last_message() {
    let (sender, receiver) = unbounded();
    sender.send(1).unwrap();
    sender.send(2).unwrap();

    assert_eq!(recv_latest(&receiver), Some(2));
    assert_eq!(recv_latest::<i32>(&receiver), None);
}

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
fn deactivate_blocks_when_external_dependents_are_alive() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(
            ModuleDescriptor::new("ModuleA", "a").with_driver(DriverDescriptor::new(
                RegistryName::from_parts("ModuleA", ServiceKind::Driver, "ClockDriver"),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
            )),
        )
        .unwrap();
    runtime
        .register_module(ModuleDescriptor::new("ModuleB", "b").with_manager(
            ManagerDescriptor::new(
                RegistryName::from_parts("ModuleB", ServiceKind::Manager, "ClockManager"),
                StartupMode::Immediate,
                vec![DependencySpec::named(RegistryName::from_parts(
                    "ModuleA",
                    ServiceKind::Driver,
                    "ClockDriver",
                ))],
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
    assert!(matches!(error, CoreError::UnloadBlocked(_, _)));
}

#[test]
fn event_bus_and_config_store_roundtrip() {
    let runtime = CoreRuntime::new();
    let events = runtime.handle().subscribe_events("editor.selection");
    runtime.publish_event("editor.selection", serde_json::json!({ "node": 7 }));
    let event = events.recv().unwrap();
    assert_eq!(event.payload["node"], 7);

    runtime
        .handle()
        .store_config("editor.theme", &serde_json::json!({ "name": "TokyoNight" }))
        .unwrap();
    let theme: Value = runtime.load_config("editor.theme").unwrap();
    assert_eq!(theme["name"], "TokyoNight");
}
