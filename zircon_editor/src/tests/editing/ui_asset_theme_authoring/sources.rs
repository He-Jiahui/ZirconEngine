use super::*;

#[test]
fn ui_asset_editor_session_projects_theme_sources_and_selection() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = crate::tests::support::load_test_ui_asset(IMPORTED_THEME_ASSET_TOML)
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

    let local_pane = session.pane_presentation();
    assert_eq!(
        local_pane.theme_source_items,
        vec![
            "Local Theme • 1 tokens • 1 rules".to_string(),
            "res://ui/theme/shared_theme.zui • 1 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(local_pane.theme_source_selected_index, 0);
    assert_eq!(local_pane.theme_selected_source_kind, "Local");
    assert_eq!(local_pane.theme_selected_source_reference, "local");
    assert_eq!(local_pane.theme_selected_source_token_count, 1);
    assert_eq!(local_pane.theme_selected_source_rule_count, 1);
    assert!(local_pane.theme_selected_source_available);
    assert!(local_pane.theme_can_promote_local);
    assert_eq!(
        local_pane.theme_selected_source_token_items,
        vec!["accent = \"#4488ff\"".to_string()]
    );
    assert_eq!(
        local_pane.theme_selected_source_rule_items,
        vec!["local_theme • #RootLabel".to_string()]
    );
    assert_eq!(
        local_pane.theme_cascade_layer_items,
        vec![
            "1. Imported • res://ui/theme/shared_theme.zui • 1 tokens • 1 rules".to_string(),
            "2. Local • 1 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(
        local_pane.theme_cascade_token_items,
        vec![
            "active • accent • Local = \"#4488ff\"".to_string(),
            "active • border • res://ui/theme/shared_theme.zui = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        local_pane.theme_cascade_rule_items,
        vec![
            "1. Imported • res://ui/theme/shared_theme.zui • shared_theme • Label".to_string(),
            "2. Local • local_theme • #RootLabel".to_string(),
        ]
    );

    assert!(session
        .select_theme_source(1)
        .expect("select imported theme"));
    let imported_pane = session.pane_presentation();
    assert_eq!(imported_pane.theme_source_selected_index, 1);
    assert_eq!(imported_pane.theme_selected_source_kind, "Imported");
    assert_eq!(
        imported_pane.theme_selected_source_reference,
        "res://ui/theme/shared_theme.zui"
    );
    assert_eq!(imported_pane.theme_selected_source_token_count, 1);
    assert_eq!(imported_pane.theme_selected_source_rule_count, 1);
    assert!(imported_pane.theme_selected_source_available);
    assert_eq!(
        imported_pane.theme_selected_source_token_items,
        vec!["border = \"#223344\"".to_string()]
    );
    assert_eq!(
        imported_pane.theme_selected_source_rule_items,
        vec!["shared_theme • Label".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_reports_missing_imported_theme_details_as_unavailable() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    assert!(session
        .select_theme_source(1)
        .expect("select unresolved imported theme"));
    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_selected_source_reference,
        "res://ui/theme/shared_theme.zui"
    );
    assert_eq!(pane.theme_selected_source_kind, "Imported");
    assert!(!pane.theme_selected_source_available);
    assert_eq!(pane.theme_selected_source_token_items, Vec::<String>::new());
    assert_eq!(pane.theme_selected_source_rule_items, Vec::<String>::new());
}

#[test]
fn ui_asset_editor_session_resolves_selected_theme_source_asset_id_only_for_available_imports() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = crate::tests::support::load_test_ui_asset(IMPORTED_THEME_ASSET_TOML)
        .expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    assert_eq!(session.selected_theme_source_asset_id(), None);

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");
    assert_eq!(
        session.selected_theme_source_asset_id().as_deref(),
        Some("res://ui/theme/shared_theme.zui")
    );

    let mut missing_session = UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(
            "res://ui/tests/theme-summary.zui",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("missing theme summary session");
    missing_session
        .select_theme_source(1)
        .expect("select unresolved imported theme");
    assert_eq!(missing_session.selected_theme_source_asset_id(), None);
}

#[test]
fn ui_asset_editor_session_projects_and_updates_promote_theme_draft_fields() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.theme_promote_asset_id,
        "res://ui/themes/theme_summary_theme.zui"
    );
    assert_eq!(
        initial.theme_promote_document_id,
        "ui.theme.theme_summary_theme"
    );
    assert_eq!(initial.theme_promote_display_name, "Theme Summary Theme");
    assert!(initial.theme_can_edit_promote_draft);

    assert!(session
        .set_promote_theme_asset_id("res://ui/themes/custom/editor_shell.zui")
        .expect("set promote theme asset id"));
    assert!(session
        .set_promote_theme_document_id("ui.theme.custom.editor_shell")
        .expect("set promote theme document id"));
    assert!(session
        .set_promote_theme_display_name("Editor Shell Theme")
        .expect("set promote theme display name"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.theme_promote_asset_id,
        "res://ui/themes/custom/editor_shell.zui"
    );
    assert_eq!(
        updated.theme_promote_document_id,
        "ui.theme.custom.editor_shell"
    );
    assert_eq!(updated.theme_promote_display_name, "Editor Shell Theme");
}

#[test]
fn ui_asset_editor_session_projects_local_cascade_theme_helpers_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = crate::tests::support::load_test_ui_asset(IMPORTED_THEME_ASSET_TOML)
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

    let pane = session.pane_presentation();
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade tokens into local layer (1)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade rules into local layer (1)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade changes into local layer (2)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade token • border = \"#223344\"".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade rule • shared_theme • Label".to_string()));

    let helper_index = pane
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt active cascade changes into local layer (2)")
        .expect("batch local cascade helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply local cascade helper"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("batch adopted local cascade source");
    assert_eq!(
        document.tokens.get("border"),
        Some(&Value::String("#223344".to_string()))
    );
    let imported_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "shared_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Label"))
        .expect("local adopted imported rule");
    assert_eq!(
        imported_rule.set.self_values.get("text"),
        Some(&Value::String("Imported Theme".to_string()))
    );
}

#[test]
fn ui_asset_editor_session_detaches_selected_imported_theme_into_local_theme_layer() {
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
        .detach_selected_theme_source_to_local()
        .expect("detach imported theme into local layer"));

    let pane = session.pane_presentation();
    assert_eq!(pane.theme_source_selected_index, 0);
    assert_eq!(pane.theme_selected_source_kind, "Local");
    assert_eq!(pane.theme_selected_source_reference, "local");
    assert_eq!(pane.theme_selected_source_token_count, 3);
    assert_eq!(pane.theme_selected_source_rule_count, 2);
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
    assert_eq!(
        pane.theme_source_items,
        vec!["Local Theme • 3 tokens • 2 rules".to_string()]
    );

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("detached theme source");
    assert!(document.imports.styles.is_empty());
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
    assert_eq!(
        document.stylesheets[0].rules[0]
            .set
            .self_values
            .get("text")
            .and_then(toml::Value::as_str),
        Some("$panel")
    );
}
