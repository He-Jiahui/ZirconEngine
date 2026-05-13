use super::support::{UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML};
use crate::ui::asset_editor::{
    ui_asset_editor_node_projection, UiAssetEditorCommand, UiAssetEditorMode, UiAssetEditorRoute,
    UiAssetEditorSession, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
};
use zircon_runtime::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const UI_ASSET_EDITOR_PROJECTION_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.v2.ui.toml"
));

const V2_IMPORTED_CARD_TOML: &str = r#"
[asset]
kind = "component"
id = "editor.test.card.components"
version = 2
display_name = "Test Card Components"

[components.Card]
root = "card_root"
default_classes = ["prototype-default"]

[components.Card.slots.body]
required = true
multiple = true

[nodes.card_root]
component = "VerticalGroup"
control_id = "CardRoot"
classes = ["card-root"]
props = { text = "Prototype title" }
children = [
  { node = "card_title" },
  { node = "card_body_slot" },
]

[nodes.card_title]
component = "Text"
props = { text = "Prototype title" }

[nodes.card_body_slot]
component = "Slot"
props = { name = "body" }
"#;

const V2_VIEW_WITH_IMPORTED_CARD_TOML: &str = r#"
[asset]
kind = "view"
id = "editor.test.imported.card.view"
version = 2
display_name = "Imported Card View"

[imports]
widgets = ["res://ui/editor/test_card.v2.ui.toml#Card"]

[root]
node = "root"

[nodes.root]
component = "res://ui/editor/test_card.v2.ui.toml#Card"
control_id = "ImportedCard"
classes = ["instance-class"]
props = { variant = "filled" }
state = { selected = true }
children = [
  { node = "body_text", slot = { name = "body" } },
]

[nodes.body_text]
component = "Text"
control_id = "ImportedCardBody"
props = { text = "Projected body" }
"#;

const V2_COMPONENT_WITH_MISSING_ROOT_TOML: &str = r#"
[asset]
kind = "component"
id = "editor.test.missing.component.root"
version = 2
display_name = "Missing Component Root"

[components.Card]
root = "missing_root"

[nodes.unused]
component = "Text"
props = { text = "Unused" }
"#;

const V2_COMPONENT_WITH_CYCLIC_ROOT_TOML: &str = r#"
[asset]
kind = "component"
id = "editor.test.cyclic.component.root"
version = 2
display_name = "Cyclic Component Root"

[components.Card]
root = "card_root"

[nodes.card_root]
component = "VerticalGroup"
children = [
  { node = "card_child" },
]

[nodes.card_child]
component = "Text"
children = [
  { node = "card_root" },
]
"#;

#[test]
fn ui_asset_editor_v2_bootstrap_asset_parses_and_compiles() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap v2 layout asset");

    let compiled =
        UiV2DocumentCompiler::compile(&layout).expect("compile bootstrap v2 editor layout");
    let root = compiled.arena.root.expect("v2 bootstrap root");
    let root_node = &compiled.arena.nodes[root.index()];

    assert_eq!(
        compiled.asset_id,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID
    );
    assert_eq!(root_node.component, "VerticalGroup");
    assert!(root_node.children.len() >= 2);
}

#[test]
fn ui_asset_editor_v2_projection_asset_self_hosts_shell_regions() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor v2 projection asset");

    assert_eq!(layout.asset.id, "editor.ui_asset_editor.projection.v2");
    assert_eq!(layout.root_node_id(), Some("ui_asset_editor_root"));
    assert!(layout
        .imports
        .styles
        .iter()
        .any(|reference| reference == "res://ui/theme/editor_material.v2.ui.toml"));

    for required_node in [
        "header_panel",
        "body",
        "left_column",
        "center_column",
        "right_column",
        "designer_panel",
        "designer_canvas_panel",
        "source_panel",
        "inspector_panel",
        "stylesheet_panel",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "v2 projection asset should include `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_bootstrap_assets_open_in_session_after_import_hydration() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_v2_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap session");

    assert!(
        session.diagnostics().is_empty(),
        "bootstrap session should auto-resolve bundled imports before explicit hydration"
    );
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID);
    assert!(pane.preview_canvas_items.len() >= 3);
}

#[test]
fn ui_asset_editor_v2_authoring_keeps_v2_source_on_edit_and_canonical_save() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap v2 session");
    let edited = UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML
        .replace("Valid prototype cache", "V2 authoring cache");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited))
        .expect("apply v2 source edit");

    assert!(session.diagnostics().is_empty());
    assert!(session
        .source_buffer()
        .text()
        .contains("V2 authoring cache"));
    let canonical = session.canonical_source().expect("v2 canonical source");
    assert!(canonical.contains("kind = \"view\""));
    assert!(canonical.contains("version = 2"));
    assert!(canonical.contains("[nodes.ui_asset_editor_root]"));
    assert!(!canonical.contains("kind = \"layout\""));
    assert!(!canonical.contains("node_id = \"ui_asset_editor_root\""));
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("ui_asset_editor_root [VerticalGroup]")));
}

