use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use serde_json::Value;

use super::*;
use crate::core::channel_util::recv_latest;
use crate::core::error::CoreError;
use crate::core::lifecycle::{ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[derive(Debug)]
struct TestDriver {
    order: usize,
}

#[derive(Debug)]
struct TestManager;

#[derive(Debug)]
struct RecordedPlugin(PluginContext);

#[test]
fn recv_latest_keeps_last_message() {
    let (sender, receiver) = unbounded();
    sender.send(1).unwrap();
    sender.send(2).unwrap();

    assert_eq!(recv_latest(&receiver), Some(2));
    assert_eq!(recv_latest::<i32>(&receiver), None);
}

#[test]
fn registry_name_accepts_only_exact_three_segment_names() {
    let valid = RegistryName::new("TestModule.Manager.ClockManager").unwrap();
    assert_eq!(valid.as_str(), "TestModule.Manager.ClockManager");

    for invalid in [
        "",
        "TestModule",
        "TestModule.Manager",
        ".Manager.ClockManager",
        "TestModule.Service.ClockManager",
        "TestModule..ClockManager",
        "TestModule.Manager.",
        " TestModule.Manager.ClockManager",
        "TestModule .Manager.ClockManager",
        "TestModule.Manager. ClockManager",
        "TestModule.Manager.ClockManager ",
        "TestModule.Manager.ClockManager.Extra",
    ] {
        let error = RegistryName::new(invalid).unwrap_err();
        assert!(matches!(
            error,
            CoreError::InvalidRegistryName(value) if value == invalid
        ));
    }
}

#[test]
fn registry_name_from_parts_builds_canonical_service_names() {
    let name = RegistryName::from_parts("TestModule", ServiceKind::Driver, "ClockDriver");

    assert_eq!(name.as_str(), "TestModule.Driver.ClockDriver");
    assert_eq!(name.module_name(), "TestModule");
    assert_eq!(name.service_kind(), ServiceKind::Driver);
    assert_eq!(name.service_name(), "ClockDriver");
}

#[test]
fn service_kind_registry_segments_are_canonical() {
    for kind in [
        ServiceKind::Driver,
        ServiceKind::Manager,
        ServiceKind::Plugin,
    ] {
        assert_eq!(
            ServiceKind::from_registry_segment(kind.as_str()),
            Some(kind)
        );
    }
    assert_eq!(ServiceKind::from_registry_segment("Service"), None);
    assert_eq!(ServiceKind::from_registry_segment("manager"), None);
}

#[test]
fn register_module_rejects_noncanonical_module_names() {
    for invalid in ["", " TestModule", "TestModule ", "\tTestModule"] {
        let runtime = CoreRuntime::new();
        let error = runtime
            .register_module(ModuleDescriptor::new(invalid, "invalid module name"))
            .unwrap_err();

        assert!(matches!(
            error,
            CoreError::InvalidModuleName(value) if value == invalid
        ));
    }
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
fn register_module_rejects_descriptor_registry_kind_mismatch() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("MismatchModule", "mismatched registry kind").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts("MismatchModule", ServiceKind::Manager, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceKindMismatch {
            name,
            expected: ServiceKind::Driver,
            actual: ServiceKind::Manager,
        } if name == "MismatchModule.Manager.ClockDriver"
    ));
}

#[test]
fn register_module_rejects_descriptor_registry_owner_mismatch() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("OwnerModule", "mismatched registry owner").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts("OtherModule", ServiceKind::Driver, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceOwnerMismatch {
            name,
            expected,
            actual,
        } if name == "OtherModule.Driver.ClockDriver"
            && expected == "OwnerModule"
            && actual == "OtherModule"
    ));
}

#[test]
fn register_module_rejects_driver_dependencies_on_managers() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("DependencyModule", "invalid driver dependency").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts(
                        "DependencyModule",
                        ServiceKind::Driver,
                        "ClockDriver",
                    ),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(RegistryName::from_parts(
                        "DependencyModule",
                        ServiceKind::Manager,
                        "ClockManager",
                    ))],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::InvalidServiceDependencyKind {
            service,
            service_kind: ServiceKind::Driver,
            dependency,
            dependency_kind: ServiceKind::Manager,
        } if service == "DependencyModule.Driver.ClockDriver"
            && dependency == "DependencyModule.Manager.ClockManager"
    ));
}

