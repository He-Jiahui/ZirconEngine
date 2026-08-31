use super::super::support::*;

#[test]
fn ui_asset_editor_session_inserts_palette_items_and_tracks_tree_edits_in_undo_stack() {
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
    let original_source = session.source_buffer().text().to_string();
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert button as child"));

    let inserted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert!(inserted.contains_node("button_2"));
    assert_eq!(
        inserted
            .node("button_2")
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );
    assert_eq!(
        inserted
            .node("button_2")
            .and_then(|node| node.props.get("text"))
            .and_then(toml::Value::as_str),
        Some("Button")
    );
    assert!(session.can_undo());
    assert_eq!(
        session.next_undo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::InsertPaletteItem)
    );
    assert_eq!(
        session.next_undo_tree_edit(),
        Some(UiAssetEditorTreeEdit::InsertPaletteItem {
            node_id: "button_2".to_string(),
            parent_node_id: Some("root".to_string()),
            palette_item_label: "Native / Button".to_string(),
            insert_mode: "child".to_string(),
        })
    );

    assert!(session.undo().expect("undo tree edit"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(session.can_redo());
    assert_eq!(
        session.next_redo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::InsertPaletteItem)
    );
    assert_eq!(
        session.next_redo_tree_edit(),
        Some(UiAssetEditorTreeEdit::InsertPaletteItem {
            node_id: "button_2".to_string(),
            parent_node_id: Some("root".to_string()),
            palette_item_label: "Native / Button".to_string(),
            insert_mode: "child".to_string(),
        })
    );

    assert!(session.redo().expect("redo tree edit"));
    let redone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert!(redone.contains_node("button_2"));
}

#[test]
fn ui_asset_editor_session_targets_palette_drag_drop_to_hovered_preview_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/simple-layout.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    session
        .select_hierarchy_index(1)
        .expect("select status label from hierarchy");
    assert_eq!(
        session.pane_presentation().inspector_selected_node_id,
        "status"
    );
    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));

    let root_frame = session
        .pane_presentation()
        .preview_canvas_items
        .into_iter()
        .find(|item| item.node_id == "root")
        .expect("root preview frame");
    assert!(session
        .update_palette_drag_target(
            root_frame.x + root_frame.width * 0.5,
            root_frame.y + root_frame.height * 0.5,
        )
        .expect("hover root preview frame"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_preview_index, 0);
    assert_eq!(targeted.palette_drag_target_action, "palette.insert.child");
    assert_eq!(targeted.palette_drag_target_label, "Insert Column Child");
    assert_eq!(targeted.inspector_selected_node_id, "status");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item at hovered target"));
    let inserted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_node_id = inserted
        .node("root")
        .and_then(|node| node.children.get(1))
        .map(|child| child.node.node_id.clone())
        .expect("inserted child");
    assert!(inserted.contains_node(&inserted_node_id));
    assert_eq!(
        inserted.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.as_str())
            .collect::<Vec<_>>()),
        Some(vec!["status", inserted_node_id.as_str()])
    );

    let dropped = session.pane_presentation();
    assert_eq!(dropped.inspector_selected_node_id, inserted_node_id);
    assert_eq!(dropped.palette_drag_target_preview_index, -1);
    assert!(dropped.palette_drag_target_action.is_empty());
}

#[test]
fn ui_asset_editor_session_projects_slot_aware_palette_drag_target_labels() {
    let scenarios = [
        (
            "asset://ui/tests/overlay-slot.zui",
            OVERLAY_SLOT_LAYOUT_ASSET_TOML,
            "Insert Overlay Child",
        ),
        (
            "asset://ui/tests/grid-slot.zui",
            GRID_SLOT_LAYOUT_ASSET_TOML,
            "Insert Grid Child",
        ),
        (
            "asset://ui/tests/flow-slot.zui",
            FLOW_SLOT_LAYOUT_ASSET_TOML,
            "Insert Flow Child",
        ),
        (
            "asset://ui/tests/scrollable-layout.zui",
            SCROLLABLE_LAYOUT_ASSET_TOML,
            "Insert Scroll Child",
        ),
    ];

    for (asset_id, source, expected_label) in scenarios {
        let route =
            UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Design);
        let mut session =
            UiAssetEditorSession::from_source(route, source, UiSize::new(640.0, 360.0))
                .expect("session");
        let palette_index = session
            .pane_presentation()
            .palette_items
            .iter()
            .position(|item| item == "Native / Button")
            .expect("button palette item");
        session
            .select_palette_index(palette_index)
            .expect("select palette item");

        let root_frame = session
            .pane_presentation()
            .preview_canvas_items
            .into_iter()
            .find(|item| item.node_id == "root")
            .expect("root preview frame");
        assert!(session
            .update_palette_drag_target(
                root_frame.x + root_frame.width * 0.5,
                root_frame.y + root_frame.height * 0.5,
            )
            .expect("hover root preview frame"));

        let presentation = session.pane_presentation();
        assert_eq!(
            presentation.palette_drag_target_action,
            "palette.insert.child"
        );
        assert_eq!(presentation.palette_drag_target_label, expected_label);
    }
}