#[test]
fn ui_asset_editor_v2_authoring_instantiates_imported_component_slots_for_preview() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/imported_card_view.v2.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(
        route,
        V2_VIEW_WITH_IMPORTED_CARD_TOML,
        UiSize::new(480.0, 320.0),
    )
    .expect("v2 imported component session");
    let component =
        UiV2AssetLoader::load_toml_str(V2_IMPORTED_CARD_TOML).expect("v2 imported component asset");

    session
        .register_v2_widget_import("res://ui/editor/test_card.v2.ui.toml", component)
        .expect("register v2 component prototype import");

    assert!(session.diagnostics().is_empty());
    let surface = session.preview_host_opt().expect("v2 preview").surface();
    let imported_root = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ImportedCard")
        })
        .expect("imported component root should be projected into preview");
    let root_metadata = imported_root.template_metadata.as_ref().unwrap();
    assert_eq!(root_metadata.component, "VerticalGroup");
    assert!(root_metadata
        .classes
        .iter()
        .any(|class| class == "card-root"));
    assert!(root_metadata
        .classes
        .iter()
        .any(|class| class == "prototype-default"));
    assert!(root_metadata
        .classes
        .iter()
        .any(|class| class == "instance-class"));
    assert_eq!(
        root_metadata
            .attributes
            .get("variant")
            .and_then(toml::Value::as_str),
        Some("filled")
    );
    assert_eq!(
        root_metadata
            .attributes
            .get("selected")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    assert!(surface.tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some("ImportedCardBody")
    }));
    let canonical = session.canonical_source().expect("v2 canonical source");
    assert!(canonical.contains("res://ui/editor/test_card.v2.ui.toml#Card"));
    assert!(canonical.contains("[nodes.body_text]"));
    assert!(!canonical.contains("[nodes.card_root]"));
}

#[test]
fn ui_asset_editor_v2_component_asset_opens_as_editable_component_tree() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/test_card.v2.ui.toml",
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(
        route,
        V2_IMPORTED_CARD_TOML,
        UiSize::new(480.0, 320.0),
    )
    .expect("v2 component asset session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .palette_items
        .iter()
        .any(|item| item == "Component / Card"));
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("card_root [VerticalGroup]")));
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("card_body_slot [Slot]")));
    assert_eq!(pane.inspector_selected_node_id, "card_root");

    let preview = session.preview_host_opt().expect("component preview");
    assert!(preview.surface().tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some("CardRoot")
    }));

    assert!(session
        .set_selected_widget_text_property("Edited component title")
        .expect("edit selected component root text"));
    let canonical = session
        .canonical_source()
        .expect("v2 component canonical source");
    assert!(canonical.contains("kind = \"component\""));
    assert!(canonical.contains("[components.Card]"));
    assert!(canonical.contains("root = \"card_root\""));
    assert!(canonical.contains("[components.Card.slots.body]"));
    assert!(canonical.contains("[nodes.card_root]"));
    assert!(canonical.contains("text = \"Edited component title\""));
    assert!(canonical.contains("[nodes.card_body_slot]"));
    assert!(!canonical.contains("kind = \"widget\""));
}

#[test]
fn ui_asset_editor_v2_component_asset_patches_props_and_state_from_authoring_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/test_card.v2.ui.toml",
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(
        route,
        V2_IMPORTED_CARD_TOML,
        UiSize::new(480.0, 320.0),
    )
    .expect("v2 component asset session");

    assert_eq!(
        session.pane_presentation().inspector_selected_node_id,
        "card_root"
    );
    assert!(session
        .set_selected_widget_prop_literal("variant", "\"outlined\"")
        .expect("patch selected component root prop"));
    assert!(session
        .set_selected_widget_state_literal("expanded", "true")
        .expect("patch selected component root state"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_items
        .iter()
        .any(|item| item == "prop variant = \"outlined\""));
    assert!(pane
        .inspector_items
        .iter()
        .any(|item| item == "state expanded = true"));
    assert!(pane
        .inspector_widget_prop_state_items
        .iter()
        .any(|item| item == "prop variant = \"outlined\""));
    assert!(pane
        .inspector_widget_prop_state_items
        .iter()
        .any(|item| item == "state expanded = true"));
    assert!(pane.inspector_widget_prop_state_rows.iter().any(|item| {
        item.kind == "prop" && item.path == "variant" && item.value == "\"outlined\""
    }));
    assert!(pane
        .inspector_widget_prop_state_rows
        .iter()
        .any(|item| item.kind == "state" && item.path == "expanded" && item.value == "true"));

    let canonical = session
        .canonical_source()
        .expect("v2 component canonical source");
    assert!(canonical.contains("kind = \"component\""));
    assert!(canonical.contains("[components.Card]"));
    assert!(canonical.contains("[nodes.card_root]"));
    assert!(canonical.contains("variant = \"outlined\""));
    assert!(canonical.contains("expanded = true"));
    assert!(!canonical.contains("kind = \"widget\""));
}

