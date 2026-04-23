use super::support::*;

#[test]
fn ui_asset_editor_session_inserts_palette_items_and_tracks_tree_edits_in_undo_stack() {
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
fn ui_asset_editor_session_targets_palette_drag_drop_to_hovered_preview_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/simple-layout.ui.toml",
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
fn ui_asset_editor_session_projects_slot_aware_palette_drag_target_labels() {
    let scenarios = [
        (
            "asset://ui/tests/overlay-slot.ui.toml",
            OVERLAY_SLOT_LAYOUT_ASSET_TOML,
            "Insert Overlay Child",
        ),
        (
            "asset://ui/tests/grid-slot.ui.toml",
            GRID_SLOT_LAYOUT_ASSET_TOML,
            "Insert Grid Child",
        ),
        (
            "asset://ui/tests/flow-slot.ui.toml",
            FLOW_SLOT_LAYOUT_ASSET_TOML,
            "Insert Flow Child",
        ),
        (
            "asset://ui/tests/scrollable-layout.ui.toml",
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
fn ui_asset_editor_undo_stack_replays_document_diffs_for_tree_edits() {
    let mut undo_stack = UiAssetEditorUndoStack::default();
    let before_document =
        crate::tests::support::load_test_ui_asset(SIMPLE_LAYOUT_ASSET_TOML).expect("before");
    let after_document =
        crate::tests::support::load_test_ui_asset(STYLE_AUTHORING_LAYOUT_ASSET_TOML)
            .expect("after");

    undo_stack.push_edit(
        "Insert Palette Item",
        Some(UiAssetEditorTreeEdit::InsertPaletteItem {
            node_id: "button_2".to_string(),
            parent_node_id: Some("root".to_string()),
            palette_item_label: "Native / Button".to_string(),
            insert_mode: "child".to_string(),
        }),
        None,
        SIMPLE_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        Some(before_document.clone()),
        STYLE_AUTHORING_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        Some(after_document.clone()),
        UiAssetEditorUndoExternalEffects::default(),
    );

    let undone = undo_stack.undo().expect("undo replay");
    let mut undone_source = STYLE_AUTHORING_LAYOUT_ASSET_TOML.to_string();
    assert!(undone
        .apply_to_source(&mut undone_source)
        .expect("apply undo source diff"));
    assert_eq!(undone_source, SIMPLE_LAYOUT_ASSET_TOML);
    let mut undone_document = after_document.clone();
    assert!(undone
        .apply_to_document(&mut undone_document)
        .expect("apply undo diff"));
    assert_eq!(undone_document, before_document);

    let redone = undo_stack.redo().expect("redo replay");
    let mut redone_source = SIMPLE_LAYOUT_ASSET_TOML.to_string();
    assert!(redone
        .apply_to_source(&mut redone_source)
        .expect("apply redo source diff"));
    assert_eq!(redone_source, STYLE_AUTHORING_LAYOUT_ASSET_TOML);
    let mut redone_document = before_document.clone();
    assert!(redone
        .apply_to_document(&mut redone_document)
        .expect("apply redo diff"));
    assert_eq!(redone_document, after_document);
}

#[test]
fn ui_asset_editor_undo_stack_tracks_inverse_tree_edits_for_command_log_entries() {
    let mut undo_stack = UiAssetEditorUndoStack::default();
    undo_stack.push_edit(
        "Move Node",
        Some(UiAssetEditorTreeEdit::MoveNode {
            node_id: "button".to_string(),
            direction: "Down".to_string(),
        }),
        None,
        SIMPLE_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::single("button"),
        Default::default(),
        None,
        Some(crate::tests::support::load_test_ui_asset(SIMPLE_LAYOUT_ASSET_TOML).expect("before")),
        STYLED_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::single("button"),
        Default::default(),
        None,
        Some(crate::tests::support::load_test_ui_asset(STYLED_LAYOUT_ASSET_TOML).expect("after")),
        UiAssetEditorUndoExternalEffects::default(),
    );

    assert_eq!(undo_stack.next_undo_label().as_deref(), Some("Move Node"));
    assert_eq!(
        undo_stack.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::MoveNode {
            node_id: "button".to_string(),
            direction: "Up".to_string(),
        })
    );

    let _ = undo_stack.undo().expect("undo");
    assert_eq!(undo_stack.next_redo_label().as_deref(), Some("Move Node"));
    assert_eq!(
        undo_stack.next_redo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::MoveNode {
            node_id: "button".to_string(),
            direction: "Up".to_string(),
        })
    );
}

#[test]
fn ui_asset_editor_session_tracks_explicit_inverse_tree_edits_for_insert_and_unwrap() {
    let insert_route = UiAssetEditorRoute::new(
        "asset://ui/tests/simple-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut insert_session = UiAssetEditorSession::from_source(
        insert_route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("insert session");
    let palette_index = insert_session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");
    insert_session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(insert_session
        .select_palette_index(palette_index)
        .expect("select button palette item"));
    assert!(insert_session
        .insert_selected_palette_item_as_child()
        .expect("insert button as child"));
    assert_eq!(
        insert_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::RemoveNode {
            node_id: "button".to_string(),
            parent_node_id: Some("root".to_string()),
        })
    );
    assert!(insert_session.undo().expect("undo insert"));
    assert_eq!(
        insert_session.next_redo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::RemoveNode {
            node_id: "button".to_string(),
            parent_node_id: Some("root".to_string()),
        })
    );

    let unwrap_route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut unwrap_session = UiAssetEditorSession::from_source(
        unwrap_route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("unwrap session");
    unwrap_session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(unwrap_session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap selected node"));
    let wrapper_id = unwrap_session
        .pane_presentation()
        .inspector_selected_node_id;
    assert!(unwrap_session
        .unwrap_selected_node()
        .expect("unwrap selected wrapper"));
    assert_eq!(
        unwrap_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::WrapNode {
            node_id: "button".to_string(),
            wrapper_node_id: wrapper_id,
            wrapper_widget_type: "VerticalBox".to_string(),
        })
    );
}

#[test]
fn ui_asset_editor_session_tracks_explicit_inverse_tree_edits_for_reparent_and_reference_conversion(
) {
    let reparent_route = UiAssetEditorRoute::new(
        "asset://ui/tests/tree-reparent.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut reparent_session = UiAssetEditorSession::from_source(
        reparent_route,
        TREE_REPARENT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("reparent session");
    reparent_session
        .select_hierarchy_index(3)
        .expect("select loose node from hierarchy");
    assert!(reparent_session
        .reparent_selected_node_into_previous()
        .expect("reparent into previous sibling container"));
    assert!(reparent_session
        .reparent_selected_node_outdent()
        .expect("outdent node"));
    assert_eq!(
        reparent_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::ReparentNode {
            node_id: "loose".to_string(),
            parent_node_id: Some("group_a".to_string()),
            direction: "into_previous".to_string(),
        })
    );

    let convert_route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut convert_session = UiAssetEditorSession::from_source(
        convert_route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("convert session");
    let imported_widget =
        crate::tests::support::load_test_ui_asset(PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML)
            .expect("parameterized imported widget");
    let reference = "asset://ui/common/toolbar_button.ui#ToolbarButton";
    convert_session
        .register_widget_import(reference, imported_widget)
        .expect("register widget import");
    let palette_index = convert_session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Reference / ToolbarButton")
        .expect("toolbar reference palette item");
    convert_session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(convert_session
        .select_palette_index(palette_index)
        .expect("select toolbar reference palette item"));
    assert!(convert_session
        .convert_selected_node_to_reference()
        .expect("convert selected node to reference"));
    assert_eq!(
        convert_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::RestoreNodeDefinition {
            node_id: "button".to_string(),
            kind: zircon_runtime::ui::template::UiNodeDefinitionKind::Native,
            widget_type: Some("Button".to_string()),
            component: None,
            component_ref: None,
        })
    );
}

#[test]
fn ui_asset_editor_session_tracks_explicit_inverse_tree_edits_for_extract_and_promote() {
    let extract_route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut extract_session = UiAssetEditorSession::from_source(
        extract_route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("extract session");
    extract_session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(extract_session
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));
    assert_eq!(
        extract_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::InlineExtractedComponent {
            node_id: "button".to_string(),
            component_name: "SaveButton".to_string(),
            component_root_id: "savebutton_root".to_string(),
        })
    );

    let promote_route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut promote_session = UiAssetEditorSession::from_source(
        promote_route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("promote session");
    promote_session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(promote_session
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));
    assert!(promote_session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.ui.toml",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component")
        .is_some());
    assert_eq!(
        promote_session.next_undo_inverse_tree_edit(),
        Some(UiAssetEditorInverseTreeEdit::RestorePromotedComponent {
            source_component_name: "SaveButton".to_string(),
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            component_name: "SaveButton".to_string(),
            document_id: "ui.widgets.save_button".to_string(),
        })
    );
}

