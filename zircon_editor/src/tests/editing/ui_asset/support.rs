pub(super) use crate::ui::asset_editor::{
    UiAssetEditorCommand, UiAssetEditorExternalEffect, UiAssetEditorInverseTreeEdit,
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession, UiAssetEditorSessionError,
    UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind, UiAssetEditorUndoExternalEffects,
    UiAssetEditorUndoStack, UiAssetPreviewPreset, UiDesignerSelectionModel,
};
pub(super) use zircon_runtime::ui::layout::UiSize;
pub(super) use zircon_runtime::ui::template::UiAssetKind;
pub(super) const SIMPLE_LAYOUT_ASSET_TOML: &str = r#"
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

pub(super) const STYLED_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const STYLE_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const MOCK_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const CROSS_NODE_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const RICH_PREVIEW_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const TREE_REPARENT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const SLOT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const LAYOUT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const OVERLAY_SLOT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const GRID_SLOT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const FLOW_SLOT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const SCROLLABLE_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const HORIZONTAL_BOX_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const VERTICAL_BOX_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const VERTICAL_LINEAR_SLOT_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const BINDING_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const STRUCTURED_BINDING_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) const IMPORTED_WIDGET_ASSET_TOML: &str = r##"
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

pub(super) const PARAMETERIZED_IMPORTED_WIDGET_ASSET_TOML: &str = r##"
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

pub(super) const REFERENCE_SELECTION_LAYOUT_ASSET_TOML: &str = r##"
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

pub(super) fn byte_offset_for_line(source: &str, line: usize) -> usize {
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

pub(super) fn selected_text<'a>(
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

pub(super) fn preview_has_control_id(
    surface: &zircon_runtime::ui::surface::UiSurface,
    control_id: &str,
) -> bool {
    surface.tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id)
    })
}

pub(super) fn slot_value<'a>(
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

pub(super) fn layout_value<'a>(
    layout: Option<&'a std::collections::BTreeMap<String, toml::Value>>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let layout = layout?;
    slot_value(layout, path)
}

pub(super) fn slot_table_value<'a>(
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
