use super::support::*;

#[test]
fn ui_asset_editor_session_creates_stylesheet_rule_from_selected_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session.create_rule_from_selection().expect("create rule"));

    let reflection = session.reflection_model();
    assert!(reflection.source_dirty);
    assert!(reflection
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == "#SaveButton"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let created = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("created rule");
    assert_eq!(created.selector, "#SaveButton");
    assert!(created.set.self_values.is_empty());
    assert!(created.set.slot.is_empty());
}

#[test]
fn ui_asset_editor_session_extracts_inline_overrides_into_stylesheet_rule() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .extract_inline_overrides_to_rule()
        .expect("extract inline overrides"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert!(button.style_overrides.self_values.is_empty());
    assert!(button.style_overrides.slot.is_empty());

    let extracted = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("extracted rule");
    assert_eq!(extracted.selector, "#SaveButton");
    assert_eq!(
        extracted
            .set
            .self_values
            .get("text")
            .and_then(|value| value.get("color"))
            .and_then(toml::Value::as_str),
        Some("#ffffff")
    );
    assert_eq!(
        extracted
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(4)
    );
}

#[test]
fn ui_asset_editor_session_adds_and_removes_selection_classes() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .add_class_to_selection("toolbar")
        .expect("add toolbar class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["primary".to_string(), "toolbar".to_string()]
    );
    assert!(session.reflection_model().source_dirty);
    assert!(!session
        .add_class_to_selection("toolbar")
        .expect("duplicate add should no-op"));

    assert!(session
        .remove_class_from_selection("primary")
        .expect("remove primary class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["toolbar".to_string()]
    );
    assert!(!session
        .remove_class_from_selection("missing")
        .expect("missing remove should no-op"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.classes, vec!["toolbar".to_string()]);
}

#[test]
fn ui_asset_editor_session_selects_renames_and_deletes_local_stylesheet_rules() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_items,
        vec![
            ".primary".to_string(),
            ".primary:hover".to_string(),
            ".primary:disabled".to_string()
        ]
    );
    assert_eq!(initial_pane.style_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_selector, "");

    assert!(session
        .select_stylesheet_rule(1)
        .expect("select local stylesheet rule"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_selected_index, 1);
    assert_eq!(selected.style_selected_rule_selector, ".primary:hover");

    assert!(session
        .rename_selected_stylesheet_rule("Button.toolbar:hover")
        .expect("rename selected stylesheet rule"));
    let renamed_document =
        crate::tests::support::load_test_ui_asset(session.source_buffer().text())
            .expect("document");
    assert_eq!(
        renamed_document.stylesheets[0].rules[1].selector,
        "Button.toolbar:hover"
    );
    let renamed = session.pane_presentation();
    assert_eq!(renamed.style_rule_selected_index, 1);
    assert_eq!(
        renamed.style_selected_rule_selector,
        "Button.toolbar:hover".to_string()
    );

    assert!(session
        .delete_selected_stylesheet_rule()
        .expect("delete selected stylesheet rule"));
    let after_delete = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let selectors = after_delete.stylesheets[0]
        .rules
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        selectors,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    let deleted = session.pane_presentation();
    assert_eq!(
        deleted.style_rule_items,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    assert_eq!(deleted.style_rule_selected_index, 1);
    assert_eq!(
        deleted.style_selected_rule_selector,
        ".primary:disabled".to_string()
    );
}

#[test]
fn ui_asset_editor_session_selects_upserts_and_deletes_stylesheet_rule_declarations() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .select_stylesheet_rule(0)
        .expect("select local stylesheet rule");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_declaration_items,
        vec!["self.background.color = \"#224488\"".to_string()]
    );
    assert_eq!(initial_pane.style_rule_declaration_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_declaration_path, "");
    assert_eq!(initial_pane.style_selected_rule_declaration_value, "");

    assert!(session
        .select_stylesheet_rule_declaration(0)
        .expect("select style declaration"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_declaration_selected_index, 0);
    assert_eq!(
        selected.style_selected_rule_declaration_path,
        "self.background.color"
    );
    assert_eq!(
        selected.style_selected_rule_declaration_value,
        "\"#224488\""
    );

    assert!(session
        .upsert_selected_stylesheet_rule_declaration("slot.padding", "6")
        .expect("rename style declaration"));
    let updated_document =
        crate::tests::support::load_test_ui_asset(session.source_buffer().text())
            .expect("document");
    let updated_rule = &updated_document.stylesheets[0].rules[0];
    assert!(updated_rule.set.self_values.is_empty());
    assert_eq!(
        updated_rule
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(6)
    );
    let updated = session.pane_presentation();
    assert_eq!(
        updated.style_rule_declaration_items,
        vec!["slot.padding = 6".to_string()]
    );
    assert_eq!(updated.style_rule_declaration_selected_index, 0);
    assert_eq!(updated.style_selected_rule_declaration_path, "slot.padding");
    assert_eq!(updated.style_selected_rule_declaration_value, "6");

    assert!(session
        .delete_selected_stylesheet_rule_declaration()
        .expect("delete style declaration"));
    let deleted_document =
        crate::tests::support::load_test_ui_asset(session.source_buffer().text())
            .expect("document");
    let deleted_rule = &deleted_document.stylesheets[0].rules[0];
    assert!(deleted_rule.set.self_values.is_empty());
    assert!(deleted_rule.set.slot.is_empty());
    let deleted = session.pane_presentation();
    assert!(deleted.style_rule_declaration_items.is_empty());
    assert_eq!(deleted.style_rule_declaration_selected_index, -1);
    assert_eq!(deleted.style_selected_rule_declaration_path, "");
    assert_eq!(deleted.style_selected_rule_declaration_value, "");
}

