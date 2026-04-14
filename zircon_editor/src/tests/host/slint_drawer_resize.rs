use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::CoreRuntime;
use zircon_manager::{resolve_config_manager, MANAGER_MODULE_NAME};

use crate::host::slint_host::drawer_resize::apply_resize_to_group;
use crate::{ActivityDrawerSlot, EditorManager, EDITOR_MANAGER_NAME};

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_manager::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_asset::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::module::module_descriptor())
        .unwrap();
    runtime
        .activate_module(MANAGER_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(crate::module::EDITOR_MODULE_NAME)
        .unwrap();
    runtime
}

#[test]
fn apply_resize_to_left_group_updates_all_left_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_left_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let _config = resolve_config_manager(&runtime.handle()).unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "left", 344.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(layout.drawers[&ActivityDrawerSlot::LeftTop].extent, 344.0);
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::LeftBottom].extent,
        344.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_bottom_group_updates_all_bottom_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_bottom_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "bottom", 228.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::BottomLeft].extent,
        228.0
    );
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::BottomRight].extent,
        228.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_right_group_updates_all_right_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_right_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "right", 312.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::RightTop].extent,
        312.0
    );
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::RightBottom].extent,
        312.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_unknown_group_returns_error() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_unknown_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let error = apply_resize_to_group(manager.as_ref(), "mystery", 240.0).unwrap_err();
    assert!(
        error.contains("Unsupported drawer resize target"),
        "unexpected error: {error}"
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
