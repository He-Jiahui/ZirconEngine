use super::super::support::*;

#[test]
fn ui_asset_editor_session_projects_structured_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/binding-authoring.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        BINDING_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> menu_action.workbench.project.save".to_string()]
    );
    assert_eq!(pane.inspector_binding_selected_index, 0);
    assert_eq!(pane.inspector_binding_id, "SaveButton/onClick");
    assert_eq!(pane.inspector_binding_event, "onClick");
    assert_eq!(
        pane.inspector_binding_route,
        "menu_action.workbench.project.save"
    );
    assert_eq!(
        pane.inspector_binding_route_target,
        "menu_action.workbench.project.save"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
}

#[test]
fn ui_asset_editor_session_updates_selected_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.zui",
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
    assert!(session.add_binding().expect("add binding"));
    assert!(session
        .set_selected_binding_id("SaveButton/onHover")
        .expect("set selected binding id"));
    assert!(session
        .set_selected_binding_event("onHover")
        .expect("set selected binding event"));
    assert!(session
        .set_selected_binding_route("menu_action.workbench.highlight_save")
        .expect("set selected binding route"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_selected_index, 0);
    assert_eq!(updated.inspector_binding_id, "SaveButton/onHover");
    assert_eq!(updated.inspector_binding_event, "onHover");
    assert_eq!(
        updated.inspector_binding_route,
        "menu_action.workbench.highlight_save"
    );
    assert_eq!(
        updated.inspector_binding_route_target,
        "menu_action.workbench.highlight_save"
    );
    assert_eq!(updated.inspector_binding_action_target, "");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("menu_action.workbench.highlight_save")
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec![
            "onClick | SaveButton/onClick -> menu_action.workbench.project.save (+2 payload)"
                .to_string()
        ]
    );
    assert_eq!(pane.inspector_binding_event_selected_index, 0);
    assert_eq!(pane.inspector_binding_action_kind_selected_index, 1);
    assert_eq!(
        pane.inspector_binding_action_kind_items,
        vec![
            "None".to_string(),
            "Route".to_string(),
            "Action".to_string()
        ]
    );
    assert_eq!(
        pane.inspector_binding_route,
        "menu_action.workbench.project.save"
    );
    assert_eq!(
        pane.inspector_binding_route_target,
        "menu_action.workbench.project.save"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
    assert_eq!(
        pane.inspector_binding_payload_items,
        vec!["confirm = true".to_string(), "mode = \"full\"".to_string()]
    );
    assert_eq!(pane.inspector_binding_payload_selected_index, 0);
    assert_eq!(pane.inspector_binding_payload_key, "confirm");
    assert_eq!(pane.inspector_binding_payload_value, "true");
}

#[test]
fn ui_asset_editor_session_updates_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .select_binding_event_option(1)
        .expect("select double click event"));
    assert!(session
        .select_binding_action_kind(2)
        .expect("select action kind"));
    assert!(session
        .set_selected_binding_action_target("editor_action.workbench.project.save")
        .expect("set action target"));
    assert!(session
        .select_binding_payload(1)
        .expect("select mode payload"));
    assert!(session
        .upsert_selected_binding_payload("mode", "\"compact\"")
        .expect("update payload"));
    assert!(session
        .upsert_selected_binding_payload("channel", "\"toolbar\"")
        .expect("add payload"));
    assert!(session
        .delete_selected_binding_payload()
        .expect("delete selected payload"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_event, "onDoubleClick");
    assert_eq!(updated.inspector_binding_event_selected_index, 1);
    assert_eq!(updated.inspector_binding_action_kind_selected_index, 2);
    assert_eq!(
        updated.inspector_binding_route,
        "editor_action.workbench.project.save"
    );
    assert_eq!(updated.inspector_binding_route_target, "");
    assert_eq!(
        updated.inspector_binding_action_target,
        "editor_action.workbench.project.save"
    );
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"compact\"".to_string()
        ]
    );

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings[0].event.to_string(), "onDoubleClick");
    assert!(button.bindings[0].route.is_none());
    let action = button.bindings[0].action.as_ref().expect("binding action");
    assert_eq!(
        action.action.as_deref(),
        Some("editor_action.workbench.project.save")
    );
    assert_eq!(
        action.payload.get("mode").and_then(toml::Value::as_str),
        Some("compact")
    );
    assert!(action.payload.get("channel").is_none());
}

#[test]
fn ui_asset_editor_session_projects_binding_payload_schema_suggestions_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "confirm = true".to_string(),
            "channel = \"toolbar\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(2)
        .expect("apply binding payload suggestion"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"full\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .select_binding_event_option(10)
        .expect("select scroll event"));
    let scroll = session.pane_presentation();
    assert_eq!(
        scroll.inspector_binding_payload_suggestion_items,
        vec![
            "axis = \"vertical\"".to_string(),
            "delta = 1".to_string(),
            "source = \"ui.scroll\"".to_string(),
        ]
    );
}
