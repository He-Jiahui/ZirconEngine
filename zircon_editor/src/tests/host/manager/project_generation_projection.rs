use std::fs;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::project::list_layout_preset_assets;
use crate::ui::workbench::startup::EditorStartupSessionDocument;
use zircon_runtime::asset::AssetUri;

use super::support::*;

#[test]
fn save_and_load_preset_roundtrip_through_manager_commands() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager
        .apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::SavePreset {
            name: "authoring".to_string(),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::LoadPreset {
            name: "authoring".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn save_and_load_preset_roundtrip_through_project_asset_files() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project_presets");
    let project_root = unique_temp_dir("zircon_editor_project_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();

    manager
        .apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
        .unwrap();

    let preset_asset = project_root
        .join("assets")
        .join("editor")
        .join("layout-presets")
        .join("rider.workbench-layout.json");
    assert!(
        preset_asset.exists(),
        "expected preset asset at {:?}",
        preset_asset
    );

    manager
        .apply_layout_command(LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn layout_preset_name_projection_reads_only_the_active_project_generation() {
    let source = include_str!("../../../ui/workbench/project/layout_preset_assets.rs");
    let source = source
        .split("pub(crate) fn list_layout_preset_assets")
        .nth(1)
        .expect("layout preset list projection should exist");
    let retired_open = ["ProjectManager", "::open"].concat();
    let retired_read_dir = ["fs", "::read_dir"].concat();
    let retired_read_to_string = ["fs", "::read_to_string"].concat();

    assert!(!source.contains(&retired_open));
    assert!(!source.contains(&retired_read_dir));
    assert!(!source.contains(&retired_read_to_string));
}

#[test]
fn layout_preset_projection_filters_large_locator_sets_without_a_project_snapshot_clone() {
    let mut locators = (0..1_000)
        .map(|index| AssetUri::parse(&format!("res://content/asset-{index}.wgsl")).unwrap())
        .collect::<Vec<_>>();
    locators.extend((0..100).map(|index| {
        AssetUri::parse(&format!(
            "res://editor/layout-presets/preset-{index}.workbench-layout.json"
        ))
        .unwrap()
    }));

    let names = list_layout_preset_assets(locators);

    assert_eq!(names.len(), 100);
    assert_eq!(names.first().map(String::as_str), Some("preset-0"));
    assert_eq!(names.last().map(String::as_str), Some("preset-99"));
}

#[test]
fn locator_and_preset_hot_paths_use_manager_owned_queries() {
    let project_access = include_str!("../../../ui/host/project_access.rs");
    let resolve_ui = project_access
        .split("pub(super) fn resolve_ui_asset_path")
        .nth(1)
        .unwrap()
        .split("pub(super) fn resolve_asset_locator_path")
        .next()
        .unwrap();
    let resolve_locator = project_access
        .split("pub(super) fn resolve_asset_locator_path")
        .nth(1)
        .unwrap()
        .split("pub(super) fn current_project_snapshot")
        .next()
        .unwrap();
    let layout = include_str!("../../../ui/host/layout_persistence.rs");
    let preset_projection = layout
        .split("pub(super) fn preset_names")
        .nth(1)
        .unwrap()
        .split("pub(super) fn load_global_default_layout")
        .next()
        .unwrap();

    assert!(resolve_ui.contains("current_project_source_path"));
    assert!(resolve_locator.contains("current_project_source_path"));
    assert!(!resolve_ui.contains("current_project_snapshot"));
    assert!(!resolve_locator.contains("current_project_snapshot"));
    assert!(preset_projection.contains("current_project_asset_uris"));
    assert!(!preset_projection.contains("current_project_snapshot"));
}

#[test]
fn layout_preset_operations_do_not_flatten_scene_project_errors_to_strings() {
    let layout = include_str!("../../../ui/host/layout_persistence.rs");

    assert!(layout
        .contains("let path = save_layout_preset_asset(&project, name, &self.current_layout())?;"));
    assert!(layout.contains("if let Some(layout) = load_layout_preset_asset(&project, name)? {"));
}

#[test]
fn editor_project_document_requires_an_explicit_project_generation() {
    let project_mod = include_str!("../../../ui/workbench/project/mod.rs");
    let load = include_str!("../../../ui/workbench/project/editor_project_document_load.rs");
    let save = include_str!("../../../ui/workbench/project/editor_project_document_save.rs");
    let retired_load_module = ["editor_project_document_load", "_from_path"].concat();
    let retired_save_module = ["editor_project_document_save", "_from_path"].concat();
    let retired_manager_open = ["ProjectManager", "::open"].concat();
    let retired_scan = ["scan", "_and_import"].concat();

    assert!(!project_mod.contains(&retired_load_module));
    assert!(!project_mod.contains(&retired_save_module));
    assert!(!load.contains(&retired_manager_open));
    assert!(!save.contains(&retired_manager_open));
    assert!(!load.contains(&retired_scan));
    assert!(!save.contains(&retired_scan));
}

#[test]
fn welcome_pane_projection_does_not_probe_the_filesystem() {
    let source = include_str!(
        "../../../ui/workbench/startup/editor_startup_session_document_welcome_pane_snapshot.rs"
    );
    let retired_authority = ["Project", "Authority"].concat();
    let retired_probe = ["probe", "_draft"].concat();
    let retired_creation_validation = ["validate", "_for_creation"].concat();

    assert!(!source.contains(&retired_authority));
    assert!(!source.contains(&retired_probe));
    assert!(!source.contains(&retired_creation_validation));
}

#[test]
fn welcome_pane_projection_uses_cached_probe_result() {
    let mut session = EditorStartupSessionDocument::default();
    session.creation_validation.clear();
    session.can_open_existing = true;

    let ready = session.welcome_pane_snapshot(false);

    assert!(ready.form.can_create);
    assert!(ready.form.can_open_existing);

    session.creation_validation = "project location is unavailable".to_string();
    session.can_open_existing = false;

    let unavailable = session.welcome_pane_snapshot(false);

    assert!(!unavailable.form.can_create);
    assert!(!unavailable.form.can_open_existing);
    assert_eq!(
        unavailable.form.validation_message,
        "project location is unavailable"
    );
}

#[test]
fn project_to_welcome_probe_lifecycle_is_surface_owned_not_project_mode_owned() {
    let present = include_str!("../../../ui/retained_host/app/welcome_session/session/present.rs");
    let apply = include_str!("../../../ui/retained_host/app/welcome_session/session/apply.rs");
    let probe = include_str!("../../../ui/retained_host/app/welcome_session/project_probe.rs");
    let startup_views =
        include_str!("../../../ui/retained_host/app/welcome_session/actions/startup_views.rs");
    let retired_mode_gate = ["startup_session", ".mode != EditorSessionMode::Welcome"].concat();

    assert!(present.contains("self.schedule_welcome_project_probe();"));
    assert!(apply.contains("self.clear_welcome_project_probe();"));
    assert!(!probe.contains(&retired_mode_gate));
    assert_eq!(
        startup_views
            .matches("self.clear_welcome_project_probe();")
            .count(),
        2
    );
}
