use super::support::{UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML};
use crate::ui::asset_editor::{
    UiAssetEditorCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
};
use zircon_runtime::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const UI_ASSET_EDITOR_PROJECTION_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.zui"
));
const UI_ASSET_EDITOR_ACTION_BAR_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/composites/chrome/workbench_ui_asset_action_bar.zui"
));
const WORKBENCH_CAPTION_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui"
));
const WORKBENCH_LABEL_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/primitives/data/workbench_label.zui"
));

mod component_contracts;

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
widgets = ["res://ui/editor/test_card.zui#Card"]

[root]
node = "root"

[nodes.root]
component = "res://ui/editor/test_card.zui#Card"
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

    assert_eq!(layout.asset.id, "res://ui/editor/ui_asset_editor.zui");
    assert_eq!(layout.root_node_id(), Some("ui_asset_editor_root"));
    assert_eq!(
        layout.imports.styles,
        vec!["res://ui/editor/theme/editor_tokens.zui"],
        "the projected workbench must have one canonical style import"
    );
    assert_eq!(
        layout.imports.widgets,
        vec![
            "res://ui/editor/components/workbench/primitives/inputs/workbench_button.zui#WorkbenchButton",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui#WorkbenchSearchInput",
            "res://ui/editor/components/workbench/primitives/data/workbench_tree_row.zui#WorkbenchTreeRow",
            "res://ui/editor/components/workbench/primitives/data/workbench_caption.zui#WorkbenchCaption",
            "res://ui/editor/components/workbench/primitives/data/workbench_label.zui#WorkbenchLabel",
            "res://ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui#WorkbenchSectionTitle",
            "res://ui/editor/components/workbench/composites/chrome/workbench_ui_asset_action_bar.zui#WorkbenchUiAssetActionBar",
        ],
        "the projected workbench must compose its controls from canonical Workbench primitives"
    );

    for required_node in [
        "header_panel",
        "body",
        "left_column",
        "center_column",
        "right_column",
        "left_scroll_region",
        "center_scroll_region",
        "right_scroll_region",
        "palette_search",
        "palette_component_grid",
        "hierarchy_search",
        "hierarchy_root_row",
        "designer_panel",
        "header_save_button",
        "header_undo_button",
        "header_redo_button",
        "designer_select_button",
        "designer_resize_slot_button",
        "designer_preview_interact_button",
        "designer_canvas_panel",
        "action_bar_panel",
        "source_panel",
        "inspector_panel",
        "stylesheet_panel",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "v2 projection asset should include `{required_node}`"
        );
    }

    for scroll_region in [
        "left_scroll_region",
        "center_scroll_region",
        "right_scroll_region",
    ] {
        let node = layout
            .nodes
            .get(scroll_region)
            .unwrap_or_else(|| panic!("missing scroll region `{scroll_region}`"));
        assert_eq!(node.component, "ScrollableBox");
    }

    for button_node in [
        "designer_select_button",
        "designer_resize_slot_button",
        "designer_preview_interact_button",
    ] {
        let node = layout
            .nodes
            .get(button_node)
            .unwrap_or_else(|| panic!("missing designer button `{button_node}`"));
        assert_eq!(node.component, "WorkbenchButton");
    }

    for button_node in [
        "header_save_button",
        "header_undo_button",
        "header_redo_button",
    ] {
        let node = layout
            .nodes
            .get(button_node)
            .unwrap_or_else(|| panic!("missing header action `{button_node}`"));
        assert_eq!(node.component, "WorkbenchIconButton");
    }

    for search_node in ["palette_search", "hierarchy_search"] {
        let node = layout
            .nodes
            .get(search_node)
            .unwrap_or_else(|| panic!("missing search input `{search_node}`"));
        assert_eq!(node.component, "WorkbenchSearchInput");
    }

    for tree_row in [
        "hierarchy_root_row",
        "hierarchy_safe_area_row",
        "hierarchy_content_row",
    ] {
        let node = layout
            .nodes
            .get(tree_row)
            .unwrap_or_else(|| panic!("missing hierarchy row `{tree_row}`"));
        assert_eq!(node.component, "WorkbenchTreeRow");
    }

    let designer_canvas = layout
        .nodes
        .get("designer_canvas_panel")
        .expect("designer canvas panel");
    assert_eq!(designer_canvas.component, "CanvasBox");
    assert!(designer_canvas.children.is_empty());
    assert!(
        !layout.nodes.contains_key("designer_sample_grid"),
        "the UI asset canvas must receive the session's real preview projection, not a Blend Space sample grid"
    );
}