#[test]
fn ui_asset_editor_undo_stack_tracks_composite_external_effect_vectors() {
    let mut undo_stack = UiAssetEditorUndoStack::default();
    undo_stack.push_edit(
        "Composite Effects",
        Some(UiAssetEditorTreeEdit::Generic {
            kind: UiAssetEditorTreeEditKind::DocumentEdit,
        }),
        None,
        SIMPLE_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        Some(crate::tests::support::load_test_ui_asset(SIMPLE_LAYOUT_ASSET_TOML).expect("before")),
        STYLED_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        Some(crate::tests::support::load_test_ui_asset(STYLED_LAYOUT_ASSET_TOML).expect("after")),
        UiAssetEditorUndoExternalEffects {
            undo: vec![
                UiAssetEditorExternalEffect::RemoveAssetSource {
                    asset_id: "res://ui/theme/editor_base.ui.toml".to_string(),
                },
                UiAssetEditorExternalEffect::UpsertAssetSource {
                    asset_id: "res://ui/theme/editor_local.ui.toml".to_string(),
                    source:
                        "[asset]\nkind = \"style\"\nid = \"ui.theme.editor_local\"\nversion = 1\n"
                            .to_string(),
                },
            ],
            redo: vec![
                UiAssetEditorExternalEffect::UpsertAssetSource {
                    asset_id: "res://ui/theme/editor_base.ui.toml".to_string(),
                    source:
                        "[asset]\nkind = \"style\"\nid = \"ui.theme.editor_base\"\nversion = 1\n"
                            .to_string(),
                },
                UiAssetEditorExternalEffect::RemoveAssetSource {
                    asset_id: "res://ui/theme/editor_local.ui.toml".to_string(),
                },
            ],
        },
    );

    assert_eq!(
        undo_stack.next_undo_external_effects(),
        vec![
            UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: "res://ui/theme/editor_base.ui.toml".to_string(),
            },
            UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: "res://ui/theme/editor_local.ui.toml".to_string(),
                source: "[asset]\nkind = \"style\"\nid = \"ui.theme.editor_local\"\nversion = 1\n"
                    .to_string(),
            },
        ]
    );

    let _ = undo_stack.undo().expect("undo composite");
    assert_eq!(
        undo_stack.next_redo_external_effects(),
        vec![
            UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: "res://ui/theme/editor_base.ui.toml".to_string(),
                source: "[asset]\nkind = \"style\"\nid = \"ui.theme.editor_base\"\nversion = 1\n"
                    .to_string(),
            },
            UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: "res://ui/theme/editor_local.ui.toml".to_string(),
            },
        ]
    );
}

