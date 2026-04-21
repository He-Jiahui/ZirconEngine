use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::support::{
    cleanup_theme_project, setup_theme_project, DUPLICATE_IMPORTED_THEME_ASSET,
    DUPLICATE_THEME_UI_LAYOUT_ASSET,
};

#[test]
fn editor_manager_prunes_duplicate_local_theme_overrides() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_prune_duplicates",
        "zircon_editor_theme_prune_duplicates_project",
        DUPLICATE_THEME_UI_LAYOUT_ASSET,
        DUPLICATE_IMPORTED_THEME_ASSET,
    );
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");

    let before = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane before theme prune");
    assert!(before
        .theme_refactor_items
        .iter()
        .any(|item| item.contains("duplicate local token • accent")));
    assert!(before
        .theme_refactor_items
        .iter()
        .any(|item| item.contains("duplicate local rule • local_theme • Button")));

    assert!(manager
        .prune_ui_asset_editor_duplicate_local_theme_overrides(&instance_id)
        .expect("prune duplicate local theme overrides"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save pruned ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved pruned ui asset");
    assert!(!document.tokens.contains_key("accent"));
    assert!(!document.tokens.contains_key("panel"));
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );

    cleanup_theme_project(&config_path, &project_root);
}

#[test]
fn editor_manager_applies_theme_refactor_items_individually() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_apply_refactor",
        "zircon_editor_theme_apply_refactor_project",
        DUPLICATE_THEME_UI_LAYOUT_ASSET,
        DUPLICATE_IMPORTED_THEME_ASSET,
    );
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");

    assert!(manager
        .apply_ui_asset_editor_theme_refactor_item(&instance_id, 0)
        .expect("apply duplicate token refactor"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save refactored ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved refactored ui asset");
    assert!(!document.tokens.contains_key("accent"));
    assert!(document.tokens.contains_key("panel"));
    assert_eq!(document.stylesheets[0].rules.len(), 1);

    cleanup_theme_project(&config_path, &project_root);
}
