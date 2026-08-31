use super::super::support::*;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::UiNodeDefinitionKind;

#[test]
fn ui_asset_editor_session_creates_reference_nodes_from_imported_widget_palette_entries() {
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
    let imported_widget = crate::tests::support::load_test_ui_asset(IMPORTED_WIDGET_ASSET_TOML)
        .expect("imported widget");
    let reference = "asset://ui/common/confirm_button.ui#ConfirmButton";
    session
        .register_widget_import(reference, imported_widget)
        .expect("register widget import");
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Reference / ConfirmButton")
        .expect("reference palette item");

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select reference palette item"));
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert reference node"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let reference_node = document
        .iter_nodes()
        .find(|node| node.kind == UiNodeDefinitionKind::Reference)
        .expect("reference node");
    assert_eq!(reference_node.component_ref.as_deref(), Some(reference));
}

#[test]
fn ui_asset_editor_session_resolves_selected_reference_asset_id() {
    let route = UiAssetEditorRoute::new(
        "res://ui/layouts/reference_selection.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        REFERENCE_SELECTION_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    assert_eq!(session.selected_reference_asset_id(), None);

    session
        .select_hierarchy_index(1)
        .expect("select reference node from hierarchy");

    assert_eq!(
        session.selected_reference_asset_id().as_deref(),
        Some("res://ui/widgets/button.zui")
    );
    assert!(session.pane_presentation().can_open_reference);
}

#[test]
fn ui_asset_editor_session_wraps_and_unwraps_selected_node() {
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
    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap selected node"));

    let wrapped = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let wrapper_id = wrapped
        .node("root")
        .and_then(|node| node.children.first())
        .map(|child| child.node.node_id.clone())
        .expect("wrapper child");
    assert_ne!(wrapper_id, "button");
    assert_eq!(
        wrapped
            .node(&wrapper_id)
            .and_then(|node| node.widget_type.as_deref()),
        Some("VerticalBox")
    );
    assert_eq!(
        wrapped.node(&wrapper_id).map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["button".to_string()])
    );

    assert!(session.unwrap_selected_node().expect("unwrap wrapper"));
    let unwrapped = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert_eq!(
        unwrapped.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["button".to_string()])
    );
}

#[test]
fn ui_asset_editor_session_projects_canvas_insert_and_wrap_availability() {
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

    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Label")
        .expect("label palette item");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select label palette item"));

    let root_pane = session.pane_presentation();
    assert!(root_pane.can_insert_child);
    assert!(root_pane.can_insert_after);
    assert!(!root_pane.can_move_up);
    assert!(!root_pane.can_move_down);
    assert!(!root_pane.can_wrap_in_vertical_box);
    assert!(!root_pane.can_unwrap);

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let button_pane = session.pane_presentation();
    assert!(!button_pane.can_insert_child);
    assert!(button_pane.can_insert_after);
    assert!(!button_pane.can_move_up);
    assert!(!button_pane.can_move_down);
    assert!(button_pane.can_wrap_in_vertical_box);
    assert!(!button_pane.can_unwrap);

    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap selected node"));
    let wrapped_pane = session.pane_presentation();
    assert!(wrapped_pane.can_unwrap);
}

#[test]
fn ui_asset_editor_session_reparents_nodes_into_sibling_containers_and_outdents() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/tree-reparent.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        TREE_REPARENT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(3)
        .expect("select loose node from hierarchy");
    assert!(session
        .reparent_selected_node_into_previous()
        .expect("reparent into previous sibling container"));

    let previous = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert_eq!(
        previous.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["group_a".to_string(), "group_b".to_string()])
    );
    assert_eq!(
        previous.node("group_a").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["nested_a".to_string(), "loose".to_string()])
    );
    let previous_pane = session.pane_presentation();
    assert_eq!(previous_pane.inspector_selected_node_id, "loose");
    assert_eq!(previous_pane.inspector_parent_node_id, "group_a");
    assert_eq!(previous_pane.source_selected_block_label, "[nodes.loose]");

    assert!(session
        .reparent_selected_node_outdent()
        .expect("outdent reparented node"));
    let outdented = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("outdented document");
    assert_eq!(
        outdented.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec![
            "group_a".to_string(),
            "loose".to_string(),
            "group_b".to_string()
        ])
    );

    assert!(session
        .reparent_selected_node_into_next()
        .expect("reparent into next sibling container"));
    let next = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    assert_eq!(
        next.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["group_a".to_string(), "group_b".to_string()])
    );
    assert_eq!(
        next.node("group_b").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["loose".to_string(), "nested_b".to_string()])
    );
    let next_pane = session.pane_presentation();
    assert_eq!(next_pane.inspector_selected_node_id, "loose");
    assert_eq!(next_pane.inspector_parent_node_id, "group_b");
    assert_eq!(next_pane.source_selected_block_label, "[nodes.loose]");
}

#[test]
fn ui_asset_editor_session_projects_canvas_move_and_reparent_availability() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/tree-reparent.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        TREE_REPARENT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(3)
        .expect("select loose node from hierarchy");
    let loose_pane = session.pane_presentation();
    assert!(loose_pane.can_move_up);
    assert!(loose_pane.can_move_down);
    assert!(loose_pane.can_reparent_into_previous);
    assert!(loose_pane.can_reparent_into_next);
    assert!(!loose_pane.can_reparent_outdent);

    assert!(session
        .reparent_selected_node_into_previous()
        .expect("reparent into previous sibling container"));
    let nested_pane = session.pane_presentation();
    assert!(!nested_pane.can_reparent_into_previous);
    assert!(!nested_pane.can_reparent_into_next);
    assert!(nested_pane.can_reparent_outdent);
}

#[test]
fn ui_asset_editor_session_converts_selected_node_to_reference_from_palette_selection() {
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
    let imported_widget =
        crate::tests::support::load_test_ui_asset(PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML)
            .expect("parameterized imported widget");
    let reference = "asset://ui/common/toolbar_button.ui#ToolbarButton";
    session
        .register_widget_import(reference, imported_widget)
        .expect("register widget import");
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Reference / ToolbarButton")
        .expect("toolbar reference palette item");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select toolbar reference palette item"));
    assert!(session.pane_presentation().can_convert_to_reference);

    assert!(session
        .convert_selected_node_to_reference()
        .expect("convert selected node to reference"));
    assert_eq!(
        session.next_undo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::ConvertToReference)
    );

    let converted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("converted document");
    let button = converted.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(button.component_ref.as_deref(), Some(reference));
    assert_eq!(button.control_id.as_deref(), Some("SaveButton"));
    assert_eq!(button.classes, vec!["primary".to_string()]);
    assert_eq!(
        button.params.get("text").and_then(toml::Value::as_str),
        Some("Save")
    );
    assert!(button.props.is_empty());
    assert!(button.layout.is_none());
    assert!(button.bindings.is_empty());

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_widget_kind, "Reference");
    assert_eq!(pane.inspector_widget_label, "ToolbarButton");
    assert!(pane.can_open_reference);
    assert!(!pane.can_convert_to_reference);

    assert!(session.undo().expect("undo convert to reference"));
    assert_eq!(
        session.next_redo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::ConvertToReference)
    );
    let undone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("undone document");
    let button = undone.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Native);
    assert_eq!(button.widget_type.as_deref(), Some("Button"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Save")
    );
}
