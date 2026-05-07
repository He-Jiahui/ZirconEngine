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
    assert!(pane.can_save_local_copy);

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
fn editor_manager_saves_dirty_conflict_local_source_as_copy_without_resolving_conflict() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_save_copy");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_hot_reload_save_copy_file").join("test.ui.toml");
    let copy_path =
        unique_temp_dir("zircon_editor_asset_hot_reload_save_copy_output").join("copy.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let manager = manager_for(&path);
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Local Copy"),
        )
        .unwrap();
    write_ui_asset(
        &ui_asset_path,
        &SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External Copy"),
    );
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .unwrap();

    let saved = manager
        .save_ui_asset_editor_local_copy(&instance_id, &copy_path)
        .expect("save local copy");
    assert!(saved.contains("Local Copy"));
    assert!(fs::read_to_string(&copy_path)
        .unwrap()
        .contains("Local Copy"));
    assert!(fs::read_to_string(&ui_asset_path)
        .unwrap()
        .contains("External Copy"));
    let adjacent_copy_path = manager
        .save_ui_asset_editor_local_copy_next_to_source(&instance_id)
        .expect("save adjacent local copy");
    assert!(adjacent_copy_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("local-copy"));
    assert!(fs::read_to_string(&adjacent_copy_path)
        .unwrap()
        .contains("Local Copy"));

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert!(pane.has_external_conflict);
    assert!(pane.source_dirty);
    assert!(pane.can_reload_from_disk);
    assert!(pane.can_keep_local_and_save);
    assert!(pane.can_save_local_copy);
    assert!(pane.can_open_diff_snapshot);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
    let _ = fs::remove_dir_all(copy_path.parent().unwrap());
}

#[test]
fn editor_manager_emergency_shell_reverts_invalid_ui_asset_source_to_last_valid() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_emergency_revert");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_emergency_revert_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let manager = manager_for(&path);
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    let edited = SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Edited");
    manager
        .update_ui_asset_editor_source(&instance_id, edited.clone())
        .unwrap();
    manager
        .update_ui_asset_editor_source(&instance_id, "not valid toml".to_string())
        .unwrap();

    let emergency = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert_eq!(emergency.shell_state, "Emergency");
    assert!(emergency.source_text.contains("not valid toml"));
    assert!(emergency.can_emergency_reload);
    assert!(emergency.can_emergency_revert);
    assert!(emergency.can_emergency_open_asset_browser);

    assert!(manager
        .revert_ui_asset_editor_to_last_valid(&instance_id)
        .expect("revert emergency source"));
    let recovered = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .unwrap();
    assert_eq!(recovered.shell_state, "Valid");
    assert!(recovered.source_text.contains("Edited"));
    assert!(!recovered.source_text.contains("not valid toml"));
    assert!(!recovered.can_emergency_revert);

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
