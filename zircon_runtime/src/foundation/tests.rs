use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam_channel::TryRecvError;

use crate::core::manager::{ManagerResolver, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME};
use crate::core::CoreRuntime;
use serde_json::json;

use crate::foundation::{module_descriptor, FOUNDATION_MODULE_NAME};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn foundation_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("foundation")
            .join("mod.rs"),
    )
    .expect("foundation mod source");

    for forbidden in [
        "pub struct FoundationModule",
        "impl EngineModule for FoundationModule",
        "fn module_name(&self)",
        "fn module_description(&self)",
        "fn descriptor(&self)",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected foundation/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn config_manager_roundtrip_works_through_resolver() {
    let _guard = env_lock().lock().unwrap();
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();

    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    config
        .set_value("editor.layout", json!({"dock": "main"}))
        .unwrap();

    assert_eq!(
        config.get_value("editor.layout").unwrap()["dock"],
        json!("main")
    );
}

#[test]
fn versioned_manager_handle_rejects_the_unloaded_generation() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let resolver = ManagerResolver::new(runtime.handle());
    let stale_handle = resolver.config_handle().unwrap();

    resolver.resolve(stale_handle.clone()).unwrap();
    runtime.deactivate_module(FOUNDATION_MODULE_NAME).unwrap();

    assert!(matches!(
        resolver.config_handle(),
        Err(crate::core::CoreError::ServiceUnavailable(name)) if name == CONFIG_MANAGER_NAME
    ));

    let error = match resolver.resolve(stale_handle.clone()) {
        Ok(_) => panic!("stale manager generation unexpectedly resolved"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        crate::core::CoreError::StaleServiceHandle {
            expected_index,
            expected_generation,
            actual_index,
            actual_generation,
            ..
        } if expected_index == stale_handle.index
            && expected_generation == stale_handle.generation
            && actual_index == stale_handle.index
            && actual_generation == stale_handle.generation + 1
    ));

    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let current_handle = resolver.config_handle().unwrap();
    assert_eq!(current_handle.index, stale_handle.index);
    assert_eq!(current_handle.generation, stale_handle.generation + 1);
    resolver.resolve(current_handle).unwrap();
}

#[test]
fn event_manager_publish_subscribe_roundtrip_works() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();

    let resolver = ManagerResolver::new(runtime.handle());
    let events = resolver.resolve(resolver.event_handle().unwrap()).unwrap();
    let receiver = events.subscribe("engine.ready");
    events.publish("engine.ready", json!({"ok": true}));

    let event = receiver.recv().unwrap();
    assert_eq!(event.payload["ok"], json!(true));
}

#[test]
fn foundation_registry_services_do_not_retain_the_runtime_root() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let weak = runtime.weak();

    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    let events = resolver.resolve(resolver.event_handle().unwrap()).unwrap();

    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "foundation registry services must not keep CoreRuntime alive"
    );
    assert_eq!(config.get_value("runtime.gone"), None);
    assert_eq!(
        config.set_value("runtime.gone", json!(true)),
        Err(crate::core::CoreError::RuntimeUnavailable)
    );

    let receiver = events.subscribe("runtime.gone");
    assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
    events.publish("runtime.gone", json!({"ignored": true}));
}

#[test]
fn config_manager_persists_values_to_disk() {
    let _guard = env_lock().lock().unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_config_{unique}.json"));
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);

    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    config
        .set_value("editor.workbench.default_layout", json!({"page": "main"}))
        .unwrap();

    let second_runtime = CoreRuntime::new();
    second_runtime.register_module(module_descriptor()).unwrap();
    second_runtime
        .activate_module(FOUNDATION_MODULE_NAME)
        .unwrap();
    let second_resolver = ManagerResolver::new(second_runtime.handle());
    let second_config = second_resolver
        .resolve(second_resolver.config_handle().unwrap())
        .unwrap();

    assert_eq!(
        second_config.get_value("editor.workbench.default_layout"),
        Some(json!({"page": "main"}))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn public_manager_services_use_foundation_module_registry_names() {
    assert_eq!(
        CONFIG_MANAGER_NAME,
        "FoundationModule.Manager.ConfigManager"
    );
    assert_eq!(EVENT_MANAGER_NAME, "FoundationModule.Manager.EventManager");
}