#[test]
fn ui_asset_editor_undo_stack_keeps_source_only_replays_for_source_edits() {
    let mut undo_stack = UiAssetEditorUndoStack::default();
    undo_stack.push_edit(
        "Source Edit",
        None,
        None,
        SIMPLE_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        None,
        STYLED_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        None,
        UiAssetEditorUndoExternalEffects::default(),
    );

    let undone = undo_stack.undo().expect("undo snapshot");
    assert!(undone.document.is_none());
    let mut undone_source = STYLED_LAYOUT_ASSET_TOML.to_string();
    assert!(undone
        .apply_to_source(&mut undone_source)
        .expect("apply undo source replay"));
    assert_eq!(undone_source, SIMPLE_LAYOUT_ASSET_TOML);

    let redone = undo_stack.redo().expect("redo snapshot");
    assert!(redone.document.is_none());
    let mut redone_source = SIMPLE_LAYOUT_ASSET_TOML.to_string();
    assert!(redone
        .apply_to_source(&mut redone_source)
        .expect("apply redo source replay"));
    assert_eq!(redone_source, STYLED_LAYOUT_ASSET_TOML);
}

#[test]
fn ui_asset_editor_session_redo_restores_tree_edit_selection_and_source_summary() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
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
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert button as child"));

    let inserted = session.pane_presentation();
    assert_eq!(inserted.inspector_selected_node_id, "button_2");
    assert_eq!(inserted.source_selected_block_label, "[nodes.button_2]");
    assert!(inserted
        .source_selected_excerpt
        .contains("[nodes.button_2]"));

    assert!(session.undo().expect("undo tree edit"));
    let undone = session.pane_presentation();
    assert_eq!(undone.inspector_selected_node_id, "root");
    assert_eq!(undone.source_selected_block_label, "[nodes.root]");

    assert!(session.redo().expect("redo tree edit"));
    let redone = session.pane_presentation();
    assert_eq!(redone.inspector_selected_node_id, "button_2");
    assert_eq!(redone.source_selected_block_label, "[nodes.button_2]");
    assert!(redone.source_selected_excerpt.contains("[nodes.button_2]"));
}
