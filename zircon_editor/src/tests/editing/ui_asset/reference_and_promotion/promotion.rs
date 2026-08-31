use super::super::support::*;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiNodeDefinitionKind, UiRootClassPolicy};

#[test]
fn ui_asset_editor_session_extracts_selected_node_into_local_component() {
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
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));

    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_promote_asset_id,
        "res://ui/widgets/save_button.zui"
    );
    assert_eq!(initial.inspector_promote_component_name, "SaveButton");
    assert_eq!(
        initial.inspector_promote_document_id,
        "ui.widgets.save_button"
    );
    assert!(initial.inspector_can_edit_promote_draft);

    assert!(session
        .set_selected_promote_widget_asset_id("res://ui/widgets/custom/editor_save.zui")
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
        "res://ui/widgets/custom/editor_save.zui"
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
        .extract_selected_node_to_component()
        .expect("extract selected node to component"));
    assert!(session.pane_presentation().can_promote_to_external_widget);

    let promoted_widget = session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.zui",
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
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            component_name: "SaveButton".to_string(),
            document_id: "ui.widgets.save_button".to_string(),
        })
    );
    assert_eq!(
        session.next_undo_external_effect(),
        Some(UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
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
        .any(|reference| { reference == "res://ui/widgets/save_button.zui#SaveButton" }));
    let button = promoted.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/save_button.zui#SaveButton")
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
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            component_name: "SaveButton".to_string(),
            document_id: "ui.widgets.save_button".to_string(),
        })
    );
    assert_eq!(
        session.next_redo_external_effect(),
        Some(UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            source: crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_widget)
                .expect("serialize promoted widget document as v2"),
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

    let promoted_theme = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/editor_base.zui",
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
        vec!["res://ui/themes/editor_base.zui".to_string()]
    );
    assert_eq!(
        session.next_undo_external_effect(),
        Some(UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/themes/editor_base.zui".to_string(),
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
            asset_id: "res://ui/themes/editor_base.zui".to_string(),
            source: crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_theme)
                .expect("serialize promoted style asset document as v2"),
        })
    );
}
