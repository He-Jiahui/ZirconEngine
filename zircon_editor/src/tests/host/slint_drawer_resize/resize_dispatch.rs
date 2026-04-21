use std::fs;

use zircon_runtime::core::manager::resolve_config_manager;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::slint_host::drawer_resize::apply_resize_to_group;
use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::support::{editor_runtime_with_config_path, env_lock, unique_temp_path};

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

    std::env::remove_var("ZIRCON_CONFIG_PATH");
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

    std::env::remove_var("ZIRCON_CONFIG_PATH");
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
    assert_eq!(layout.drawers[&ActivityDrawerSlot::RightTop].extent, 312.0);
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::RightBottom].extent,
        312.0
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
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

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