#[test]
fn ui_asset_editor_v2_component_asset_rejects_missing_component_root() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/missing_component_root.v2.ui.toml",
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let error = match UiAssetEditorSession::from_v2_source(
        route,
        V2_COMPONENT_WITH_MISSING_ROOT_TOML,
        UiSize::new(480.0, 320.0),
    ) {
        Ok(_) => panic!("missing component root should be rejected"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(message.contains("editor.test.missing.component.root"));
    assert!(message.contains("missing_root"));
}

#[test]
fn ui_asset_editor_v2_component_asset_rejects_cyclic_component_projection() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/cyclic_component_root.v2.ui.toml",
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let error = match UiAssetEditorSession::from_v2_source(
        route,
        V2_COMPONENT_WITH_CYCLIC_ROOT_TOML,
        UiSize::new(480.0, 320.0),
    ) {
        Ok(_) => panic!("cyclic component projection should be rejected"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(message.contains("editor.test.cyclic.component.root"));
    assert!(message.contains("cycle"));
    assert!(message.contains("card_root"));
}

#[test]
fn ui_asset_editor_bootstrap_style_asset_opens_as_v2_style_session() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
        UiAssetKind::Style,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_v2_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap v2 style session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(!pane.preview_available);
    assert!(pane.style_token_items.len() >= 20);
    assert!(pane.style_rule_items.len() >= 10);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID);
}

#[test]
fn ui_asset_editor_bootstrap_layout_no_longer_imports_legacy_editor_widgets() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap v2 layout asset");

    assert!(layout
        .imports
        .styles
        .iter()
        .any(|reference| reference == UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID));
    assert!(layout.imports.widgets.is_empty());
    assert!(!UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML.contains("editor_widgets.ui.toml"));
    assert!(!UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML.contains("editor_base.ui.toml"));
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_shell_columns_and_panels() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap v2 layout asset");

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
        "designer_tool_mode_row",
        "designer_canvas_panel",
        "designer_diagnostic_overlay_panel",
        "emergency_shell_panel",
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
            layout.nodes.contains_key(required_node),
            "bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_header_shell_rows() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap v2 layout asset");

    for required_node in ["header_asset_row", "header_status_row", "header_action_row"] {
        assert!(
            layout.nodes.contains_key(required_node),
            "bootstrap layout should include header shell node `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_bootstrap_template_projection_exposes_pane_shell_regions() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(1280.0, 720.0)).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
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
        "DesignerToolModeRow",
        "DesignerCanvasPanel",
        "DesignerDiagnosticOverlayPanel",
        "EmergencyShellPanel",
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
    assert!(node("DesignerToolModeRow").frame.y >= node("DesignerPanel").frame.y);
    assert!(node("DesignerCanvasPanel").frame.y >= node("DesignerToolModeRow").frame.y);
    assert!(node("DesignerDiagnosticOverlayPanel").frame.y >= node("DesignerCanvasPanel").frame.y);
    assert!(node("EmergencyShellPanel").frame.y >= node("DesignerDiagnosticOverlayPanel").frame.y);
    assert!(node("RenderStackPanel").frame.y >= node("EmergencyShellPanel").frame.y);
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
    assert!(node("StylesheetStatePrimaryRow").frame.y >= node("StylesheetActionRow").frame.y);
    assert!(
        node("StylesheetStateSecondaryRow").frame.y >= node("StylesheetStatePrimaryRow").frame.y
    );
    assert!(node("StylesheetContentPanel").frame.y >= node("StylesheetStateSecondaryRow").frame.y);
    assert!(node("StylesheetThemeSection").frame.y >= node("StylesheetContentPanel").frame.y);
    assert!(node("StylesheetAuthoringSection").frame.y >= node("StylesheetThemeSection").frame.y);
    assert!(
        node("StylesheetMatchedRuleSection").frame.y >= node("StylesheetAuthoringSection").frame.y
    );
}
