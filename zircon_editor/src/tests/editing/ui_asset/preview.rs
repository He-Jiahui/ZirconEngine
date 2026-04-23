use super::support::*;

#[test]
fn ui_asset_editor_session_compiles_preview_surface_and_projects_reflection_state() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route.clone(),
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    assert_eq!(session.route(), &route);
    assert_eq!(session.preview_host().surface().tree.nodes.len(), 2);
    assert!(session.diagnostics().is_empty());
    assert_eq!(
        session.reflection_model().selection,
        UiDesignerSelectionModel::single("root")
    );
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Ready")
    );
}

#[test]
fn ui_asset_editor_session_preserves_last_good_preview_when_source_turns_invalid() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );

    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("source edit should still update buffer");

    assert_eq!(session.source_buffer().text(), "not valid toml");
    assert!(!session.diagnostics().is_empty());
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
    assert!(session.reflection_model().preview_available);
    assert!(session.reflection_model().last_error.is_some());
}

#[test]
fn ui_asset_editor_session_undo_and_redo_source_edits_restore_preview_state() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Source,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("invalid edit still tracked");

    assert!(session.can_undo());
    assert!(!session.can_redo());

    assert!(session.undo().expect("undo invalid edit"));
    assert_eq!(session.source_buffer().text(), edited);
    assert!(session.diagnostics().is_empty());
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );

    assert!(session.undo().expect("undo valid edit"));
    assert_eq!(session.source_buffer().text(), SIMPLE_LAYOUT_ASSET_TOML);
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Ready")
    );
    assert!(session.can_redo());

    assert!(session.redo().expect("redo valid edit"));
    assert_eq!(session.source_buffer().text(), edited);
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
}

#[test]
fn ui_asset_editor_session_switches_modes_and_updates_selection_from_hierarchy() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/styled-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLED_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .set_mode(UiAssetEditorMode::Preview)
        .expect("preview mode");
    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let reflection = session.reflection_model();
    assert_eq!(reflection.route.mode, UiAssetEditorMode::Preview);
    assert_eq!(
        reflection.selection.primary_node_id.as_deref(),
        Some("button")
    );
    assert_eq!(
        reflection.style_inspector.selected_node_id.as_deref(),
        Some("button")
    );
    assert_eq!(
        reflection.style_inspector.classes,
        vec!["primary".to_string()]
    );
    assert_eq!(reflection.style_inspector.matched_rules.len(), 2);
    assert_eq!(
        reflection.style_inspector.matched_rules[0].selector,
        ".primary".to_string()
    );
    assert_eq!(
        reflection.style_inspector.matched_rules[1].selector,
        "VerticalBox > Button.primary".to_string()
    );
}

#[test]
fn ui_asset_editor_session_switches_preview_presets_and_rebuilds_preview_surface() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("session");

    assert_eq!(
        session.reflection_model().route.preview_preset,
        UiAssetPreviewPreset::EditorDocked
    );
    assert_eq!(
        session.preview_host().preview_size(),
        UiSize::new(1280.0, 720.0)
    );
    assert!(session
        .pane_presentation()
        .preview_summary
        .contains("1280x720"));

    assert!(session
        .set_preview_preset(UiAssetPreviewPreset::GameHud)
        .expect("set game hud preview preset"));
    assert_eq!(
        session.reflection_model().route.preview_preset,
        UiAssetPreviewPreset::GameHud
    );
    assert_eq!(
        session.preview_host().preview_size(),
        UiSize::new(1920.0, 1080.0)
    );
    assert!(session
        .pane_presentation()
        .preview_summary
        .contains("1920x1080"));
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Ready")
    );

    assert!(session
        .set_preview_preset(UiAssetPreviewPreset::Dialog)
        .expect("set dialog preview preset"));
    assert_eq!(
        session.reflection_model().route.preview_preset,
        UiAssetPreviewPreset::Dialog
    );
    assert_eq!(
        session.preview_host().preview_size(),
        UiSize::new(640.0, 480.0)
    );
    assert!(session
        .pane_presentation()
        .preview_summary
        .contains("640x480"));
    assert!(!session
        .set_preview_preset(UiAssetPreviewPreset::Dialog)
        .expect("same preset should no-op"));
}

