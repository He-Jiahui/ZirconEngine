use super::*;

#[test]
fn ui_asset_editor_session_adopts_imported_theme_rule_body_helper_items() {
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

    let token_helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported token • accent = \"#223344\"")
        .expect("imported token helper");
    assert!(session
        .apply_theme_rule_helper_item(token_helper_index)
        .expect("apply imported token helper"));

    let token_document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("token helper source");
    assert_eq!(
        token_document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );

    let rule_helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported rule • local_theme • Button")
        .expect("imported rule helper");
    assert!(session
        .apply_theme_rule_helper_item(rule_helper_index)
        .expect("apply imported rule helper"));

    let rule_document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("rule helper source");
    let button_rule = rule_document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("local imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );
}

#[test]
fn ui_asset_editor_session_applies_theme_batch_adopt_helper_items() {
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
        .position(|item| item == "Adopt all imported changes (3)")
        .expect("batch imported theme change helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply batch imported theme change helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("batch adopted theme source");
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
        .expect("batch adopted imported rule");
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

#[test]
fn ui_asset_editor_session_prunes_selected_theme_compare_duplicates() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

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
        .position(|item| item == "Prune compare duplicates shared with selected theme (3)")
        .expect("compare prune helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply compare prune helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("compare pruned theme source");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.zui".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_applies_theme_refactor_items_individually() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");

    let before = session.pane_presentation();
    assert_eq!(
        before.theme_refactor_items,
        vec![
            "duplicate local token • accent • inherited = \"#223344\"".to_string(),
            "duplicate local token • panel • inherited = \"$accent\"".to_string(),
            "duplicate local rule • local_theme • Button".to_string(),
            "redundant imported theme • res://ui/theme/shared_theme.zui".to_string(),
        ]
    );

    assert!(session
        .apply_theme_refactor_item(0)
        .expect("remove duplicate token"));
    let after_token = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("token pruned source");
    assert!(!after_token.tokens.contains_key("accent"));
    assert!(after_token.tokens.contains_key("panel"));
    assert_eq!(after_token.stylesheets[0].rules.len(), 1);

    assert!(session
        .apply_theme_refactor_item(1)
        .expect("remove duplicate rule"));
    let after_rule = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("rule pruned source");
    assert!(!after_rule.tokens.contains_key("accent"));
    assert!(after_rule.tokens.contains_key("panel"));
    assert!(after_rule.stylesheets.is_empty());
}

#[test]
fn ui_asset_editor_session_applies_all_theme_refactors_from_helper() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_COLLISION_ASSET_TOML)
            .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Apply all theme refactors (4)")
        .expect("batch theme refactor helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply batch theme refactor helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("batch refactored source");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert!(document.imports.styles.is_empty());
    assert!(session.pane_presentation().theme_refactor_items.is_empty());
}

#[test]
fn ui_asset_editor_session_projects_cross_asset_theme_rule_cascade_activity() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-multi-cascade.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme_a =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_CASCADE_A_ASSET_TOML)
            .expect("theme a");
    let imported_theme_b =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_CASCADE_B_ASSET_TOML)
            .expect("theme b");
    let mut session = UiAssetEditorSession::from_source(
        route,
        MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("multi cascade session");

    session
        .register_style_import("res://ui/theme/shared_a.zui", imported_theme_a)
        .expect("register theme a");
    session
        .register_style_import("res://ui/theme/shared_b.zui", imported_theme_b)
        .expect("register theme b");

    let pane = session.pane_presentation();
    assert!(pane
        .theme_cascade_token_items
        .contains(&"active • accent • Local = \"#5588ff\"".to_string()));
    assert!(pane
        .theme_cascade_token_items
        .contains(&"shadowed • accent • res://ui/theme/shared_b.zui = \"#334455\"".to_string()));
    assert!(pane
        .theme_cascade_token_items
        .contains(&"shadowed • accent • res://ui/theme/shared_a.zui = \"#112233\"".to_string()));
    assert!(pane.theme_cascade_rule_items.contains(
        &"active • rule • Button • Local • local_theme • self.text = \"Local Theme\"".to_string()
    ));
    assert!(pane
        .theme_cascade_rule_items
        .contains(
            &"shadowed • rule • Button • res://ui/theme/shared_b.zui • shared_theme_b • self.text = \"Imported Theme B\""
                .to_string(),
        ));
    assert!(pane
        .theme_cascade_rule_items
        .contains(
            &"shadowed • rule • Button • res://ui/theme/shared_a.zui • shared_theme_a • self.text = \"Imported Theme A\""
                .to_string(),
        ));
}

#[test]
fn ui_asset_editor_session_theme_compare_uses_active_imported_cascade_values() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-multi-cascade.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme_a =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_CASCADE_A_ASSET_TOML)
            .expect("theme a");
    let imported_theme_b =
        crate::tests::support::load_test_ui_asset(IMPORTED_THEME_CASCADE_B_ASSET_TOML)
            .expect("theme b");
    let mut session = UiAssetEditorSession::from_source(
        route,
        MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("multi cascade session");

    session
        .register_style_import("res://ui/theme/shared_a.zui", imported_theme_a)
        .expect("register theme a");
    session
        .register_style_import("res://ui/theme/shared_b.zui", imported_theme_b)
        .expect("register theme b");

    let pane = session.pane_presentation();
    assert!(pane.theme_compare_items.contains(
        &"overrides imported • token • accent • imported = \"#334455\" • local = \"#5588ff\""
            .to_string(),
    ));
    assert!(pane
        .theme_compare_items
        .contains(
            &"overrides imported • rule • local_theme • Button • imported self.text = \"Imported Theme B\" • local self.text = \"Local Theme\""
                .to_string(),
        ));
    assert!(!pane.theme_compare_items.contains(
        &"overrides imported • token • accent • imported = \"#112233\" • local = \"#5588ff\""
            .to_string(),
    ));
}
