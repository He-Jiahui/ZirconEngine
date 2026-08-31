use super::*;

#[test]
fn ui_asset_editor_session_projects_preview_mock_subjects_and_expression_results() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_subject_items,
        vec![
            "SaveButton • button".to_string(),
            "StatusLabel • status".to_string(),
        ]
    );
    assert_eq!(initial.preview_mock_subject_selected_index, 0);

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    assert!(session
        .select_preview_mock_property(0)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button preview subject"));
    let button_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text_expr"))
        .expect("expression property");
    assert!(session
        .select_preview_mock_property(button_index)
        .expect("select expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Dirty");
}

#[test]
fn ui_asset_editor_session_projects_binding_target_suggestions_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    let initial = session.pane_presentation();
    assert!(initial
        .inspector_binding_route_suggestion_items
        .iter()
        .any(|item| item.contains("menu_action.workbench.project.save")));

    assert!(session
        .apply_selected_binding_route_suggestion(1)
        .expect("apply route suggestion"));
    let route_applied = session.pane_presentation();
    assert_ne!(
        route_applied.inspector_binding_route_target,
        "menu_action.workbench.project.save"
    );

    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    let action_suggestions = session.pane_presentation();
    assert!(action_suggestions
        .inspector_binding_action_suggestion_items
        .iter()
        .any(|item| item.contains("editor_action.workbench.project.save")));

    assert!(session
        .apply_selected_binding_action_suggestion(0)
        .expect("apply action suggestion"));
    let action_applied = session.pane_presentation();
    assert_eq!(
        action_applied.inspector_binding_action_target,
        "editor_action.workbench.project.save"
    );
}

#[test]
fn ui_asset_editor_session_projects_expression_dependencies_into_preview_state_graph() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_state_graph.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview state graph session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata [Object]"))
        .expect("metadata property");
    assert!(session
        .select_preview_mock_property(metadata_index)
        .expect("select metadata property"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select title nested entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Dirty")
        .expect("override metadata title"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let expression_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text_expr"))
        .expect("text expression property");
    assert!(session
        .select_preview_mock_property(expression_index)
        .expect("select expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_expression_result, "Dirty");
    assert!(updated
        .preview_state_graph_items
        .contains(&"StatusLabel.metadata = { count = 1, title = \"Dirty\" }".to_string()));
    assert!(updated
        .preview_state_graph_items
        .contains(&"SaveButton.text_expr -> StatusLabel.metadata.title = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_preview_mock_schema_items_for_object_and_collection_values() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_state_graph.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview state graph session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));

    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata [Object]"))
        .expect("metadata property");
    assert!(session
        .select_preview_mock_property(metadata_index)
        .expect("select metadata property"));
    let object_schema = session.pane_presentation();
    assert_eq!(
        object_schema.preview_mock_schema_items,
        vec![
            "StatusLabel.metadata.count [Number]".to_string(),
            "StatusLabel.metadata.title [Text]".to_string(),
        ]
    );

    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    let collection_schema = session.pane_presentation();
    assert_eq!(
        collection_schema.preview_mock_schema_items,
        vec![
            "StatusLabel.items[0] [Text]".to_string(),
            "StatusLabel.items[1] [Text]".to_string(),
            "StatusLabel.items[n] [Text]".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_binding_schema_items_for_route_and_action_targets() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    let route_schema = session.pane_presentation();
    assert_eq!(
        route_schema.inspector_binding_schema_items,
        vec![
            "event [UiEvent] = onClick".to_string(),
            "route.target [Route] = menu_action.workbench.project.save".to_string(),
            "payload.confirm [Bool] default = true".to_string(),
            "payload.channel [Text] default = \"toolbar\"".to_string(),
            "payload.source [Text] default = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    assert!(session
        .set_selected_binding_action_target("editor_action.workbench.project.save")
        .expect("set action target"));
    let action_schema = session.pane_presentation();
    assert_eq!(
        action_schema.inspector_binding_schema_items,
        vec![
            "event [UiEvent] = onClick".to_string(),
            "action.target [EditorAction] = editor_action.workbench.project.save".to_string(),
            "payload.confirm [Bool] default = true".to_string(),
            "payload.source [Text] default = \"ui.click\"".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_evaluates_preview_mock_bracket_expression_paths() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_bracket_expression.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_BRACKET_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview bracket expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select second collection entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Shipped")
        .expect("override second collection entry"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let expression_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("item_expr"))
        .expect("item expression property");
    assert!(session
        .select_preview_mock_property(expression_index)
        .expect("select item expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Shipped");
    assert!(updated
        .preview_state_graph_items
        .contains(&"StatusLabel.items = [\"Ready\", \"Shipped\"]".to_string()));
    assert!(updated
        .preview_state_graph_items
        .contains(&"SaveButton.item_expr -> StatusLabel.items[1] = \"Shipped\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_target_aware_structured_binding_payload_schemas() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    assert!(session
        .select_binding_event_option(5)
        .expect("select change event"));
    assert!(session
        .apply_selected_binding_route_suggestion(0)
        .expect("apply selection changed route"));
    let route_payload = session.pane_presentation();
    assert_eq!(
        route_payload.inspector_binding_payload_suggestion_items,
        vec![
            "primary = \"SelectedNode\"".to_string(),
            "selection_ids = [\"SelectedNode\"]".to_string(),
            "context = { additive = false, source = \"hierarchy\" }".to_string(),
        ]
    );
    assert!(route_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids [Collection] default = [\"SelectedNode\"]".to_string()));
    assert!(route_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids[n] [Text] default = \"SelectedNode\"".to_string()));
    assert!(route_payload.inspector_binding_schema_items.contains(
        &"payload.context [Object] default = { additive = false, source = \"hierarchy\" }"
            .to_string()
    ));

    assert!(session
        .select_binding_event_option(7)
        .expect("select toggle event"));
    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    assert!(session
        .set_selected_binding_action_target("editor_action.workbench.visibility.toggle")
        .expect("set toggle visibility action"));
    let action_payload = session.pane_presentation();
    assert_eq!(
        action_payload.inspector_binding_payload_suggestion_items,
        vec![
            "checked = true".to_string(),
            "selection_ids = [\"SelectedNode\"]".to_string(),
            "context = { scope = \"selection\", source = \"ui.toggle\" }".to_string(),
        ]
    );
    assert!(action_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids [Collection] default = [\"SelectedNode\"]".to_string()));
    assert!(action_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids[n] [Text] default = \"SelectedNode\"".to_string()));
    assert!(action_payload.inspector_binding_schema_items.contains(
        &"payload.context [Object] default = { scope = \"selection\", source = \"ui.toggle\" }"
            .to_string()
    ));
}

#[test]
fn ui_asset_editor_session_projects_binding_expression_payload_previews_and_interaction_edges() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("status_text", "=StatusLabel.text")
        .expect("upsert binding expression payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", dirty = true }",
        )
        .expect("upsert binding object payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.status_text [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.status_text.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.preview [Object] = { dirty = true, title = \"Dirty\" }".to_string()
    ));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.onClick => menu_action.workbench.project.save".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.status_text -> StatusLabel.text = \"Dirty\"".to_string()
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.title -> StatusLabel.text = \"Dirty\"".to_string()
    ));
}
