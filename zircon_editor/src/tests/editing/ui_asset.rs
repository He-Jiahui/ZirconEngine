use crate::core::editing::ui_asset::{
    UiAssetEditorCommand, UiAssetEditorExternalEffect, UiAssetEditorInverseTreeEdit,
    UiAssetEditorSession, UiAssetEditorSessionError, UiAssetEditorTreeEdit,
    UiAssetEditorTreeEditKind, UiAssetEditorUndoExternalEffects,
};
use crate::{
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorUndoStack, UiAssetPreviewPreset,
    UiDesignerSelectionModel, UiSize,
};
use zircon_runtime::ui::{template::UiAssetKind, template::UiAssetLoader};

const SIMPLE_LAYOUT_ASSET_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.test.layout"
version = 1
display_name = "Test Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }
layout = { width = { stretch = "Stretch" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" } }
"#;

const STYLED_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.styled_layout"
version = 1
display_name = "Styled Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
classes = ["shell"]
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = "VerticalBox > Button.primary"
set = { self.text = { color = "#ffffff" } }
"##;

const STYLE_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.style_authoring"
version = 1
display_name = "Style Authoring Layout"

[tokens]
accent = "#4488ff"
panel_gap = 12

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
style_overrides = { self = { text = { color = "#ffffff" } }, slot = { padding = 4 } }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = ".primary:hover"
set = { self.text = { color = "#ffeeaa" } }

[[stylesheets.rules]]
selector = ".primary:disabled"
set = { self.background = { color = "#444444" } }
"##;

const MOCK_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.mock_preview"
version = 1
display_name = "Mock Preview Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", checked = false, mode = "Full", icon = "asset://ui/icons/save.png" }
"##;

const CROSS_NODE_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.cross_node_preview"
version = 1
display_name = "Cross Node Preview Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", checked = false, mode = "Full", icon = "asset://ui/icons/save.png" }
"##;

const RICH_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.rich_preview"
version = 1
display_name = "Rich Preview Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", checked = false, count = 3, mode = "Full", icon = "asset://ui/icons/save.png", items = ["Save", "Publish"], metadata = { state = "Ready", enabled = true }, text_expr = "=preview.save_label" }
"##;

const TREE_REPARENT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.tree_reparent"
version = 1
display_name = "Tree Reparent Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "group_a" }, { child = "loose" }, { child = "group_b" }]

[nodes.group_a]
kind = "native"
type = "VerticalBox"
control_id = "GroupA"
children = [{ child = "nested_a" }]

[nodes.nested_a]
kind = "native"
type = "Label"
control_id = "NestedA"
props = { text = "Nested A" }

[nodes.loose]
kind = "native"
type = "Button"
control_id = "LooseButton"
props = { text = "Loose" }

[nodes.group_b]
kind = "native"
type = "VerticalBox"
control_id = "GroupB"
children = [{ child = "nested_b" }]

[nodes.nested_b]
kind = "native"
type = "Label"
control_id = "NestedB"
props = { text = "Nested B" }
"##;

const SLOT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.slot_authoring"
version = 1
display_name = "Slot Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button", mount = "actions", slot = { padding = 8, layout = { width = { preferred = 180 }, height = { preferred = 32 } } } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
"##;

const LAYOUT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.layout_authoring"
version = 1
display_name = "Layout Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
layout = { width = { preferred = 220 }, height = { preferred = 48 } }
"##;

const OVERLAY_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.overlay_slot"
version = 1
display_name = "Overlay Slot Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Overlay"
control_id = "Root"
children = [{ child = "badge", slot = { layout = { anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -16.0, y = 12.0 }, z_index = 4 } } }]

[nodes.badge]
kind = "native"
type = "Label"
control_id = "Badge"
props = { text = "New" }
"##;

const GRID_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.grid_slot"
version = 1
display_name = "Grid Slot Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "GridBox"
control_id = "Root"
children = [{ child = "button", slot = { row = 1, column = 2, row_span = 2, column_span = 3 } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "Button"
props = { text = "Grid" }
"##;

const FLOW_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.flow_slot"
version = 1
display_name = "Flow Slot Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "FlowBox"
control_id = "Root"
children = [{ child = "button", slot = { break_before = true, alignment = "Center" } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "Button"
props = { text = "Flow" }
"##;

const SCROLLABLE_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.scrollable_layout"
version = 1
display_name = "Scrollable Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "ScrollableBox"
control_id = "Root"
layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 6, scrollbar_visibility = "Always", virtualization = { item_extent = 28, overscan = 2 } }, clip = true }
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "Button"
props = { text = "Scroll" }
"##;

const HORIZONTAL_BOX_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.horizontal_box_layout"
version = 1
display_name = "Horizontal Box Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "HorizontalBox"
control_id = "Root"
layout = { container = { kind = "HorizontalBox", gap = 10 } }
children = [{ child = "left" }, { child = "right" }]

[nodes.left]
kind = "native"
type = "Label"
control_id = "Left"
props = { text = "Left" }

[nodes.right]
kind = "native"
type = "Label"
control_id = "Right"
props = { text = "Right" }
"##;

const VERTICAL_BOX_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.vertical_box_layout"
version = 1
display_name = "Vertical Box Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { container = { kind = "VerticalBox", gap = 12 } }
children = [{ child = "top" }, { child = "bottom" }]

[nodes.top]
kind = "native"
type = "Label"
control_id = "Top"
props = { text = "Top" }

[nodes.bottom]
kind = "native"
type = "Label"
control_id = "Bottom"
props = { text = "Bottom" }
"##;

const HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.horizontal_linear_slot_layout"
version = 1
display_name = "Horizontal Linear Slot Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "HorizontalBox"
control_id = "Root"
layout = { container = { kind = "HorizontalBox", gap = 10 } }
children = [{ child = "fill", slot = { layout = { width = { preferred = 120, weight = 3, stretch = "Stretch" }, height = { preferred = 40, weight = 2, stretch = "Fixed" } } } }]

[nodes.fill]
kind = "native"
type = "Label"
control_id = "Fill"
props = { text = "Fill" }
"##;

const VERTICAL_LINEAR_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.vertical_linear_slot_layout"
version = 1
display_name = "Vertical Linear Slot Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { container = { kind = "VerticalBox", gap = 12 } }
children = [{ child = "fill", slot = { layout = { width = { preferred = 88, weight = 4, stretch = "Stretch" }, height = { preferred = 64, weight = 5, stretch = "Fixed" } } } }]

[nodes.fill]
kind = "native"
type = "Label"
control_id = "Fill"
props = { text = "Fill" }
"##;

const BINDING_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.binding_authoring"
version = 1
display_name = "Binding Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject" }]
"##;

