use std::fs;

use zircon_runtime::scene::DefaultLevelManager;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::project::EditorProjectDocument;

use super::support::*;

fn write_project(project_root: &std::path::Path) {
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(project_root, &world, None).unwrap();
}

fn manager_for(path: &std::path::Path) -> std::sync::Arc<EditorManager> {
    let runtime = editor_runtime_with_config_path(path);
    runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap()
}

#[test]
fn editor_manager_refreshes_clean_ui_asset_session_from_external_file_change() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_clean");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_hot_reload_clean_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let manager = manager_for(&path);
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();

    let changed = SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External");
    write_ui_asset(&ui_asset_path, &changed);
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .expect("refresh external change");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(pane.source_text.contains("External"));
    assert!(!pane.source_dirty);
    assert!(!pane.has_external_conflict);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_marks_dirty_ui_asset_session_conflicted_without_overwriting_local_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_conflict");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_hot_reload_conflict_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let manager = manager_for(&path);
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Local"),
        )
        .unwrap();

    write_ui_asset(
        &ui_asset_path,
        &SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External"),
    );
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .unwrap();

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(pane.source_text.contains("Local"));
    assert!(!pane.source_text.contains("External"));
    assert!(pane.has_external_conflict);
    assert!(pane.can_reload_from_disk);
    assert!(pane.can_keep_local_and_save);

    let snapshot = manager
        .open_ui_asset_editor_diff_snapshot(&instance_id)
        .unwrap()
        .expect("diff snapshot");
    assert!(snapshot.local_source.contains("Local"));
    assert!(snapshot.external_source.contains("External"));
    assert_ne!(snapshot.local_hash, snapshot.external_hash);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_resolves_conflict_by_reloading_from_disk_or_keeping_local() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_resolution");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_hot_reload_resolution_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let manager = manager_for(&path);
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Local"),
        )
        .unwrap();
    write_ui_asset(
        &ui_asset_path,
        &SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External"),
    );
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .unwrap();

    assert!(manager
        .reload_ui_asset_editor_from_disk(&instance_id)
        .expect("reload from disk"));
    let reloaded = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(reloaded.source_text.contains("External"));
    assert!(!reloaded.has_external_conflict);

    manager
        .update_ui_asset_editor_source(
            &instance_id,
            SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Local Again"),
        )
        .unwrap();
    write_ui_asset(
        &ui_asset_path,
        &SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External Again"),
    );
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .unwrap();
    manager
        .keep_ui_asset_editor_local_and_save(&instance_id)
        .expect("keep local save");
    assert!(fs::read_to_string(&ui_asset_path)
        .unwrap()
        .contains("Local Again"));
    let kept = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(!kept.has_external_conflict);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_marks_and_recovers_stale_imports_from_watched_changes() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_stale_import");
    let project_root = unique_temp_dir("zircon_editor_asset_stale_import_project");
    write_project(&project_root);
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, DETACH_THEME_UI_LAYOUT_ASSET);
    write_ui_asset(&theme_path, IMPORTED_THEME_COLLISION_ASSET);

    let manager = manager_for(&path);
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .unwrap();
    assert!(
        manager
            .ui_asset_editor_pane_presentation(&instance_id)
            .unwrap()
            .preview_available
    );

    fs::write(&theme_path, "not = [valid").unwrap();
    manager
        .refresh_ui_asset_workspace_for_changes(vec![
            "res://ui/theme/shared_theme.ui.toml".to_string()
        ])
        .unwrap();
    let stale = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(stale.preview_available);
    assert!(stale
        .stale_import_items
        .iter()
        .any(|item| item.contains("res://ui/theme/shared_theme.ui.toml")));

    write_ui_asset(&theme_path, IMPORTED_THEME_COLLISION_ASSET);
    manager
        .refresh_ui_asset_workspace_for_changes(vec![
            "res://ui/theme/shared_theme.ui.toml".to_string()
        ])
        .unwrap();
    let recovered = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(recovered.preview_available);
    assert!(recovered.stale_import_items.is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}