#[test]
fn ui_asset_editor_session_applies_editor_only_mock_preview_values_without_dirtying_source() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/mock-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        MOCK_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let original_source = session.source_buffer().text().to_string();

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .select_preview_mock_property(0)
        .expect("select preview mock property");
    assert!(session
        .set_selected_preview_mock_value("Preview Save")
        .expect("set preview mock value"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_selected_index, 0);
    assert_eq!(updated.preview_mock_property, "text");
    assert_eq!(updated.preview_mock_kind, "Text");
    assert_eq!(updated.preview_mock_value, "Preview Save");
    assert!(updated.preview_mock_can_clear);
    assert_eq!(
        selected_text(session.preview_host().surface(), "SaveButton"),
        Some("Preview Save")
    );
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.reflection_model().source_dirty);

    assert!(session
        .clear_selected_preview_mock_value()
        .expect("clear preview mock value"));
    let cleared = session.pane_presentation();
    assert_eq!(cleared.preview_mock_value, "Save");
    assert!(!cleared.preview_mock_can_clear);
    assert_eq!(
        selected_text(session.preview_host().surface(), "SaveButton"),
        Some("Save")
    );
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.reflection_model().source_dirty);
}

#[test]
fn ui_asset_editor_session_projects_mock_preview_property_kinds_for_selected_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/mock-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        MOCK_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.preview_mock_items,
        vec![
            "text [Text] = Save".to_string(),
            "checked [Bool] = false".to_string(),
            "mode [Enum] = Full".to_string(),
            "icon [Resource] = asset://ui/icons/save.png".to_string(),
        ]
    );
    assert_eq!(pane.preview_mock_selected_index, 0);
    assert_eq!(pane.preview_mock_property, "text");
    assert_eq!(pane.preview_mock_kind, "Text");
    assert_eq!(pane.preview_mock_value, "Save");
}

#[test]
fn ui_asset_editor_session_supports_cross_node_preview_mock_subjects_without_changing_selection() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/cross-node-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        CROSS_NODE_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let original_source = session.source_buffer().text().to_string();

    session
        .select_hierarchy_index(2)
        .expect("select button from hierarchy");
    assert_eq!(
        session
            .reflection_model()
            .selection
            .primary_node_id
            .as_deref(),
        Some("button")
    );

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select preview mock subject node"));
    let subject_pane = session.pane_presentation();
    assert_eq!(
        subject_pane.preview_mock_items,
        vec!["StatusLabel.text [Text] = Ready".to_string()]
    );
    assert_eq!(subject_pane.preview_mock_selected_index, 0);
    assert_eq!(subject_pane.preview_mock_property, "StatusLabel.text");
    assert_eq!(subject_pane.preview_mock_value, "Ready");

    session
        .select_preview_mock_property(0)
        .expect("select status text preview property");
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status preview text"));
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Dirty")
    );
    assert_eq!(
        selected_text(session.preview_host().surface(), "SaveButton"),
        Some("Save")
    );
    assert_eq!(
        session
            .reflection_model()
            .selection
            .primary_node_id
            .as_deref(),
        Some("button")
    );
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.reflection_model().source_dirty);
}