#[test]
fn ui_asset_editor_v2_projection_uses_renderable_text_primitives_for_content_regions() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor v2 projection asset");

    for (node_id, component) in [
        ("designer_diagnostic_caption", "WorkbenchCaption"),
        ("emergency_shell_caption", "WorkbenchCaption"),
        ("render_stack_label", "WorkbenchLabel"),
        ("source_info_caption", "WorkbenchCaption"),
        ("source_outline_caption", "WorkbenchCaption"),
        ("mock_workspace_caption", "WorkbenchCaption"),
        ("mock_subjects_caption", "WorkbenchCaption"),
        ("mock_editor_caption", "WorkbenchCaption"),
        ("mock_state_graph_caption", "WorkbenchCaption"),
        ("source_text_caption", "WorkbenchCaption"),
        ("inspector_content_label", "WorkbenchLabel"),
        ("inspector_widget_caption", "WorkbenchCaption"),
        ("inspector_promote_caption", "WorkbenchCaption"),
        ("inspector_slot_caption", "WorkbenchCaption"),
        ("inspector_layout_caption", "WorkbenchCaption"),
        ("inspector_binding_caption", "WorkbenchCaption"),
        ("stylesheet_action_caption", "WorkbenchCaption"),
        ("stylesheet_state_primary_caption", "WorkbenchCaption"),
        ("stylesheet_state_secondary_caption", "WorkbenchCaption"),
        ("stylesheet_content_label", "WorkbenchLabel"),
        ("stylesheet_theme_caption", "WorkbenchCaption"),
        ("stylesheet_authoring_caption", "WorkbenchCaption"),
        ("stylesheet_matched_rule_caption", "WorkbenchCaption"),
    ] {
        let node = layout
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("missing renderable content node `{node_id}`"));
        assert_eq!(node.component, component);
        assert!(
            node.props
                .get("text")
                .and_then(toml::Value::as_str)
                .is_some_and(|text| !text.is_empty()),
            "{node_id} must own visible text"
        );
        assert_eq!(
            node.props
                .get("text_overflow")
                .and_then(toml::Value::as_str),
            Some("ellipsis"),
            "{node_id} must ellipsize rather than overflow a narrow pane"
        );
    }

    for container_id in [
        "designer_diagnostic_overlay_panel",
        "emergency_shell_panel",
        "render_stack_panel",
        "source_info_panel",
        "source_outline_panel",
        "mock_workspace_panel",
        "mock_subjects_panel",
        "mock_editor_panel",
        "mock_state_graph_panel",
        "source_text_panel",
        "inspector_content_panel",
        "inspector_widget_section",
        "inspector_promote_section",
        "inspector_slot_section",
        "inspector_layout_section",
        "inspector_binding_section",
        "stylesheet_action_row",
        "stylesheet_state_primary_row",
        "stylesheet_state_secondary_row",
        "stylesheet_content_panel",
        "stylesheet_theme_section",
        "stylesheet_authoring_section",
        "stylesheet_matched_rule_section",
    ] {
        let node = layout
            .nodes
            .get(container_id)
            .unwrap_or_else(|| panic!("missing content container `{container_id}`"));
        assert!(
            !node.props.contains_key("text"),
            "{container_id} must compose a text primitive instead of storing non-renderable text"
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
        .replace("Prototype cache valid", "V2 authoring cache");

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
        "res://ui/editor/imported_card_view.zui",
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
        .register_v2_widget_import("res://ui/editor/test_card.zui", component)
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
    assert!(canonical.contains("res://ui/editor/test_card.zui#Card"));
    assert!(canonical.contains("[nodes.body_text]"));
    assert!(!canonical.contains("[nodes.card_root]"));
}

#[test]
fn ui_asset_editor_v2_component_asset_opens_as_editable_component_tree() {
    let route = UiAssetEditorRoute::new(
        "res://ui/editor/test_card.zui",
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
        "res://ui/editor/test_card.zui",
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
        "res://ui/editor/missing_component_root.zui",
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
        "res://ui/editor/cyclic_component_root.zui",
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
    for token in [
        "accent = \"$editor.accent\"",
        "muted = \"$editor.text.secondary\"",
        "outline = \"$editor.border\"",
        "panel = \"$editor.surface.1\"",
        "panel_inset = \"$editor.surface.recessed\"",
        "surface = \"$editor.surface.0\"",
        "text = \"$editor.text.primary\"",
    ] {
        assert!(
            pane.style_token_items.contains(&token.to_string()),
            "the bootstrap style session must expose its Workbench token alias `{token}`"
        );
    }
    assert!(
        pane.style_rule_items.is_empty(),
        "the Workbench token asset must not retain Material-specific local rules"
    );
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