const STRUCTURED_BINDING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.structured_binding_authoring"
version = 1
display_name = "Structured Binding Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject", action = { route = "MenuAction.SaveProject", payload = { confirm = true, mode = "full" } } }]
"##;

const IMPORTED_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.confirm_button"
version = 1
display_name = "Confirm Button"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "ConfirmButton"
props = { text = "Confirm" }
"##;

const PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar_button"
version = 1
display_name = "Toolbar Button"

[root]
node = "button_root"

[components.ToolbarButton]
root = "button_root"

[components.ToolbarButton.params.text]
type = "string"
default = "Toolbar"

[nodes.button_root]
kind = "native"
type = "Button"
control_id = "ToolbarButton"
props = { text = "$param.text" }
"##;

const REFERENCE_SELECTION_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.reference_selection"
version = 1
display_name = "Reference Selection Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.ui.toml#ToolbarButton"
control_id = "ToolbarHost"
"##;

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

#[test]
fn ui_asset_editor_session_creates_stylesheet_rule_from_selected_node() {
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
    assert!(session.create_rule_from_selection().expect("create rule"));

    let reflection = session.reflection_model();
    assert!(reflection.source_dirty);
    assert!(reflection
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == "#SaveButton"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let created = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("created rule");
    assert_eq!(created.selector, "#SaveButton");
    assert!(created.set.self_values.is_empty());
    assert!(created.set.slot.is_empty());
}

#[test]
fn ui_asset_editor_session_extracts_inline_overrides_into_stylesheet_rule() {
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
        .extract_inline_overrides_to_rule()
        .expect("extract inline overrides"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert!(button.style_overrides.self_values.is_empty());
    assert!(button.style_overrides.slot.is_empty());

    let extracted = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("extracted rule");
    assert_eq!(extracted.selector, "#SaveButton");
    assert_eq!(
        extracted
            .set
            .self_values
            .get("text")
            .and_then(|value| value.get("color"))
            .and_then(toml::Value::as_str),
        Some("#ffffff")
    );
    assert_eq!(
        extracted
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(4)
    );
}

#[test]
fn ui_asset_editor_session_adds_and_removes_selection_classes() {
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
        .add_class_to_selection("toolbar")
        .expect("add toolbar class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["primary".to_string(), "toolbar".to_string()]
    );
    assert!(session.reflection_model().source_dirty);
    assert!(!session
        .add_class_to_selection("toolbar")
        .expect("duplicate add should no-op"));

    assert!(session
        .remove_class_from_selection("primary")
        .expect("remove primary class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["toolbar".to_string()]
    );
    assert!(!session
        .remove_class_from_selection("missing")
        .expect("missing remove should no-op"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.classes, vec!["toolbar".to_string()]);
}

#[test]
fn ui_asset_editor_session_selects_renames_and_deletes_local_stylesheet_rules() {
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
    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_items,
        vec![
            ".primary".to_string(),
            ".primary:hover".to_string(),
            ".primary:disabled".to_string()
        ]
    );
    assert_eq!(initial_pane.style_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_selector, "");

    assert!(session
        .select_stylesheet_rule(1)
        .expect("select local stylesheet rule"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_selected_index, 1);
    assert_eq!(selected.style_selected_rule_selector, ".primary:hover");

    assert!(session
        .rename_selected_stylesheet_rule("Button.toolbar:hover")
        .expect("rename selected stylesheet rule"));
    let renamed_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        renamed_document.stylesheets[0].rules[1].selector,
        "Button.toolbar:hover"
    );
    let renamed = session.pane_presentation();
    assert_eq!(renamed.style_rule_selected_index, 1);
    assert_eq!(
        renamed.style_selected_rule_selector,
        "Button.toolbar:hover".to_string()
    );

    assert!(session
        .delete_selected_stylesheet_rule()
        .expect("delete selected stylesheet rule"));
    let after_delete =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let selectors = after_delete.stylesheets[0]
        .rules
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        selectors,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    let deleted = session.pane_presentation();
    assert_eq!(
        deleted.style_rule_items,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    assert_eq!(deleted.style_rule_selected_index, 1);
    assert_eq!(
        deleted.style_selected_rule_selector,
        ".primary:disabled".to_string()
    );
}

#[test]
fn ui_asset_editor_session_selects_upserts_and_deletes_stylesheet_rule_declarations() {
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
    session
        .select_stylesheet_rule(0)
        .expect("select local stylesheet rule");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_declaration_items,
        vec!["self.background.color = \"#224488\"".to_string()]
    );
    assert_eq!(initial_pane.style_rule_declaration_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_declaration_path, "");
    assert_eq!(initial_pane.style_selected_rule_declaration_value, "");

    assert!(session
        .select_stylesheet_rule_declaration(0)
        .expect("select style declaration"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_declaration_selected_index, 0);
    assert_eq!(
        selected.style_selected_rule_declaration_path,
        "self.background.color"
    );
    assert_eq!(
        selected.style_selected_rule_declaration_value,
        "\"#224488\""
    );

    assert!(session
        .upsert_selected_stylesheet_rule_declaration("slot.padding", "6")
        .expect("rename style declaration"));
    let updated_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let updated_rule = &updated_document.stylesheets[0].rules[0];
    assert!(updated_rule.set.self_values.is_empty());
    assert_eq!(
        updated_rule
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(6)
    );
    let updated = session.pane_presentation();
    assert_eq!(
        updated.style_rule_declaration_items,
        vec!["slot.padding = 6".to_string()]
    );
    assert_eq!(updated.style_rule_declaration_selected_index, 0);
    assert_eq!(updated.style_selected_rule_declaration_path, "slot.padding");
    assert_eq!(updated.style_selected_rule_declaration_value, "6");

    assert!(session
        .delete_selected_stylesheet_rule_declaration()
        .expect("delete style declaration"));
    let deleted_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let deleted_rule = &deleted_document.stylesheets[0].rules[0];
    assert!(deleted_rule.set.self_values.is_empty());
    assert!(deleted_rule.set.slot.is_empty());
    let deleted = session.pane_presentation();
    assert!(deleted.style_rule_declaration_items.is_empty());
    assert_eq!(deleted.style_rule_declaration_selected_index, -1);
    assert_eq!(deleted.style_selected_rule_declaration_path, "");
    assert_eq!(deleted.style_selected_rule_declaration_value, "");
}

#[test]
fn ui_asset_editor_session_upserts_and_deletes_local_tokens() {
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

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string()
        ]
    );
    assert_eq!(initial_pane.style_token_selected_index, -1);
    assert_eq!(initial_pane.style_selected_token_name, "");
    assert_eq!(initial_pane.style_selected_token_value, "");

    assert!(session
        .upsert_style_token("surface_fill", "#223344")
        .expect("add token"));
    let added = session.pane_presentation();
    assert_eq!(
        added.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
    assert_eq!(added.style_token_selected_index, 2);
    assert_eq!(added.style_selected_token_name, "surface_fill");
    assert_eq!(added.style_selected_token_value, "\"#223344\"");

    assert!(session.select_style_token(0).expect("select accent token"));
    assert!(session
        .upsert_style_token("accent_primary", "#99bbff")
        .expect("rename selected token"));
    let renamed_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(!renamed_document.tokens.contains_key("accent"));
    assert_eq!(
        renamed_document
            .tokens
            .get("accent_primary")
            .and_then(toml::Value::as_str),
        Some("#99bbff")
    );

    assert!(session
        .delete_selected_style_token()
        .expect("delete selected token"));
    let deleted = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(!deleted.tokens.contains_key("accent_primary"));
    assert_eq!(
        session.pane_presentation().style_token_items,
        vec![
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
}

#[test]
fn ui_asset_editor_session_toggles_pseudo_state_preview_matches() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
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
    assert!(!session.reflection_model().source_dirty);
    assert_eq!(
        session
            .reflection_model()
            .style_inspector
            .active_pseudo_states,
        Vec::<String>::new()
    );
    assert!(!session
        .reflection_model()
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview"));
    let hover = session.reflection_model();
    assert_eq!(hover.style_inspector.active_pseudo_states, vec!["hover"]);
    assert!(hover
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("disabled")
        .expect("enable disabled preview"));
    let disabled = session.reflection_model();
    assert_eq!(
        disabled.style_inspector.active_pseudo_states,
        vec!["hover", "disabled"]
    );
    assert!(disabled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:disabled"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("disable hover preview"));
    let toggled = session.reflection_model();
    assert_eq!(
        toggled.style_inspector.active_pseudo_states,
        vec!["disabled"]
    );
    assert!(!toggled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));
    assert!(!toggled.source_dirty);
}

#[test]
fn ui_asset_editor_session_projects_matched_style_rules_into_stylesheet_summary_items() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
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
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let pane = session.pane_presentation();
    assert!(pane
        .stylesheet_items
        .contains(&"selection selector: #SaveButton".to_string()));
    assert!(pane.stylesheet_items.contains(&"states: hover".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary (spec 10, order 0)".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary:hover (spec 20, order 1)".to_string()));
}

#[test]
fn ui_asset_editor_session_selects_matched_style_rules_and_projects_details() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
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
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_matched_rule_items,
        vec![
            ".primary [editor.test.style_authoring::local]".to_string(),
            ".primary:hover [editor.test.style_authoring::local]".to_string()
        ]
    );
    assert_eq!(initial_pane.style_matched_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_origin, "");
    assert_eq!(initial_pane.style_selected_matched_rule_selector, "");
    assert_eq!(initial_pane.style_selected_matched_rule_specificity, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_source_order, -1);
    assert!(initial_pane
        .style_selected_matched_rule_declaration_items
        .is_empty());

    assert!(session
        .select_matched_style_rule(1)
        .expect("select matched style rule"));
    let selected_pane = session.pane_presentation();
    assert_eq!(selected_pane.style_matched_rule_selected_index, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_origin,
        "editor.test.style_authoring::local"
    );
    assert_eq!(
        selected_pane.style_selected_matched_rule_selector,
        ".primary:hover"
    );
    assert_eq!(selected_pane.style_selected_matched_rule_specificity, 20);
    assert_eq!(selected_pane.style_selected_matched_rule_source_order, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_declaration_items,
        vec!["self.text.color = \"#ffeeaa\"".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_widget_inspector_fields() {
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

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.inspector_parent_node_id, "root");
    assert_eq!(pane.inspector_mount, "");
    assert_eq!(pane.inspector_widget_kind, "Native");
    assert_eq!(pane.inspector_widget_label, "Button");
    assert_eq!(pane.inspector_control_id, "SaveButton");
    assert_eq!(pane.inspector_text_prop, "Save");
    assert!(pane.inspector_can_edit_control_id);
    assert!(pane.inspector_can_edit_text_prop);
}

#[test]
fn ui_asset_editor_session_updates_selected_widget_inspector_fields() {
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
        .set_selected_widget_control_id("ConfirmButton")
        .expect("set selected control id"));
    assert!(session
        .set_selected_widget_text_property("Confirm")
        .expect("set selected text property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_control_id, "ConfirmButton");
    assert_eq!(updated.inspector_text_prop, "Confirm");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.control_id.as_deref(), Some("ConfirmButton"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Confirm")
    );
    assert!(preview_has_control_id(
        session.preview_host().surface(),
        "ConfirmButton"
    ));
}

#[test]
fn ui_asset_editor_session_projects_structured_slot_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/slot-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SLOT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_mount, "actions");
    assert_eq!(pane.inspector_slot_padding, "8");
    assert_eq!(pane.inspector_slot_width_preferred, "180");
    assert_eq!(pane.inspector_slot_height_preferred, "32");
}

#[test]
fn ui_asset_editor_session_updates_selected_slot_inspector_fields() {
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
        .set_selected_slot_mount("footer")
        .expect("set selected mount"));
    assert!(session
        .set_selected_slot_padding("12")
        .expect("set selected slot padding"));
    assert!(session
        .set_selected_slot_width_preferred("240")
        .expect("set selected slot width preferred"));
    assert!(session
        .set_selected_slot_height_preferred("44")
        .expect("set selected slot height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_mount, "footer");
    assert_eq!(updated.inspector_slot_padding, "12");
    assert_eq!(updated.inspector_slot_width_preferred, "240");
    assert_eq!(updated.inspector_slot_height_preferred, "44");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let child_mount = document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "button")
        .expect("button child mount");
    assert_eq!(child_mount.mount.as_deref(), Some("footer"));
    assert_eq!(
        slot_value(&child_mount.slot, &["padding"]).and_then(toml::Value::as_integer),
        Some(12)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(240)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(44)
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_layout_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LAYOUT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_layout_width_preferred, "220");
    assert_eq!(pane.inspector_layout_height_preferred, "48");
}

#[test]
fn ui_asset_editor_session_updates_selected_layout_inspector_fields() {
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
        .set_selected_layout_width_preferred("220")
        .expect("set selected layout width preferred"));
    assert!(session
        .set_selected_layout_height_preferred("48")
        .expect("set selected layout height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_layout_width_preferred, "220");
    assert_eq!(updated.inspector_layout_height_preferred, "48");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(
        layout_value(button.layout.as_ref(), &["width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(220)
    );
    assert_eq!(
        layout_value(button.layout.as_ref(), &["height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(48)
    );
}

#[test]
fn ui_asset_editor_session_projects_parent_specific_slot_and_layout_semantics() {
    let overlay_route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut overlay_session = UiAssetEditorSession::from_source(
        overlay_route,
        OVERLAY_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay session");
    overlay_session
        .select_hierarchy_index(1)
        .expect("select overlay child");
    let overlay = overlay_session.pane_presentation();
    assert_eq!(overlay.inspector_slot_kind, "Overlay");
    assert_eq!(overlay.inspector_slot_semantic_title, "Overlay Slot");
    assert_eq!(overlay.inspector_slot_overlay_anchor_x, "1");
    assert_eq!(overlay.inspector_slot_overlay_anchor_y, "0");
    assert_eq!(overlay.inspector_slot_overlay_pivot_x, "1");
    assert_eq!(overlay.inspector_slot_overlay_pivot_y, "0");
    assert_eq!(overlay.inspector_slot_overlay_position_x, "-16");
    assert_eq!(overlay.inspector_slot_overlay_position_y, "12");
    assert_eq!(overlay.inspector_slot_overlay_z_index, "4");
    assert_eq!(
        overlay.inspector_slot_semantic_items,
        vec![
            "layout.anchor.x = 1".to_string(),
            "layout.anchor.y = 0".to_string(),
            "layout.pivot.x = 1".to_string(),
            "layout.pivot.y = 0".to_string(),
            "layout.position.x = -16".to_string(),
            "layout.position.y = 12".to_string(),
            "layout.z_index = 4".to_string()
        ]
    );

    let grid_route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut grid_session = UiAssetEditorSession::from_source(
        grid_route,
        GRID_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid session");
    grid_session
        .select_hierarchy_index(1)
        .expect("select grid child");
    let grid = grid_session.pane_presentation();
    assert_eq!(grid.inspector_slot_kind, "Grid");
    assert_eq!(grid.inspector_slot_semantic_title, "Grid Slot");
    assert_eq!(grid.inspector_slot_grid_row, "1");
    assert_eq!(grid.inspector_slot_grid_column, "2");
    assert_eq!(grid.inspector_slot_grid_row_span, "2");
    assert_eq!(grid.inspector_slot_grid_column_span, "3");
    assert_eq!(
        grid.inspector_slot_semantic_items,
        vec![
            "row = 1".to_string(),
            "column = 2".to_string(),
            "row_span = 2".to_string(),
            "column_span = 3".to_string()
        ]
    );

    let flow_route = UiAssetEditorRoute::new(
        "asset://ui/tests/flow-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut flow_session = UiAssetEditorSession::from_source(
        flow_route,
        FLOW_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("flow session");
    flow_session
        .select_hierarchy_index(1)
        .expect("select flow child");
    let flow = flow_session.pane_presentation();
    assert_eq!(flow.inspector_slot_kind, "Flow");
    assert_eq!(flow.inspector_slot_semantic_title, "Flow Slot");
    assert_eq!(flow.inspector_slot_flow_break_before, "true");
    assert_eq!(flow.inspector_slot_flow_alignment, "\"Center\"");
    assert_eq!(
        flow.inspector_slot_semantic_items,
        vec![
            "break_before = true".to_string(),
            "alignment = \"Center\"".to_string()
        ]
    );

    let scroll_route = UiAssetEditorRoute::new(
        "asset://ui/tests/scrollable-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let scroll_session = UiAssetEditorSession::from_source(
        scroll_route,
        SCROLLABLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("scroll session");
    let scroll = scroll_session.pane_presentation();
    assert_eq!(scroll.inspector_layout_kind, "ScrollableBox");
    assert_eq!(scroll.inspector_layout_semantic_title, "Scrollable Layout");
    assert_eq!(scroll.inspector_layout_scroll_axis, "\"Vertical\"");
    assert_eq!(scroll.inspector_layout_scroll_gap, "6");
    assert_eq!(scroll.inspector_layout_scrollbar_visibility, "\"Always\"");
    assert_eq!(scroll.inspector_layout_virtualization_item_extent, "28");
    assert_eq!(scroll.inspector_layout_virtualization_overscan, "2");
    assert_eq!(scroll.inspector_layout_clip, "true");
    assert_eq!(
        scroll.inspector_layout_semantic_items,
        vec![
            "container.axis = \"Vertical\"".to_string(),
            "container.gap = 6".to_string(),
            "container.scrollbar_visibility = \"Always\"".to_string(),
            "container.virtualization.item_extent = 28".to_string(),
            "container.virtualization.overscan = 2".to_string(),
            "clip = true".to_string()
        ]
    );

    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    let horizontal = horizontal_session.pane_presentation();
    assert_eq!(horizontal.inspector_layout_kind, "HorizontalBox");
    assert_eq!(horizontal.inspector_layout_semantic_title, "Linear Layout");
    assert_eq!(horizontal.inspector_layout_box_gap, "10");
    assert_eq!(
        horizontal.inspector_layout_semantic_items,
        vec!["container.gap = 10".to_string()]
    );

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    let vertical = vertical_session.pane_presentation();
    assert_eq!(vertical.inspector_layout_kind, "VerticalBox");
    assert_eq!(vertical.inspector_layout_semantic_title, "Linear Layout");
    assert_eq!(vertical.inspector_layout_box_gap, "12");
    assert_eq!(
        vertical.inspector_layout_semantic_items,
        vec!["container.gap = 12".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_updates_parent_specific_slot_and_layout_semantics() {
    let overlay_route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut overlay_session = UiAssetEditorSession::from_source(
        overlay_route,
        OVERLAY_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay session");
    overlay_session
        .select_hierarchy_index(1)
        .expect("select overlay child");
    assert!(overlay_session
        .set_selected_slot_semantic_field("layout.pivot.x", "0.5")
        .expect("update overlay pivot x"));

    let overlay_document =
        UiAssetLoader::load_toml_str(overlay_session.source_buffer().text()).expect("document");
    let overlay_mount = overlay_document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "badge")
        .expect("overlay child mount");
    assert_eq!(
        slot_value(&overlay_mount.slot, &["layout", "pivot", "x"]).and_then(toml::Value::as_float),
        Some(0.5)
    );

    let grid_route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut grid_session = UiAssetEditorSession::from_source(
        grid_route,
        GRID_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid session");
    grid_session
        .select_hierarchy_index(1)
        .expect("select grid child");
    assert!(grid_session
        .set_selected_slot_semantic_field("column_span", "4")
        .expect("update grid column span"));
    let grid_document =
        UiAssetLoader::load_toml_str(grid_session.source_buffer().text()).expect("document");
    let grid_mount = grid_document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "button")
        .expect("grid child mount");
    assert_eq!(
        slot_value(&grid_mount.slot, &["column_span"]).and_then(toml::Value::as_integer),
        Some(4)
    );

    let flow_route = UiAssetEditorRoute::new(
        "asset://ui/tests/flow-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut flow_session = UiAssetEditorSession::from_source(
        flow_route,
        FLOW_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("flow session");
    flow_session
        .select_hierarchy_index(1)
        .expect("select flow child");
    assert!(flow_session
        .set_selected_slot_semantic_field("break_before", "false")
        .expect("update flow break before"));
    let flow_document =
        UiAssetLoader::load_toml_str(flow_session.source_buffer().text()).expect("document");
    let flow_mount = flow_document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "button")
        .expect("flow child mount");
    assert_eq!(
        slot_value(&flow_mount.slot, &["break_before"]).and_then(toml::Value::as_bool),
        Some(false)
    );

    let scroll_route = UiAssetEditorRoute::new(
        "asset://ui/tests/scrollable-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut scroll_session = UiAssetEditorSession::from_source(
        scroll_route,
        SCROLLABLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("scroll session");
    assert!(scroll_session
        .set_selected_layout_semantic_field("container.scrollbar_visibility", "\"Auto\"")
        .expect("update scroll scrollbar visibility"));
    let scroll_document =
        UiAssetLoader::load_toml_str(scroll_session.source_buffer().text()).expect("document");
    let scroll_root = scroll_document.nodes.get("root").expect("scroll root");
    assert_eq!(
        layout_value(
            scroll_root.layout.as_ref(),
            &["container", "scrollbar_visibility"]
        )
        .and_then(toml::Value::as_str),
        Some("Auto")
    );

    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    assert!(horizontal_session
        .set_selected_layout_semantic_field("container.gap", "18")
        .expect("update horizontal box gap"));
    let horizontal_document =
        UiAssetLoader::load_toml_str(horizontal_session.source_buffer().text()).expect("document");
    let horizontal_root = horizontal_document
        .nodes
        .get("root")
        .expect("horizontal root");
    assert_eq!(
        layout_value(horizontal_root.layout.as_ref(), &["container", "gap"])
            .and_then(toml::Value::as_integer),
        Some(18)
    );
    assert_eq!(
        horizontal_session
            .pane_presentation()
            .inspector_layout_box_gap,
        "18"
    );

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    assert!(vertical_session
        .set_selected_layout_semantic_field("container.gap", "20")
        .expect("update vertical box gap"));
    let vertical_document =
        UiAssetLoader::load_toml_str(vertical_session.source_buffer().text()).expect("document");
    let vertical_root = vertical_document.nodes.get("root").expect("vertical root");
    assert_eq!(
        layout_value(vertical_root.layout.as_ref(), &["container", "gap"])
            .and_then(toml::Value::as_integer),
        Some(20)
    );
    assert_eq!(
        vertical_session
            .pane_presentation()
            .inspector_layout_box_gap,
        "20"
    );
}

#[test]
fn ui_asset_editor_session_projects_linear_slot_typed_fields() {
    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    horizontal_session
        .select_hierarchy_index(1)
        .expect("select horizontal linear child");

    let horizontal = horizontal_session.pane_presentation();
    assert_eq!(horizontal.inspector_slot_kind, "HorizontalBox");
    assert_eq!(horizontal.inspector_slot_semantic_title, "Linear Slot");
    assert_eq!(horizontal.inspector_slot_linear_main_weight, "3");
    assert_eq!(horizontal.inspector_slot_linear_main_stretch, "Stretch");
    assert_eq!(horizontal.inspector_slot_linear_cross_weight, "2");
    assert_eq!(horizontal.inspector_slot_linear_cross_stretch, "Fixed");

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    vertical_session
        .select_hierarchy_index(1)
        .expect("select vertical linear child");

    let vertical = vertical_session.pane_presentation();
    assert_eq!(vertical.inspector_slot_kind, "VerticalBox");
    assert_eq!(vertical.inspector_slot_semantic_title, "Linear Slot");
    assert_eq!(vertical.inspector_slot_linear_main_weight, "5");
    assert_eq!(vertical.inspector_slot_linear_main_stretch, "Fixed");
    assert_eq!(vertical.inspector_slot_linear_cross_weight, "4");
    assert_eq!(vertical.inspector_slot_linear_cross_stretch, "Stretch");
}

#[test]
fn ui_asset_editor_session_updates_linear_slot_typed_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    session
        .select_hierarchy_index(1)
        .expect("select horizontal linear child");

    assert!(session
        .set_selected_slot_semantic_field("layout.width.weight", "6")
        .expect("update main-axis weight"));
    assert!(session
        .set_selected_slot_semantic_field("layout.height.stretch", "\"Stretch\"")
        .expect("update cross-axis stretch"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_slot_linear_main_weight, "6");
    assert_eq!(updated.inspector_slot_linear_cross_stretch, "Stretch");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let child_mount = document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "fill")
        .expect("fill child mount");
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "weight"])
            .and_then(toml::Value::as_integer),
        Some(6)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "stretch"])
            .and_then(toml::Value::as_str),
        Some("Stretch")
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        BINDING_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> MenuAction.SaveProject".to_string()]
    );
    assert_eq!(pane.inspector_binding_selected_index, 0);
    assert_eq!(pane.inspector_binding_id, "SaveButton/onClick");
    assert_eq!(pane.inspector_binding_event, "onClick");
    assert_eq!(pane.inspector_binding_route, "MenuAction.SaveProject");
    assert_eq!(
        pane.inspector_binding_route_target,
        "MenuAction.SaveProject"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
}

#[test]
fn ui_asset_editor_session_updates_selected_binding_inspector_fields() {
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
    assert!(session.add_binding().expect("add binding"));
    assert!(session
        .set_selected_binding_id("SaveButton/onHover")
        .expect("set selected binding id"));
    assert!(session
        .set_selected_binding_event("onHover")
        .expect("set selected binding event"));
    assert!(session
        .set_selected_binding_route("MenuAction.HighlightSave")
        .expect("set selected binding route"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_selected_index, 0);
    assert_eq!(updated.inspector_binding_id, "SaveButton/onHover");
    assert_eq!(updated.inspector_binding_event, "onHover");
    assert_eq!(updated.inspector_binding_route, "MenuAction.HighlightSave");
    assert_eq!(
        updated.inspector_binding_route_target,
        "MenuAction.HighlightSave"
    );
    assert_eq!(updated.inspector_binding_action_target, "");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("MenuAction.HighlightSave")
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> MenuAction.SaveProject (+2 payload)".to_string()]
    );
    assert_eq!(pane.inspector_binding_event_selected_index, 0);
    assert_eq!(pane.inspector_binding_action_kind_selected_index, 1);
    assert_eq!(
        pane.inspector_binding_action_kind_items,
        vec![
            "None".to_string(),
            "Route".to_string(),
            "Action".to_string()
        ]
    );
    assert_eq!(pane.inspector_binding_route, "MenuAction.SaveProject");
    assert_eq!(
        pane.inspector_binding_route_target,
        "MenuAction.SaveProject"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
    assert_eq!(
        pane.inspector_binding_payload_items,
        vec!["confirm = true".to_string(), "mode = \"full\"".to_string()]
    );
    assert_eq!(pane.inspector_binding_payload_selected_index, 0);
    assert_eq!(pane.inspector_binding_payload_key, "confirm");
    assert_eq!(pane.inspector_binding_payload_value, "true");
}

#[test]
fn ui_asset_editor_session_updates_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .select_binding_event_option(1)
        .expect("select double click event"));
    assert!(session
        .select_binding_action_kind(2)
        .expect("select action kind"));
    assert!(session
        .set_selected_binding_action_target("EditorActions.SaveProject")
        .expect("set action target"));
    assert!(session
        .select_binding_payload(1)
        .expect("select mode payload"));
    assert!(session
        .upsert_selected_binding_payload("mode", "\"compact\"")
        .expect("update payload"));
    assert!(session
        .upsert_selected_binding_payload("channel", "\"toolbar\"")
        .expect("add payload"));
    assert!(session
        .delete_selected_binding_payload()
        .expect("delete selected payload"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_event, "onDoubleClick");
    assert_eq!(updated.inspector_binding_event_selected_index, 1);
    assert_eq!(updated.inspector_binding_action_kind_selected_index, 2);
    assert_eq!(updated.inspector_binding_route, "EditorActions.SaveProject");
    assert_eq!(updated.inspector_binding_route_target, "");
    assert_eq!(
        updated.inspector_binding_action_target,
        "EditorActions.SaveProject"
    );
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"compact\"".to_string()
        ]
    );

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.bindings[0].event.to_string(), "onDoubleClick");
    assert!(button.bindings[0].route.is_none());
    let action = button.bindings[0].action.as_ref().expect("binding action");
    assert_eq!(action.action.as_deref(), Some("EditorActions.SaveProject"));
    assert_eq!(
        action.payload.get("mode").and_then(toml::Value::as_str),
        Some("compact")
    );
    assert!(action.payload.get("channel").is_none());
}

#[test]
fn ui_asset_editor_session_projects_binding_payload_schema_suggestions_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "confirm = true".to_string(),
            "channel = \"toolbar\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(2)
        .expect("apply binding payload suggestion"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"full\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .select_binding_event_option(10)
        .expect("select scroll event"));
    let scroll = session.pane_presentation();
    assert_eq!(
        scroll.inspector_binding_payload_suggestion_items,
        vec![
            "axis = \"vertical\"".to_string(),
            "delta = 1".to_string(),
            "source = \"ui.scroll\"".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_selection_indices_source_summary_and_canvas_frames() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert!(pane.preview_selected_index >= 0);
    assert_eq!(pane.preview_surface_width, 640.0);
    assert_eq!(pane.preview_surface_height, 360.0);
    let selected_canvas_node = pane
        .preview_canvas_items
        .iter()
        .find(|item| item.selected)
        .expect("selected canvas node");
    assert_eq!(selected_canvas_node.node_id, "button");
    assert_eq!(selected_canvas_node.label, "SaveButton");
    assert!(selected_canvas_node.width > 0.0);
    assert!(selected_canvas_node.height > 0.0);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert!(pane.source_selected_line > 0);
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
    assert!(pane.source_roundtrip_status.contains("line"));
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_preview_canvas_projection() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_preview_index = session.pane_presentation().preview_selected_index;
    assert!(selected_preview_index >= 0);

    session
        .select_preview_index(selected_preview_index as usize)
        .expect("select preview node");
    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_outline_projection() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let pane = session.pane_presentation();
    let outline_index = pane.source_outline_selected_index;
    assert!(outline_index >= 0);
    assert!(pane
        .source_outline_items
        .iter()
        .any(|entry| entry.contains("[nodes.button]")));

    session
        .select_source_outline_index(outline_index as usize)
        .expect("select source outline node");
    let roundtripped = session.pane_presentation();
    assert_eq!(roundtripped.inspector_selected_node_id, "button");
    assert_eq!(
        roundtripped.preview_selected_index,
        pane.preview_selected_index
    );
    assert_eq!(roundtripped.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_line_inside_block() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_line = session.pane_presentation().source_selected_line;
    assert!(selected_line > 0);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_line((selected_line + 1) as usize)
        .expect("select node from source line");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_byte_offset_inside_block() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_line = session.pane_presentation().source_selected_line;
    assert!(selected_line > 0);
    let byte_offset =
        byte_offset_for_line(session.source_buffer().text(), (selected_line + 1) as usize);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_source_byte_offset(byte_offset)
        .expect("select node from source byte offset"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_ignores_source_byte_offset_outside_node_blocks() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    assert!(!session
        .select_source_byte_offset(0)
        .expect("offset outside node block should no-op"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_rejects_source_line_outside_node_blocks() {
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

    let error = session
        .select_source_line(1)
        .expect_err("source line outside any node block should fail");

    assert!(
        matches!(
            error,
            UiAssetEditorSessionError::InvalidSelectionIndex { index: 1 }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn ui_asset_editor_session_tracks_source_cursor_line_inside_selected_block() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32);
}

#[test]
fn ui_asset_editor_session_preserves_source_cursor_line_through_valid_source_roundtrip() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");

    let valid_source =
        session
            .source_buffer()
            .text()
            .replacen("[nodes.button]\n", "\n[nodes.button]\n", 1);
    session
        .apply_command(UiAssetEditorCommand::edit_source(valid_source))
        .expect("apply valid source edit");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32 + 1);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32 + 1);
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
}

#[test]
fn ui_asset_editor_session_undo_restores_source_cursor_line_within_selected_block() {
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
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");
    assert_eq!(
        session.pane_presentation().source_selected_line,
        target_line as i32
    );

    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));
    assert!(session
        .insert_selected_palette_item_after_selection()
        .expect("insert button after selection"));
    assert_eq!(
        session.pane_presentation().inspector_selected_node_id,
        "button_2"
    );

    assert!(session.undo().expect("undo tree edit"));
    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32);
}

#[test]
fn ui_asset_editor_session_falls_back_to_last_valid_source_selection_when_source_is_invalid() {
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

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");

    let invalid_source =
        session
            .source_buffer()
            .text()
            .replacen("[nodes.button]", "[nodes.button", 1);
    session
        .apply_command(UiAssetEditorCommand::edit_source(invalid_source))
        .expect("apply invalid source edit");

    let pane = session.pane_presentation();
    let expected_invalid_cursor_offset =
        byte_offset_for_line(session.source_buffer().text(), target_line);
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(
        pane.source_cursor_byte_offset,
        expected_invalid_cursor_offset as i32
    );
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
    assert!(pane.source_roundtrip_status.contains("last valid snapshot"));
    assert!(pane
        .source_outline_items
        .iter()
        .any(|entry| entry.contains("[nodes.button]")));
    assert!(pane.source_outline_selected_index >= 0);
}

fn byte_offset_for_line(source: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    let mut current_line = 1usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            current_line += 1;
            if current_line == line {
                return index + 1;
            }
        }
    }
    source.len()
}

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

    let inserted = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(inserted.nodes.contains_key("button_2"));
    assert_eq!(
        inserted
            .nodes
            .get("button_2")
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );
    assert_eq!(
        inserted
            .nodes
            .get("button_2")
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
    let redone = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(redone.nodes.contains_key("button_2"));
}

#[test]
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
    let inserted = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_node_id = inserted
        .nodes
        .get("root")
        .and_then(|node| node.children.get(1))
        .map(|child| child.child.clone())
        .expect("inserted child");
    assert!(inserted.nodes.contains_key(&inserted_node_id));
    assert_eq!(
        inserted.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.as_str())
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

#[test]
fn ui_asset_editor_undo_stack_replays_document_diffs_for_tree_edits() {
    let mut undo_stack = UiAssetEditorUndoStack::default();
    let before_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_ASSET_TOML).expect("before");
    let after_document =
        UiAssetLoader::load_toml_str(STYLE_AUTHORING_LAYOUT_ASSET_TOML).expect("after");

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
        Some(UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_ASSET_TOML).expect("before")),
        STYLED_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::single("button"),
        Default::default(),
        None,
        Some(UiAssetLoader::load_toml_str(STYLED_LAYOUT_ASSET_TOML).expect("after")),
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
    let imported_widget = UiAssetLoader::load_toml_str(PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML)
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
        Some(UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_ASSET_TOML).expect("before")),
        STYLED_LAYOUT_ASSET_TOML.to_string(),
        UiDesignerSelectionModel::default(),
        Default::default(),
        None,
        Some(UiAssetLoader::load_toml_str(STYLED_LAYOUT_ASSET_TOML).expect("after")),
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
    let imported_widget =
        UiAssetLoader::load_toml_str(IMPORTED_WIDGET_ASSET_TOML).expect("imported widget");
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let reference_node = document
        .nodes
        .values()
        .find(|node| node.kind == zircon_runtime::ui::template::UiNodeDefinitionKind::Reference)
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

    let wrapped = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let wrapper_id = wrapped
        .nodes
        .get("root")
        .and_then(|node| node.children.first())
        .map(|child| child.child.clone())
        .expect("wrapper child");
    assert_ne!(wrapper_id, "button");
    assert_eq!(
        wrapped
            .nodes
            .get(&wrapper_id)
            .and_then(|node| node.widget_type.as_deref()),
        Some("VerticalBox")
    );
    assert_eq!(
        wrapped.nodes.get(&wrapper_id).map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
            .collect::<Vec<_>>()),
        Some(vec!["button".to_string()])
    );

    assert!(session.unwrap_selected_node().expect("unwrap wrapper"));
    let unwrapped = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        unwrapped.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
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

    let previous = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        previous.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
            .collect::<Vec<_>>()),
        Some(vec!["group_a".to_string(), "group_b".to_string()])
    );
    assert_eq!(
        previous.nodes.get("group_a").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
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
    let outdented =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("outdented document");
    assert_eq!(
        outdented.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
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
    let next = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        next.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
            .collect::<Vec<_>>()),
        Some(vec!["group_a".to_string(), "group_b".to_string()])
    );
    assert_eq!(
        next.nodes.get("group_b").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
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
    let imported_widget = UiAssetLoader::load_toml_str(PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML)
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

    let converted =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("converted document");
    let button = converted.nodes.get("button").expect("button node");
    assert_eq!(
        button.kind,
        zircon_runtime::ui::template::UiNodeDefinitionKind::Reference
    );
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
    let undone =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("undone document");
    let button = undone.nodes.get("button").expect("button node");
    assert_eq!(
        button.kind,
        zircon_runtime::ui::template::UiNodeDefinitionKind::Native
    );
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

    let extracted =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("extracted document");
    let component = extracted
        .components
        .get("SaveButton")
        .expect("new local component");
    let instance = extracted.nodes.get("button").expect("component instance");
    assert_eq!(
        instance.kind,
        zircon_runtime::ui::template::UiNodeDefinitionKind::Component
    );
    assert_eq!(instance.component.as_deref(), Some("SaveButton"));
    assert_eq!(instance.control_id.as_deref(), Some("SaveButton"));
    assert_eq!(instance.classes, vec!["primary".to_string()]);
    assert!(instance.params.is_empty());
    assert!(instance.props.is_empty());
    assert!(instance.layout.is_none());
    assert!(instance.bindings.is_empty());
    assert!(instance.children.is_empty());

    let component_root = extracted
        .nodes
        .get(&component.root)
        .expect("extracted component root");
    assert_eq!(
        component_root.kind,
        zircon_runtime::ui::template::UiNodeDefinitionKind::Native
    );
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
    let redone =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("redone document");
    assert_eq!(
        redone
            .nodes
            .get("button")
            .and_then(|node| node.component.as_deref()),
        Some("SaveButton")
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
        promoted_widget.root.as_ref().map(|root| root.node.as_str()),
        Some("savebutton_root")
    );
    assert!(promoted_widget.components.contains_key("SaveButton"));
    assert_eq!(
        promoted_widget
            .nodes
            .get("savebutton_root")
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );

    let promoted =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("promoted document");
    assert!(!promoted.components.contains_key("SaveButton"));
    assert!(!promoted.nodes.contains_key("savebutton_root"));
    assert!(promoted
        .imports
        .widgets
        .iter()
        .any(|reference| { reference == "res://ui/widgets/save_button.ui.toml#SaveButton" }));
    let button = promoted.nodes.get("button").expect("button node");
    assert_eq!(
        button.kind,
        zircon_runtime::ui::template::UiNodeDefinitionKind::Reference
    );
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
    let undone =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("undone document");
    assert_eq!(
        undone
            .nodes
            .get("button")
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
    assert!(promoted_theme.nodes.is_empty());
    assert!(promoted_theme.components.is_empty());

    let promoted =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("promoted document");
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
    let undone =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("undone document");
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

fn selected_text<'a>(
    surface: &'a zircon_runtime::ui::surface::UiSurface,
    control_id: &str,
) -> Option<&'a str> {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("text"))
        .and_then(|value| value.as_str())
}

fn preview_has_control_id(surface: &zircon_runtime::ui::surface::UiSurface, control_id: &str) -> bool {
    surface.tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id)
    })
}

