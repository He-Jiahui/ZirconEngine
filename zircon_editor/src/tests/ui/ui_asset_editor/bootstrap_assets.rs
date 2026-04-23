use super::support::{
    hydrate_bootstrap_imports, register_bootstrap_imports, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML,
};
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
};
use slint::Model;
use zircon_runtime::ui::template::{UiDocumentCompiler, UiNodeDefinitionKind};
use zircon_runtime::ui::{layout::UiSize, template::UiAssetKind};

#[test]
fn ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");
    let mut compiler = UiDocumentCompiler::default();
    register_bootstrap_imports(&mut compiler);

    let compiled = compiler
        .compile(&layout)
        .expect("compile bootstrap editor layout");
    let root = &compiled.template_instance().root;

    assert_eq!(
        compiled.asset.id,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID
    );
    assert_eq!(root.component.as_deref(), Some("VerticalBox"));
    assert!(root.children.len() >= 2);
}

#[test]
fn ui_asset_editor_bootstrap_assets_open_in_session_after_import_hydration() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap session");

    assert!(
        !session.diagnostics().is_empty(),
        "bootstrap session should report missing imports before hydration"
    );

    hydrate_bootstrap_imports(&mut session);

    assert!(
        session.diagnostics().is_empty(),
        "bootstrap session should compile once imports are hydrated"
    );
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID);
    assert!(pane.preview_canvas_items.len() >= 3);
}

#[test]
fn ui_asset_editor_bootstrap_widget_asset_opens_as_self_hosted_widget_session() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap widget session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(!pane.preview_canvas_items.is_empty());
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("section_card_root")));
}

#[test]
fn ui_asset_editor_bootstrap_style_asset_opens_as_self_hosted_style_session() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
        UiAssetKind::Style,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap style session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(!pane.preview_available);
    assert_eq!(pane.style_token_items.len(), 4);
    assert_eq!(pane.style_rule_items.len(), 5);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID);
}

