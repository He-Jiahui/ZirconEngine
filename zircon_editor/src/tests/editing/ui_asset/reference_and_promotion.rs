use super::support::*;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiNodeDefinitionKind, UiRootClassPolicy};

#[test]
fn ui_asset_editor_session_creates_reference_nodes_from_imported_widget_palette_entries() {
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
        "res://ui/layouts/reference_selection.ui.toml",
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
        Some("res://ui/widgets/button.ui.toml")
    );
    assert!(session.pane_presentation().can_open_reference);
}

#[test]
fn ui_asset_editor_session_wraps_and_unwraps_selected_node() {
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
        "asset://ui/tests/tree-reparent.ui.toml",
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
        "asset://ui/tests/tree-reparent.ui.toml",
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

#[test]
fn ui_asset_editor_session_extracts_selected_node_into_local_component() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session.pane_presentation().can_extract_component);

    assert!(session
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));
    assert_eq!(
        session.next_undo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::ExtractComponent)
    );
    assert_eq!(
        session.next_undo_tree_edit(),
        Some(UiAssetEditorTreeEdit::ExtractComponent {
            node_id: "button".to_string(),
            component_name: "SaveButton".to_string(),
            component_root_id: "savebutton_root".to_string(),
        })
    );

    let extracted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("extracted document");
    let component = extracted
        .components
        .get("SaveButton")
        .expect("new local component");
    let instance = extracted.node("button").expect("component instance");
    assert_eq!(instance.kind, UiNodeDefinitionKind::Component);
    assert_eq!(instance.component.as_deref(), Some("SaveButton"));
    assert_eq!(instance.control_id.as_deref(), Some("SaveButton"));
    assert_eq!(instance.classes, vec!["primary".to_string()]);
    assert!(instance.params.is_empty());
    assert!(instance.props.is_empty());
    assert!(instance.layout.is_none());
    assert!(instance.bindings.is_empty());
    assert!(instance.children.is_empty());

    let component_root = extracted
        .node(&component.root.node_id)
        .expect("extracted component root");
    assert_eq!(component_root.kind, UiNodeDefinitionKind::Native);
    assert_eq!(component_root.widget_type.as_deref(), Some("Button"));
    assert_eq!(component_root.control_id.as_deref(), Some("SaveButton"));
    assert_eq!(component_root.classes, vec!["primary".to_string()]);
    assert_eq!(
        component_root
            .props
            .get("text")
            .and_then(toml::Value::as_str),
        Some("Save")
    );

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.inspector_widget_kind, "Component");
    assert_eq!(pane.inspector_widget_label, "SaveButton");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert!(pane
        .palette_items
        .iter()
        .any(|item| item == "Component / SaveButton"));

    assert!(session.undo().expect("undo extract component"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert_eq!(
        session.next_redo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::ExtractComponent)
    );
    assert_eq!(
        session.next_redo_tree_edit(),
        Some(UiAssetEditorTreeEdit::ExtractComponent {
            node_id: "button".to_string(),
            component_name: "SaveButton".to_string(),
            component_root_id: "savebutton_root".to_string(),
        })
    );
    assert!(session.redo().expect("redo extract component"));
    let redone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("redone document");
    assert_eq!(
        redone
            .node("button")
            .and_then(|node| node.component.as_deref()),
        Some("SaveButton")
    );
}

#[test]
fn ui_asset_editor_session_projects_and_updates_root_class_policy() {
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
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));

    let initial = session.pane_presentation();
    assert_eq!(initial.inspector_component_root_class_policy, "append_only");
    assert!(initial.inspector_can_edit_component_root_class_policy);
    assert!(initial
        .inspector_items
        .iter()
        .any(|item| item == "root class policy: append_only"));

    assert!(session
        .set_selected_component_root_class_policy("closed")
        .expect("set root class policy"));
    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_component_root_class_policy, "closed");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("updated document");
    assert_eq!(
        document
            .components
            .get("SaveButton")
            .map(|component| component.contract.root_class_policy),
        Some(UiRootClassPolicy::Closed)
    );

    assert!(session.undo().expect("undo root class policy"));
    let undone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("undone document");
    assert_eq!(
        undone
            .components
            .get("SaveButton")
            .map(|component| component.contract.root_class_policy),
        Some(UiRootClassPolicy::AppendOnly)
    );
    assert!(session.redo().expect("redo root class policy"));
    assert_eq!(
        session
            .pane_presentation()
            .inspector_component_root_class_policy,
        "closed"
    );
}

#[test]
fn ui_asset_editor_session_projects_and_updates_promote_widget_draft_fields() {
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
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));

    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_promote_asset_id,
        "res://ui/widgets/save_button.ui.toml"
    );
    assert_eq!(initial.inspector_promote_component_name, "SaveButton");
    assert_eq!(
        initial.inspector_promote_document_id,
        "ui.widgets.save_button"
    );
    assert!(initial.inspector_can_edit_promote_draft);

    assert!(session
        .set_selected_promote_widget_asset_id("res://ui/widgets/custom/editor_save.ui.toml")
        .expect("set promote widget asset id"));
    assert!(session
        .set_selected_promote_widget_component_name("EditorSaveButton")
        .expect("set promote widget component name"));
    assert!(session
        .set_selected_promote_widget_document_id("ui.widgets.custom.editor_save")
        .expect("set promote widget document id"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.inspector_promote_asset_id,
        "res://ui/widgets/custom/editor_save.ui.toml"
    );
    assert_eq!(updated.inspector_promote_component_name, "EditorSaveButton");
    assert_eq!(
        updated.inspector_promote_document_id,
        "ui.widgets.custom.editor_save"
    );
}