#[test]
fn ui_asset_editor_session_projects_rich_preview_mock_kinds_and_state_graph() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/rich-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        RICH_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(2)
        .expect("select button from hierarchy");

    let initial = session.pane_presentation();
    assert!(initial
        .preview_mock_items
        .contains(&"text [Text] = Save".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"checked [Bool] = false".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"count [Number] = 3".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"mode [Enum] = Full".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"icon [Resource] = asset://ui/icons/save.png".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"items [Collection] = [\"Save\", \"Publish\"]".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"metadata [Object] = { enabled = true, state = \"Ready\" }".to_string()));
    assert!(initial
        .preview_mock_items
        .contains(&"text_expr [Expression] = =preview.save_label".to_string()));
    assert!(initial.preview_state_graph_items.is_empty());

    let items_index = initial
        .preview_mock_items
        .iter()
        .position(|item| item.starts_with("items [Collection]"))
        .expect("items preview entry");
    session
        .select_preview_mock_property(items_index)
        .expect("select collection preview property");
    assert!(session
        .set_selected_preview_mock_value("[\"Save\", \"Ship\"]")
        .expect("set collection preview override"));

    let graph_after_items = session.pane_presentation();
    assert_eq!(graph_after_items.preview_mock_kind, "Collection");
    assert_eq!(graph_after_items.preview_mock_value, "[\"Save\", \"Ship\"]");
    assert_eq!(
        graph_after_items.preview_state_graph_items,
        vec!["SaveButton.items = [\"Save\", \"Ship\"]".to_string()]
    );

    session
        .select_preview_mock_subject_node("status")
        .expect("select status node as preview subject");
    let status_pane = session.pane_presentation();
    let status_index = status_pane
        .preview_mock_items
        .iter()
        .position(|item| item.starts_with("StatusLabel.text [Text]"))
        .expect("status preview entry");
    session
        .select_preview_mock_property(status_index)
        .expect("select status text preview property");
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("set status preview override"));

    let graph_with_status = session.pane_presentation();
    assert_eq!(
        graph_with_status.preview_state_graph_items,
        vec![
            "SaveButton.items = [\"Save\", \"Ship\"]".to_string(),
            "StatusLabel.text = Dirty".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_edits_preview_mock_collection_entries_structurally() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/rich-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        RICH_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(2)
        .expect("select button from hierarchy");
    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.starts_with("items [Collection]"))
        .expect("items preview entry");
    session
        .select_preview_mock_property(items_index)
        .expect("select collection preview property");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_nested_items,
        vec![
            "[0] [Text] = Save".to_string(),
            "[1] [Text] = Publish".to_string(),
        ]
    );
    assert_eq!(initial.preview_mock_nested_selected_index, 0);
    assert_eq!(initial.preview_mock_nested_key, "0");
    assert_eq!(initial.preview_mock_nested_kind, "Text");
    assert_eq!(initial.preview_mock_nested_value, "Save");

    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select collection nested entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Ship")
        .expect("set selected collection entry"));
    assert!(session
        .upsert_selected_preview_mock_nested_entry("2", "\"Archive\"")
        .expect("append collection entry"));
    assert!(session
        .select_preview_mock_nested_entry(0)
        .expect("reselect collection entry"));
    assert!(session
        .delete_selected_preview_mock_nested_entry()
        .expect("delete selected collection entry"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_value, "[\"Ship\", \"Archive\"]");
    assert_eq!(
        updated.preview_mock_nested_items,
        vec![
            "[0] [Text] = Ship".to_string(),
            "[1] [Text] = Archive".to_string(),
        ]
    );
    assert_eq!(
        updated.preview_state_graph_items,
        vec!["SaveButton.items = [\"Ship\", \"Archive\"]".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_edits_preview_mock_object_entries_structurally() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/rich-preview.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        RICH_PREVIEW_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(2)
        .expect("select button from hierarchy");
    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.starts_with("metadata [Object]"))
        .expect("metadata preview entry");
    session
        .select_preview_mock_property(metadata_index)
        .expect("select object preview property");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_nested_items,
        vec![
            "enabled [Bool] = true".to_string(),
            "state [Text] = Ready".to_string(),
        ]
    );

    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select object nested entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Dirty")
        .expect("set selected object entry"));
    assert!(session
        .upsert_selected_preview_mock_nested_entry("count", "4")
        .expect("add object nested entry"));
    assert!(session
        .select_preview_mock_nested_entry(0)
        .expect("reselect object nested entry"));
    assert!(session
        .delete_selected_preview_mock_nested_entry()
        .expect("delete object nested entry"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.preview_mock_nested_items,
        vec![
            "count [Number] = 4".to_string(),
            "state [Text] = Dirty".to_string(),
        ]
    );
    assert!(updated.preview_mock_value.contains("count = 4"));
    assert!(updated.preview_mock_value.contains("state = \"Dirty\""));
    assert!(!updated.preview_mock_value.contains("enabled"));
    assert_eq!(
        updated.preview_state_graph_items,
        vec!["SaveButton.metadata = { count = 4, state = \"Dirty\" }".to_string()]
    );
}
