use super::*;

#[test]
fn ui_asset_editor_session_evaluates_function_preview_expressions_and_binding_payload_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let summary_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("summary_expr"))
        .expect("summary expression property");
    assert!(session
        .select_preview_mock_property(summary_index)
        .expect("select summary expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty / Save"
    );

    let fallback_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("fallback_expr"))
        .expect("fallback expression property");
    assert!(session
        .select_preview_mock_property(fallback_index)
        .expect("select fallback expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty"
    );

    let count_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("item_count_expr"))
        .expect("count expression property");
    assert!(session
        .select_preview_mock_property(count_index)
        .expect("select count expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "2"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload(
            "summary",
            r#"'=concat(StatusLabel.text, " / ", self.text)'"#,
        )
        .expect("upsert concat binding payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "fallback",
            r#"'=coalesce(StatusLabel.subtitle, StatusLabel.text, "Unknown")'"#,
        )
        .expect("upsert coalesce binding payload"));
    assert!(session
        .upsert_selected_binding_payload("item_count", r#"'=count(StatusLabel.items)'"#)
        .expect("upsert count binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.summary_expr -> StatusLabel.text = \"Dirty\"".to_string()));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.summary_expr -> SaveButton.text = \"Save\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.summary.preview [Text] = \"Dirty / Save\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.fallback.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.item_count.preview [Number] = 2".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.summary -> StatusLabel.text = \"Dirty\"".to_string()
    ));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.onClick.payload.summary -> SaveButton.text = \"Save\"".to_string()));
}

#[test]
fn ui_asset_editor_session_evaluates_collection_and_branch_preview_expressions() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

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
        .set_selected_preview_mock_nested_value("Queued")
        .expect("override first item"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select second item"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("override second item"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));

    let first_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("first_item_expr"))
        .expect("first item expression property");
    assert!(session
        .select_preview_mock_property(first_index)
        .expect("select first item expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Queued"
    );

    let last_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("last_item_expr"))
        .expect("last item expression property");
    assert!(session
        .select_preview_mock_property(last_index)
        .expect("select last item expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Reviewed"
    );

    let joined_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("joined_items_expr"))
        .expect("joined items expression property");
    assert!(session
        .select_preview_mock_property(joined_index)
        .expect("select joined items expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Queued | Reviewed"
    );

    let eq_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("status_matches_expr"))
        .expect("status matches expression property");
    assert!(session
        .select_preview_mock_property(eq_index)
        .expect("select status matches expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "true"
    );

    let if_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("cta_expr"))
        .expect("cta expression property");
    assert!(session
        .select_preview_mock_property(if_index)
        .expect("select cta expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Go"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload("first_item", r#"'=first(StatusLabel.items)'"#)
        .expect("upsert first item binding payload"));
    assert!(session
        .upsert_selected_binding_payload("joined_items", r#"'=join(StatusLabel.items, " | ")'"#,)
        .expect("upsert joined items binding payload"));
    assert!(session
        .upsert_selected_binding_payload("is_dirty", r#"'=eq(StatusLabel.text, "Dirty")'"#)
        .expect("upsert eq binding payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "cta",
            r#"'=if(eq(StatusLabel.text, "Dirty"), "Go", "Stop")'"#,
        )
        .expect("upsert if binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.first_item.preview [Text] = \"Queued\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.joined_items.preview [Text] = \"Queued | Reviewed\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.is_dirty.preview [Bool] = true".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.cta.preview [Text] = \"Go\"".to_string()));
}

#[test]
fn ui_asset_editor_session_evaluates_accessor_preview_expressions_and_binding_payload_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

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
    let title_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("title [Text]"))
        .expect("metadata title");
    assert!(session
        .select_preview_mock_nested_entry(title_index)
        .expect("select metadata title"));
    assert!(session
        .set_selected_preview_mock_nested_value("Dirty")
        .expect("override metadata title"));

    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    let review_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("[1] [Text]"))
        .expect("second item");
    assert!(session
        .select_preview_mock_nested_entry(review_index)
        .expect("select second item"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("override second item"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));

    let metadata_title_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata_title_expr"))
        .expect("metadata title expression");
    assert!(session
        .select_preview_mock_property(metadata_title_index)
        .expect("select metadata title expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty"
    );

    let review_item_expr_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("review_item_expr"))
        .expect("review item expression");
    assert!(session
        .select_preview_mock_property(review_item_expr_index)
        .expect("select review item expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Reviewed"
    );

    let has_title_expr_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("has_title_expr"))
        .expect("has title expression");
    assert!(session
        .select_preview_mock_property(has_title_expr_index)
        .expect("select has title expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "true"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload(
            "metadata_title",
            r#"'=get(StatusLabel.metadata, "title")'"#,
        )
        .expect("upsert metadata title binding payload"));
    assert!(session
        .upsert_selected_binding_payload("review_item", r#"'=at(StatusLabel.items, 1)'"#)
        .expect("upsert review item binding payload"));
    assert!(session
        .upsert_selected_binding_payload("has_title", r#"'=has(StatusLabel.metadata, "title")'"#)
        .expect("upsert has title binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.metadata_title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.review_item.preview [Text] = \"Reviewed\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.has_title.preview [Bool] = true".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.metadata_title_expr -> StatusLabel.metadata.title = \"Dirty\"".to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.review_item_expr -> StatusLabel.items[1] = \"Reviewed\"".to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.metadata_title -> StatusLabel.metadata.title = \"Dirty\""
            .to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.review_item -> StatusLabel.items[1] = \"Reviewed\""
            .to_string(),
    ));
}