#[test]
fn ui_asset_editor_session_promotes_selected_local_component_to_external_widget_asset() {
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
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));
    assert!(session.pane_presentation().can_promote_to_external_widget);

    let promoted_widget = session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.ui.toml",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component to external widget")
        .expect("promoted widget document");
    assert_eq!(
        session.next_undo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::PromoteToExternalWidget)
    );
    assert_eq!(
        session.next_undo_tree_edit(),
        Some(UiAssetEditorTreeEdit::PromoteToExternalWidget {
            source_component_name: "SaveButton".to_string(),
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            component_name: "SaveButton".to_string(),
            document_id: "ui.widgets.save_button".to_string(),
        })
    );
    assert_eq!(
        session.next_undo_external_effect(),
        Some(UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
        })
    );

    assert_eq!(promoted_widget.asset.kind, UiAssetKind::Widget);
    assert_eq!(promoted_widget.asset.id, "ui.widgets.save_button");
    assert_eq!(promoted_widget.asset.display_name, "SaveButton");
    assert_eq!(
        promoted_widget
            .root
            .as_ref()
            .map(|root| root.node_id.as_str()),
        Some("savebutton_root")
    );
    assert!(promoted_widget.components.contains_key("SaveButton"));
    assert_eq!(
        promoted_widget
            .node("savebutton_root")
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );

    let promoted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("promoted document");
    assert!(!promoted.components.contains_key("SaveButton"));
    assert!(!promoted.contains_node("savebutton_root"));
    assert!(promoted
        .imports
        .widgets
        .iter()
        .any(|reference| { reference == "res://ui/widgets/save_button.ui.toml#SaveButton" }));
    let button = promoted.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/save_button.ui.toml#SaveButton")
    );
    assert_eq!(button.control_id.as_deref(), Some("SaveButton"));
    assert_eq!(button.classes, vec!["primary".to_string()]);
    assert!(button.props.is_empty());
    assert!(button.layout.is_none());
    assert!(button.bindings.is_empty());

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_widget_kind, "Reference");
    assert_eq!(pane.inspector_widget_label, "SaveButton");
    assert!(pane.can_open_reference);
    assert!(!pane.can_promote_to_external_widget);

    assert!(session.undo().expect("undo promote widget"));
    assert_eq!(
        session.next_redo_tree_edit_kind(),
        Some(UiAssetEditorTreeEditKind::PromoteToExternalWidget)
    );
    assert_eq!(
        session.next_redo_tree_edit(),
        Some(UiAssetEditorTreeEdit::PromoteToExternalWidget {
            source_component_name: "SaveButton".to_string(),
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            component_name: "SaveButton".to_string(),
            document_id: "ui.widgets.save_button".to_string(),
        })
    );
    assert_eq!(
        session.next_redo_external_effect(),
        Some(UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            source: toml::to_string_pretty(&promoted_widget)
                .expect("serialize promoted widget document"),
        })
    );
    let undone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("undone document");
    assert_eq!(
        undone
            .node("button")
            .and_then(|node| node.component.as_deref()),
        Some("SaveButton")
    );
    assert!(undone.components.contains_key("SaveButton"));
}

#[test]
fn ui_asset_editor_session_promotes_local_theme_to_external_style_asset_and_links_import() {
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

    let promoted_theme = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/editor_base.ui.toml",
            "ui.theme.editor_base",
            "Editor Base",
        )
        .expect("promote local theme")
        .expect("promoted style asset document");

    assert_eq!(promoted_theme.asset.kind, UiAssetKind::Style);
    assert_eq!(promoted_theme.asset.id, "ui.theme.editor_base");
    assert_eq!(promoted_theme.asset.display_name, "Editor Base");
    assert_eq!(
        promoted_theme
            .tokens
            .get("accent")
            .and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(promoted_theme.stylesheets.len(), 1);
    assert!(promoted_theme.root.is_none());
    assert!(promoted_theme.iter_nodes().next().is_none());
    assert!(promoted_theme.components.is_empty());

    let promoted = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("promoted document");
    assert!(promoted.tokens.is_empty());
    assert!(promoted.stylesheets.is_empty());
    assert_eq!(
        promoted.imports.styles,
        vec!["res://ui/themes/editor_base.ui.toml".to_string()]
    );
    assert_eq!(
        session.next_undo_external_effect(),
        Some(UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/themes/editor_base.ui.toml".to_string(),
        })
    );

    assert!(session.undo().expect("undo promote local theme"));
    let undone = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("undone document");
    assert_eq!(
        undone.tokens.get("accent").and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(undone.stylesheets.len(), 1);
    assert!(undone.imports.styles.is_empty());
    assert_eq!(
        session.next_redo_external_effect(),
        Some(UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/editor_base.ui.toml".to_string(),
            source: toml::to_string_pretty(&promoted_theme)
                .expect("serialize promoted style asset document"),
        })
    );
}
