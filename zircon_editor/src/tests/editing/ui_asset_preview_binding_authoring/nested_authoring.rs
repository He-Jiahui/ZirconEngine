use super::*;

#[test]
fn ui_asset_editor_session_projects_nested_binding_payload_schema_previews() {
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
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", steps = [\"Idle\", \"=StatusLabel.text\"] }",
        )
        .expect("upsert nested binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context [Object] = { steps = [\"Idle\", \"=StatusLabel.text\"], title = \"=StatusLabel.text\" }"
                .to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.steps [Collection] = [\"Idle\", \"=StatusLabel.text\"]".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[0] [Text] = \"Idle\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1] [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1].preview [Text] = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_recursive_preview_mock_paths_and_nested_expression_results() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));
    let initial = session.pane_presentation();
    assert!(initial
        .preview_mock_schema_items
        .contains(&"StatusLabel.context.dialog.title [Text]".to_string()));
    assert!(initial
        .preview_mock_schema_items
        .contains(&"StatusLabel.context.dialog.steps[1].label [Text]".to_string()));
    let nested_index = initial
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog.steps[1].label"))
        .expect("deep nested preview entry");
    assert!(session
        .select_preview_mock_nested_entry(nested_index)
        .expect("select deep nested preview entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("set deep nested preview entry"));

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
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Reviewed");
    assert!(updated
        .preview_state_graph_items
        .contains(
            &"StatusLabel.context = { dialog = { steps = [{ label = \"Plan\" }, { label = \"Reviewed\" }], title = \"Ready\" } }"
                .to_string()
        ));
    assert!(updated.preview_state_graph_items.contains(
        &"SaveButton.text_expr -> StatusLabel.context.dialog.steps[1].label = \"Reviewed\""
            .to_string()
    ));
}

#[test]
fn ui_asset_editor_session_projects_preview_mock_suggestions_relative_to_selected_nested_container_and_applies_them(
) {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    let dialog_index = initial
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog [Object]"))
        .expect("dialog nested entry");
    assert!(session
        .select_preview_mock_nested_entry(dialog_index)
        .expect("select dialog nested entry"));
    let dialog_scope = session.pane_presentation();
    assert_eq!(
        dialog_scope.preview_mock_suggestion_items,
        vec![
            "steps = [{ label = \"Plan\" }, { label = \"Dirty\" }]".to_string(),
            "title = \"Ready\"".to_string(),
        ]
    );

    let steps_index = dialog_scope
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog.steps [Collection]"))
        .expect("dialog.steps nested entry");
    assert!(session
        .select_preview_mock_nested_entry(steps_index)
        .expect("select dialog.steps nested entry"));
    let steps_scope = session.pane_presentation();
    assert_eq!(
        steps_scope.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    assert!(session
        .apply_selected_preview_mock_suggestion(2)
        .expect("apply append suggestion"));
    let updated = session.pane_presentation();
    assert!(updated
        .preview_mock_nested_items
        .iter()
        .any(|item| item.contains("dialog.steps[2] [Object] = { label = \"Plan\" }")));
    assert_eq!(updated.preview_mock_nested_key, "dialog.steps[2]");
    assert_eq!(updated.preview_mock_nested_value, "{ label = \"Plan\" }");
    assert_eq!(
        updated.preview_mock_suggestion_items,
        vec!["label = \"Plan\"".to_string()]
    );
    assert!(updated
        .preview_state_graph_items
        .contains(
            &"StatusLabel.context = { dialog = { steps = [{ label = \"Plan\" }, { label = \"Dirty\" }, { label = \"Plan\" }], title = \"Ready\" } }"
                .to_string()
        ));
}

#[test]
fn ui_asset_editor_session_selects_preview_mock_schema_items_as_nested_authoring_targets() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));

    let schema_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("StatusLabel.context.dialog.steps[1].label"))
        .expect("deep nested preview entry");
    assert!(session
        .select_preview_mock_nested_entry(schema_index)
        .expect("select preview nested entry"));

    let selected = session.pane_presentation();
    assert_eq!(
        selected.preview_mock_nested_key,
        "dialog.steps[1].label".to_string()
    );
    assert_eq!(selected.preview_mock_nested_value, "Dirty".to_string());
}

#[test]
fn ui_asset_editor_session_selects_binding_schema_items_as_payload_authoring_targets() {
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
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", steps = [\"Idle\", \"=StatusLabel.text\"] }",
        )
        .expect("upsert nested binding payload"));

    let schema_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps[1] = \"=StatusLabel.text\""))
        .expect("binding payload expression item");
    assert!(session
        .select_binding_payload(schema_index)
        .expect("select binding payload item"));

    let selected = session.pane_presentation();
    assert_eq!(
        selected.inspector_binding_payload_key,
        "context.steps[1]".to_string()
    );
    assert_eq!(
        selected.inspector_binding_payload_value,
        "\"=StatusLabel.text\"".to_string()
    );
}

#[test]
fn ui_asset_editor_session_supports_recursive_binding_payload_paths_and_previews() {
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
        .upsert_selected_binding_payload("context.dialog.title", "=StatusLabel.text")
        .expect("upsert nested dialog title payload"));
    assert!(session
        .upsert_selected_binding_payload("context.steps[0].label", "\"Queued\"")
        .expect("upsert queued step payload"));
    assert!(session
        .upsert_selected_binding_payload("context.steps[1].label", "=StatusLabel.text")
        .expect("upsert nested step payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\"")));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.dialog.title [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.dialog.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1].label.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.dialog.title -> StatusLabel.text = \"Dirty\""
            .to_string()
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.steps[1].label -> StatusLabel.text = \"Dirty\""
            .to_string()
    ));

    let delete_index = pane
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\""))
        .expect("nested payload item");
    session
        .select_binding_payload(delete_index)
        .expect("select nested payload item");
    assert!(session
        .delete_selected_binding_payload()
        .expect("delete nested payload item"));

    let updated = session.pane_presentation();
    assert!(updated
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(!updated
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\"")));
}
