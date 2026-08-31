use super::*;

#[test]
fn ui_asset_editor_session_upserts_binding_payload_entries_relative_to_selected_container() {
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
        .upsert_selected_binding_payload("context", "{ title = \"=StatusLabel.text\" }")
        .expect("upsert root context payload"));
    assert!(session
        .upsert_selected_binding_payload("subtitle", "\"Preview\"")
        .expect("upsert context subtitle payload"));
    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| {
            item.contains("context = { subtitle = \"Preview\", title = \"=StatusLabel.text\" }")
        })
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("reselect context payload");
    assert!(session
        .upsert_selected_binding_payload("steps", "[\"Plan\"]")
        .expect("upsert context steps payload"));

    let context_steps_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps = [\"Plan\"]"))
        .expect("context steps payload");
    session
        .select_binding_payload(context_steps_index)
        .expect("select context steps payload");
    assert!(session
        .upsert_selected_binding_payload("", "\"Review\"")
        .expect("append selected collection payload entry"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context = { steps = [\"Plan\", \"Review\"], subtitle = \"Preview\", title = \"=StatusLabel.text\" }")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.subtitle = \"Preview\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1] = \"Review\"")));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.subtitle [Text] = \"Preview\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1] [Text] = \"Review\"".to_string()));
}

#[test]
fn ui_asset_editor_session_upserts_binding_payload_nested_relative_paths_from_selected_container() {
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
        .upsert_selected_binding_payload("context", "{}")
        .expect("upsert root context payload"));

    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("select context payload");
    assert!(session
        .upsert_selected_binding_payload("dialog.title", "=StatusLabel.text")
        .expect("upsert nested relative object payload"));

    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = { dialog = { title = \"=StatusLabel.text\" } }"))
        .expect("updated context payload");
    session
        .select_binding_payload(context_index)
        .expect("reselect updated context payload");
    assert!(session
        .upsert_selected_binding_payload("steps", "[]")
        .expect("upsert steps collection payload"));

    let steps_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps = []"))
        .expect("steps payload");
    session
        .select_binding_payload(steps_index)
        .expect("select steps payload");
    assert!(session
        .upsert_selected_binding_payload("[0].label", "=StatusLabel.text")
        .expect("upsert indexed relative collection payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[0].label = \"=StatusLabel.text\"")));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.dialog.title [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.dialog.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.steps[0].label [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[0].label.preview [Text] = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_binding_payload_suggestions_relative_to_selected_container() {
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

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "value = \"preview\"".to_string(),
            "committed = true".to_string(),
            "fields = [\"title\"]".to_string(),
            "context = { source = \"ui.click\", subject = \"field\" }".to_string(),
        ]
    );

    assert!(session
        .upsert_selected_binding_payload("context", "{}")
        .expect("upsert context payload"));
    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("select context payload");

    let context_pane = session.pane_presentation();
    assert_eq!(
        context_pane.inspector_binding_payload_suggestion_items,
        vec![
            "source = \"ui.click\"".to_string(),
            "subject = \"field\"".to_string(),
        ]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(1)
        .expect("apply contextual object suggestion"));
    let after_object = session.pane_presentation();
    assert!(after_object
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.subject = \"field\"")));
    assert_eq!(
        after_object.inspector_binding_payload_key,
        "context.subject".to_string()
    );

    assert!(session
        .upsert_selected_binding_payload("fields", "[]")
        .expect("upsert fields payload"));
    let fields_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("fields = []"))
        .expect("fields payload");
    session
        .select_binding_payload(fields_index)
        .expect("select fields payload");

    let fields_pane = session.pane_presentation();
    assert_eq!(
        fields_pane.inspector_binding_payload_suggestion_items,
        vec!["[0] = \"title\"".to_string(), "[1] = \"title\"".to_string()]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(0)
        .expect("apply contextual collection suggestion"));
    let after_collection = session.pane_presentation();
    assert!(after_collection
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("fields[0] = \"title\"")));
    assert_eq!(
        after_collection.inspector_binding_payload_key,
        "fields[0]".to_string()
    );
}

#[test]
fn ui_asset_editor_session_projects_collection_template_schema_paths_for_nested_binding_payloads() {
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

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ steps = [{ label = \"=StatusLabel.context.dialog.steps[0].label\" }] }"
        )
        .expect("upsert nested collection payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps [Collection] = [{ label = \"=StatusLabel.context.dialog.steps[0].label\" }]".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps[n] [Object] = { label = \"=StatusLabel.context.dialog.steps[0].label\" }".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps[n].label [Expression] = \"=StatusLabel.context.dialog.steps[0].label\"".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[n].label.preview [Text] = \"Plan\"".to_string()));
}