#[test]
fn register_module_does_not_leave_services_when_descriptor_validation_fails() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("RollbackModule", "failed descriptor registration")
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts("RollbackModule", ServiceKind::Driver, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts("RollbackModule", ServiceKind::Driver, "BadManager"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceKindMismatch {
            name,
            expected: ServiceKind::Manager,
            actual: ServiceKind::Driver,
        } if name == "RollbackModule.Driver.BadManager"
    ));
    let missing_driver = runtime
        .resolve_driver::<TestDriver>("RollbackModule.Driver.ClockDriver")
        .unwrap_err();
    assert!(matches!(
        missing_driver,
        CoreError::MissingService(name) if name == "RollbackModule.Driver.ClockDriver"
    ));
}

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

    let services = runtime.inner.services.lock().unwrap();
    let stored_key = services
        .keys()
        .next()
        .expect("registered module should contribute one service");

    assert!(services.contains_key(service_name.as_str()));
    assert_eq!(stored_key.as_str(), service_name.as_str());
    assert_eq!(stored_key.module_name(), "KeyedModule");
    assert_eq!(stored_key.service_kind(), ServiceKind::Manager);
    assert_eq!(stored_key.service_name(), "KeyedManager");

    let runtime_inner_source = include_str!("state/runtime_inner.rs");
    assert!(runtime_inner_source.contains("HashMap<RegistryName, ServiceEntry>"));
    assert!(!runtime_inner_source.contains("HashMap<String, ServiceEntry>"));
}

#[test]
fn activation_keeps_module_service_lists_as_registry_names() {
    let activation_source = include_str!("handle/activation.rs");

    assert!(activation_source.contains("Vec<RegistryName>"));
    assert!(activation_source.contains("HashSet<RegistryName>"));
    assert!(
        activation_source.contains("fn running_dependents(")
            && activation_source.contains("service_name: &RegistryName")
    );
    assert!(!activation_source.contains("HashSet<String>"));
    assert!(!activation_source.contains("entry.name.to_string()"));
}

#[test]
fn resolution_uses_registry_names_for_recursion_stack_and_dependency_walk() {
    let resolution_source = include_str!("handle/resolution.rs");

    assert!(resolution_source.contains("stack: &mut Vec<RegistryName>"));
    assert!(resolution_source.contains("let (owner_module, dependency_names, factory)"));
    assert!(resolution_source.contains(".map(|dependency| dependency.name.clone())"));
    assert!(!resolution_source.contains("stack: &mut Vec<String>"));
    assert!(!resolution_source.contains("stack.push(service_name.to_string())"));
    assert!(!resolution_source.contains("entry.dependencies.clone()"));
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
fn plugin_resolution_builds_plugin_context_instead_of_passing_only_core_handle() {
    let seen = Arc::new(Mutex::new(None::<PluginContext>));
    let seen_for_factory = Arc::clone(&seen);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    core.register_module(
        ModuleDescriptor::new("PluginContextSpec", "plugin context test").with_plugin(
            PluginDescriptor::new(
                RegistryName::from_parts(
                    "PluginContextSpec",
                    ServiceKind::Plugin,
                    "RecordedPlugin",
                ),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(move |context: &PluginContext| {
                    *seen_for_factory.lock().unwrap() = Some(context.clone());
                    Ok(Arc::new(RecordedPlugin(context.clone())) as ServiceObject)
                }),
            ),
        ),
    )
    .unwrap();

    core.activate_module("PluginContextSpec").unwrap();
    let resolved = core
        .resolve_plugin::<RecordedPlugin>("PluginContextSpec.Plugin.RecordedPlugin")
        .unwrap();
    let context = seen.lock().unwrap().clone().unwrap();

    assert_eq!(resolved.0.plugin_name, context.plugin_name);
    assert_eq!(
        context.plugin_name,
        "PluginContextSpec.Plugin.RecordedPlugin"
    );
    assert!(context.core.upgrade().is_some());
    assert!(context.package_root.is_none());
    assert!(context.source_root.is_none());
    assert!(context.data_root.is_none());
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
