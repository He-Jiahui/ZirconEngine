use super::*;

#[test]
fn ui_asset_editor_session_projects_theme_compare_rule_body_diffs() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-diff.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_RULE_DIFF_ASSET_TOML)
            .expect("imported diff theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme diff session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported diff theme");

    let local_compare = session.pane_presentation();
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("overrides imported • rule • local_theme • Button")));
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("imported self.background.color = \"$accent\"")));
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("local self.text = \"$panel\"")));

    assert!(session
        .select_theme_source(1)
        .expect("select imported diff theme"));
    let imported_compare = session.pane_presentation();
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("shadowed by local • rule • local_theme • Button")));
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("self.text = \"Imported Theme\"")));
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("local self.text = \"$panel\"")));
}

#[test]
fn ui_asset_editor_session_clones_selected_imported_theme_into_local_theme_layer() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    assert!(session
        .clone_selected_theme_source_to_local()
        .expect("clone imported theme into local layer"));

    let pane = session.pane_presentation();
    assert_eq!(pane.theme_source_selected_index, 0);
    assert_eq!(pane.theme_selected_source_kind, "Local");
    assert_eq!(pane.theme_selected_source_reference, "local");
    assert_eq!(
        pane.theme_source_items,
        vec![
            "Local Theme • 3 tokens • 2 rules".to_string(),
            "res://ui/theme/shared_theme.zui • 2 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_selected_source_rule_items,
        vec![
            "shared_theme_local_theme • Button".to_string(),
            "local_theme • #RootLabel".to_string(),
        ]
    );

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("cloned theme source");
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.zui".to_string()]
    );
    assert_eq!(
        document.tokens.get("accent").and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(
        document
            .tokens
            .get("shared_theme_accent")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );
    assert_eq!(
        document.tokens.get("panel").and_then(toml::Value::as_str),
        Some("$shared_theme_accent")
    );
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_projects_and_applies_redundant_imported_theme_refactor_after_clone() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");
    assert!(session
        .clone_selected_theme_source_to_local()
        .expect("clone imported theme into local layer"));

    let before = session.pane_presentation();
    let redundant_index = before
        .theme_refactor_items
        .iter()
        .position(|item| item == "redundant imported theme • res://ui/theme/shared_theme.zui")
        .expect("redundant imported theme refactor");

    assert!(session
        .apply_theme_refactor_item(redundant_index)
        .expect("remove redundant imported theme"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("cloned theme source without redundant import");
    assert!(document.imports.styles.is_empty());
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_projects_local_theme_layer_merge_preview_for_imported_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_MERGE_PREVIEW_ASSET_TOML)
            .expect("imported merge preview theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_merge_preview_items,
        vec![
            "Detach • imports • res://ui/theme/base_tokens.zui".to_string(),
            "Detach • token • accent = \"#4488ff\"".to_string(),
            "Detach • token • panel = \"$shared_theme_accent\"".to_string(),
            "Detach • token • shared_theme_accent = \"#223344\"".to_string(),
            "Detach • rule • shared_theme_local_theme • Button".to_string(),
            "Detach • rule • local_theme • #RootLabel".to_string(),
            "Clone • imports • res://ui/theme/shared_theme.zui, res://ui/theme/base_tokens.zui"
                .to_string(),
            "Clone • token • accent = \"#4488ff\"".to_string(),
            "Clone • token • panel = \"$shared_theme_accent\"".to_string(),
            "Clone • token • shared_theme_accent = \"#223344\"".to_string(),
            "Clone • rule • shared_theme_local_theme • Button".to_string(),
            "Clone • rule • local_theme • #RootLabel".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_theme_compare_items_for_selected_imported_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_compare_items,
        vec![
            "shadowed by local • token • accent • imported = \"#223344\" • local = \"#4488ff\""
                .to_string(),
            "imported-only • token • panel = \"$accent\"".to_string(),
            "imported-only • rule • local_theme • Button • self.text = \"$panel\"".to_string(),
            "local-only • rule • local_theme • #RootLabel • self.text = \"Theme Summary Local\""
                .to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_selected_theme_compare_detail_items() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    assert!(session
        .select_theme_source(1)
        .expect("select imported theme source"));

    let compare_index = session
        .pane_presentation()
        .theme_compare_items
        .iter()
        .position(|item| item.contains("shadowed by local • token • accent"))
        .expect("compare token item");
    let pane = session.pane_presentation();
    let selected = pane
        .theme_compare_items
        .get(compare_index)
        .expect("selected compare item");
    assert!(selected.contains("shadowed by local"));
    assert!(selected.contains("token"));
    assert!(selected.contains("accent"));
}

#[test]
fn ui_asset_editor_session_applies_theme_rule_helper_items_for_selected_imports() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let before = session.pane_presentation();
    assert_eq!(
        before.theme_rule_helper_items,
        vec![
            "Detach res://ui/theme/shared_theme.zui into local theme layer".to_string(),
            "Clone res://ui/theme/shared_theme.zui into local theme layer".to_string(),
            "Adopt compare diffs from selected theme (3)".to_string(),
            "Adopt all imported tokens (2)".to_string(),
            "Adopt all imported rules (1)".to_string(),
            "Adopt all imported changes (3)".to_string(),
            "Adopt imported token • accent = \"#223344\"".to_string(),
            "Adopt imported token • panel = \"$accent\"".to_string(),
            "Adopt imported rule • local_theme • Button".to_string(),
        ]
    );

    assert!(session
        .apply_theme_rule_helper_item(0)
        .expect("apply detach helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("detached theme source");
    assert!(document.imports.styles.is_empty());
    assert!(document.tokens.contains_key("shared_theme_accent"));
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_applies_compare_diff_theme_helper_for_selected_import() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt compare diffs from selected theme (3)")
        .expect("compare diff helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply compare diff helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("compare diff adopted theme source");
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
        .expect("compare diff adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    let pane = session.pane_presentation();
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • accent = \"#223344\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • panel = \"$accent\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • rule • local_theme • Button"));
}