#[test]
fn ui_asset_editor_session_upserts_and_deletes_local_tokens() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string()
        ]
    );
    assert_eq!(initial_pane.style_token_selected_index, -1);
    assert_eq!(initial_pane.style_selected_token_name, "");
    assert_eq!(initial_pane.style_selected_token_value, "");

    assert!(session
        .upsert_style_token("surface_fill", "#223344")
        .expect("add token"));
    let added = session.pane_presentation();
    assert_eq!(
        added.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
    assert_eq!(added.style_token_selected_index, 2);
    assert_eq!(added.style_selected_token_name, "surface_fill");
    assert_eq!(added.style_selected_token_value, "\"#223344\"");

    assert!(session.select_style_token(0).expect("select accent token"));
    assert!(session
        .upsert_style_token("accent_primary", "#99bbff")
        .expect("rename selected token"));
    let renamed_document =
        crate::tests::support::load_test_ui_asset(session.source_buffer().text())
            .expect("document");
    assert!(!renamed_document.tokens.contains_key("accent"));
    assert_eq!(
        renamed_document
            .tokens
            .get("accent_primary")
            .and_then(toml::Value::as_str),
        Some("#99bbff")
    );

    assert!(session
        .delete_selected_style_token()
        .expect("delete selected token"));
    let deleted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert!(!deleted.tokens.contains_key("accent_primary"));
    assert_eq!(
        session.pane_presentation().style_token_items,
        vec![
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
}

#[test]
fn ui_asset_editor_session_toggles_pseudo_state_preview_matches() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(!session.reflection_model().source_dirty);
    assert_eq!(
        session
            .reflection_model()
            .style_inspector
            .active_pseudo_states,
        Vec::<String>::new()
    );
    assert!(!session
        .reflection_model()
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview"));
    let hover = session.reflection_model();
    assert_eq!(hover.style_inspector.active_pseudo_states, vec!["hover"]);
    assert!(hover
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("disabled")
        .expect("enable disabled preview"));
    let disabled = session.reflection_model();
    assert_eq!(
        disabled.style_inspector.active_pseudo_states,
        vec!["hover", "disabled"]
    );
    assert!(disabled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:disabled"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("disable hover preview"));
    let toggled = session.reflection_model();
    assert_eq!(
        toggled.style_inspector.active_pseudo_states,
        vec!["disabled"]
    );
    assert!(!toggled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));
    assert!(!toggled.source_dirty);
}

#[test]
fn ui_asset_editor_session_projects_matched_style_rules_into_stylesheet_summary_items() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let pane = session.pane_presentation();
    assert!(pane
        .stylesheet_items
        .contains(&"selection selector: #SaveButton".to_string()));
    assert!(pane.stylesheet_items.contains(&"states: hover".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary (spec 10, order 0)".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary:hover (spec 20, order 1)".to_string()));
}

#[test]
fn ui_asset_editor_session_selects_matched_style_rules_and_projects_details() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_matched_rule_items,
        vec![
            ".primary [editor.test.style_authoring::local]".to_string(),
            ".primary:hover [editor.test.style_authoring::local]".to_string()
        ]
    );
    assert_eq!(initial_pane.style_matched_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_origin, "");
    assert_eq!(initial_pane.style_selected_matched_rule_selector, "");
    assert_eq!(initial_pane.style_selected_matched_rule_specificity, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_source_order, -1);
    assert!(initial_pane
        .style_selected_matched_rule_declaration_items
        .is_empty());

    assert!(session
        .select_matched_style_rule(1)
        .expect("select matched style rule"));
    let selected_pane = session.pane_presentation();
    assert_eq!(selected_pane.style_matched_rule_selected_index, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_origin,
        "editor.test.style_authoring::local"
    );
    assert_eq!(
        selected_pane.style_selected_matched_rule_selector,
        ".primary:hover"
    );
    assert_eq!(selected_pane.style_selected_matched_rule_specificity, 20);
    assert_eq!(selected_pane.style_selected_matched_rule_source_order, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_declaration_items,
        vec!["self.text.color = \"#ffeeaa\"".to_string()]
    );
}
