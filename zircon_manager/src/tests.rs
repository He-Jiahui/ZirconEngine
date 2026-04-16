use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use zircon_core::CoreRuntime;

use crate::{
    module_descriptor, resolve_config_manager, ManagerResolver, CONFIG_MANAGER_NAME,
    EVENT_MANAGER_NAME, MANAGER_MODULE_NAME,
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn config_manager_roundtrip_works_through_resolver() {
    let _guard = env_lock().lock().unwrap();
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(MANAGER_MODULE_NAME).unwrap();

    let config = resolve_config_manager(&runtime.handle()).unwrap();
    config
        .set_value("editor.layout", json!({"dock": "main"}))
        .unwrap();

    assert_eq!(
        config.get_value("editor.layout").unwrap()["dock"],
        json!("main")
    );
}

#[test]
fn event_manager_publish_subscribe_roundtrip_works() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(MANAGER_MODULE_NAME).unwrap();

    let events = ManagerResolver::new(runtime.handle()).event().unwrap();
    let receiver = events.subscribe("engine.ready");
    events.publish("engine.ready", json!({"ok": true}));

    let event = receiver.recv().unwrap();
    assert_eq!(event.payload["ok"], json!(true));
}

#[test]
fn config_manager_persists_values_to_disk() {
    let _guard = env_lock().lock().unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_editor_config_{unique}.json"));
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", &path);

    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(MANAGER_MODULE_NAME).unwrap();
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    config
        .set_value("editor.workbench.default_layout", json!({"page": "main"}))
        .unwrap();

    let second_runtime = CoreRuntime::new();
    second_runtime.register_module(module_descriptor()).unwrap();
    second_runtime.activate_module(MANAGER_MODULE_NAME).unwrap();
    let second_config = second_runtime
        .handle()
        .resolve_manager::<crate::ConfigManagerHandle>(CONFIG_MANAGER_NAME)
        .unwrap()
        .shared();

    assert_eq!(
        second_config.get_value("editor.workbench.default_layout"),
        Some(json!({"page": "main"}))
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn public_manager_services_use_manager_module_registry_names() {
    assert_eq!(CONFIG_MANAGER_NAME, "ManagerModule.Manager.ConfigManager");
    assert_eq!(EVENT_MANAGER_NAME, "ManagerModule.Manager.EventManager");
}
