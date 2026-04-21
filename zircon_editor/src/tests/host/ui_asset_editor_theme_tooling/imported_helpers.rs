use toml::Value;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::support::{
    cleanup_theme_project, setup_theme_project, DUPLICATE_IMPORTED_THEME_ASSET,
    DUPLICATE_THEME_UI_LAYOUT_ASSET, IMPORTED_THEME_COLLISION_ASSET, THEME_SUMMARY_LAYOUT_ASSET,
};

#[test]
fn editor_manager_applies_theme_rule_helper_items_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_apply_helper",
        "zircon_editor_theme_apply_helper_project",
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
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, 0)
        .expect("apply detach helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save helper-applied ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved helper ui asset");
    assert!(document.imports.styles.is_empty());
    assert!(document.tokens.contains_key("shared_theme_accent"));

    cleanup_theme_project(&config_path, &project_root);
}

#[test]
fn editor_manager_applies_compare_diff_theme_helper_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_compare_diff_helper",
        "zircon_editor_theme_compare_diff_helper_project",
        THEME_SUMMARY_LAYOUT_ASSET,
        IMPORTED_THEME_COLLISION_ASSET,
    );
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt compare diffs from selected theme (3)")
        .expect("compare diff theme helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, helper_index)
        .expect("apply compare diff theme helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save compare diff helper-applied ui asset editor");
    let document = crate::tests::support::load_test_ui_asset(&saved)
        .expect("saved compare diff helper ui asset");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    assert_eq!(
        document.tokens.get("panel"),
        Some(&Value::String("$accent".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("adopted compare diff imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    cleanup_theme_project(&config_path, &project_root);
}

#[test]
fn editor_manager_applies_theme_rule_body_helper_items_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_rule_body_helper",
        "zircon_editor_theme_rule_body_helper_project",
        THEME_SUMMARY_LAYOUT_ASSET,
        IMPORTED_THEME_COLLISION_ASSET,
    );
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let token_helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported token • accent = \"#223344\"")
        .expect("imported token helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, token_helper_index)
        .expect("apply imported token helper"));

    let rule_helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported rule • local_theme • Button")
        .expect("imported rule helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, rule_helper_index)
        .expect("apply imported rule helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save helper-applied ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved helper ui asset");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    cleanup_theme_project(&config_path, &project_root);
}