fn slot_value<'a>(
    slot: &'a std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = slot.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(table) = value else {
        return None;
    };
    slot_table_value(table, rest)
}

fn layout_value<'a>(
    layout: Option<&'a std::collections::BTreeMap<String, toml::Value>>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let layout = layout?;
    slot_value(layout, path)
}

fn slot_table_value<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(child) = value else {
        return None;
    };
    slot_table_value(child, rest)
}

#[test]
fn ui_asset_editor_subsystem_is_grouped_by_domain_folders() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("core")
        .join("editing")
        .join("ui_asset");

    for relative in [
        "binding/mod.rs",
        "binding/binding_inspector.rs",
        "preview/mod.rs",
        "preview/preview_host.rs",
        "preview/preview_mock.rs",
        "preview/preview_projection.rs",
        "source/mod.rs",
        "source/source_buffer.rs",
        "source/source_sync.rs",
        "style/mod.rs",
        "style/inspector_fields.rs",
        "style/inspector_semantics.rs",
        "style/matched_rule_inspection.rs",
        "style/style_rule_declarations.rs",
        "tree/mod.rs",
        "tree/tree_editing.rs",
        "tree/drag_drop_policy.rs",
        "tree/palette_drop/mod.rs",
        "tree/palette_drop/resolution.rs",
        "tree/palette_drop/overlay_slots.rs",
        "tree/palette_drop/grid_slots.rs",
        "tree/palette_drop/flow_slots.rs",
        "session/mod.rs",
        "session/ui_asset_editor_session.rs",
        "session/session_state.rs",
        "session/preview_compile.rs",
        "session/style_inspection.rs",
        "session/hierarchy_projection.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected ui asset editor module {relative} under {:?}",
            root
        );
    }
}