#[test]
fn ui_asset_editor_bootstrap_layout_uses_shared_header_shell_widget_reference() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    assert!(layout
        .imports
        .widgets
        .iter()
        .any(|reference| reference == UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE));
    let header_panel = layout.node("header_panel").expect("header panel node");
    assert_eq!(header_panel.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        header_panel.component_ref.as_deref(),
        Some(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE)
    );
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_shell_columns_and_panels() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    for required_node in [
        "header_panel",
        "header_asset_row",
        "header_status_row",
        "header_action_row",
        "body",
        "left_column",
        "center_column",
        "right_column",
        "palette_panel",
        "hierarchy_panel",
        "designer_panel",
        "designer_canvas_panel",
        "render_stack_panel",
        "action_bar_panel",
        "action_insert_row",
        "action_reparent_row",
        "action_structure_row",
        "source_panel",
        "source_info_panel",
        "source_outline_panel",
        "mock_workspace_panel",
        "mock_subjects_panel",
        "mock_editor_panel",
        "mock_state_graph_panel",
        "source_text_panel",
        "inspector_panel",
        "inspector_content_panel",
        "inspector_widget_section",
        "inspector_promote_section",
        "inspector_slot_section",
        "inspector_layout_section",
        "inspector_binding_section",
        "stylesheet_panel",
        "stylesheet_action_row",
        "stylesheet_state_primary_row",
        "stylesheet_state_secondary_row",
        "stylesheet_content_panel",
        "stylesheet_theme_section",
        "stylesheet_authoring_section",
        "stylesheet_matched_rule_section",
    ] {
        assert!(
            layout.contains_node(required_node),
            "bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_header_shell_rows() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    for required_node in ["header_asset_row", "header_status_row", "header_action_row"] {
        assert!(
            layout.contains_node(required_node),
            "bootstrap layout should include header shell node `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_bootstrap_template_projection_exposes_pane_shell_regions() {
    let nodes = crate::ui::layouts::views::ui_asset_editor_pane_nodes(UiSize::new(1280.0, 720.0));
    let nodes = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes.iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for control_id in [
        "HeaderPanel",
        "HeaderAssetRow",
        "HeaderStatusRow",
        "HeaderActionRow",
        "LeftColumn",
        "CenterColumn",
        "RightColumn",
        "PalettePanel",
        "HierarchyPanel",
        "DesignerPanel",
        "DesignerCanvasPanel",
        "RenderStackPanel",
        "ActionBarPanel",
        "ActionInsertRow",
        "ActionReparentRow",
        "ActionStructureRow",
        "SourcePanel",
        "SourceInfoPanel",
        "SourceOutlinePanel",
        "MockWorkspacePanel",
        "MockSubjectsPanel",
        "MockEditorPanel",
        "MockStateGraphPanel",
        "SourceTextPanel",
        "InspectorPanel",
        "InspectorContentPanel",
        "InspectorWidgetSection",
        "InspectorPromoteSection",
        "InspectorSlotSection",
        "InspectorLayoutSection",
        "InspectorBindingSection",
        "StylesheetPanel",
        "StylesheetActionRow",
        "StylesheetStatePrimaryRow",
        "StylesheetStateSecondaryRow",
        "StylesheetContentPanel",
        "StylesheetThemeSection",
        "StylesheetAuthoringSection",
        "StylesheetMatchedRuleSection",
    ] {
        let node = node(control_id);
        assert!(
            node.frame.width > 0.0 && node.frame.height > 0.0,
            "expected `{control_id}` node to be laid out by the bootstrap asset, got {:?}",
            node.frame
        );
    }

    assert!(node("HeaderPanel").frame.y <= node("PalettePanel").frame.y);
    assert!(node("HeaderAssetRow").frame.y >= node("HeaderPanel").frame.y);
    assert!(node("HeaderStatusRow").frame.y >= node("HeaderAssetRow").frame.y);
    assert!(node("HeaderActionRow").frame.y >= node("HeaderStatusRow").frame.y);
    assert!(node("LeftColumn").frame.x < node("CenterColumn").frame.x);
    assert!(node("CenterColumn").frame.x < node("RightColumn").frame.x);
    assert!(node("PalettePanel").frame.x < node("DesignerPanel").frame.x);
    assert!(node("DesignerPanel").frame.x < node("InspectorPanel").frame.x);
    assert!(node("DesignerCanvasPanel").frame.y >= node("DesignerPanel").frame.y);
    assert!(node("RenderStackPanel").frame.y >= node("DesignerCanvasPanel").frame.y);
    assert!(node("ActionBarPanel").frame.y >= node("DesignerPanel").frame.y);
    assert!(node("ActionInsertRow").frame.y >= node("ActionBarPanel").frame.y);
    assert!(node("ActionReparentRow").frame.y >= node("ActionInsertRow").frame.y);
    assert!(node("ActionStructureRow").frame.y >= node("ActionReparentRow").frame.y);
    assert!(node("SourcePanel").frame.y >= node("ActionBarPanel").frame.y);
    assert!(node("SourceInfoPanel").frame.y >= node("SourcePanel").frame.y);
    assert!(node("SourceOutlinePanel").frame.y >= node("SourceInfoPanel").frame.y);
    assert!(node("MockWorkspacePanel").frame.y >= node("SourceOutlinePanel").frame.y);
    assert!(node("MockSubjectsPanel").frame.y >= node("MockWorkspacePanel").frame.y);
    assert!(node("MockEditorPanel").frame.y >= node("MockSubjectsPanel").frame.y);
    assert!(node("MockStateGraphPanel").frame.y >= node("MockEditorPanel").frame.y);
    assert!(node("SourceTextPanel").frame.y >= node("MockStateGraphPanel").frame.y);
    assert!(node("InspectorContentPanel").frame.y >= node("InspectorPanel").frame.y);
    assert!(node("InspectorWidgetSection").frame.y >= node("InspectorContentPanel").frame.y);
    assert!(node("InspectorPromoteSection").frame.y >= node("InspectorWidgetSection").frame.y);
    assert!(node("InspectorSlotSection").frame.y >= node("InspectorPromoteSection").frame.y);
    assert!(node("InspectorLayoutSection").frame.y >= node("InspectorSlotSection").frame.y);
    assert!(node("InspectorBindingSection").frame.y >= node("InspectorLayoutSection").frame.y);
    assert!(node("StylesheetPanel").frame.y >= node("InspectorPanel").frame.y);
    assert!(node("StylesheetActionRow").frame.y >= node("StylesheetPanel").frame.y);
    assert!(
        node("StylesheetStatePrimaryRow").frame.y >= node("StylesheetActionRow").frame.y
    );
    assert!(
        node("StylesheetStateSecondaryRow").frame.y
            >= node("StylesheetStatePrimaryRow").frame.y
    );
    assert!(
        node("StylesheetContentPanel").frame.y
            >= node("StylesheetStateSecondaryRow").frame.y
    );
    assert!(node("StylesheetThemeSection").frame.y >= node("StylesheetContentPanel").frame.y);
    assert!(
        node("StylesheetAuthoringSection").frame.y >= node("StylesheetThemeSection").frame.y
    );
    assert!(
        node("StylesheetMatchedRuleSection").frame.y
            >= node("StylesheetAuthoringSection").frame.y
    );
}
