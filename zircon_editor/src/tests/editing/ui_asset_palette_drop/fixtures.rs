pub(super) const GRID_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.grid_drop"
version = 1
display_name = "Grid Drop Layout"

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

pub(super) const OVERLAY_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.overlay_drop"
version = 1
display_name = "Overlay Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Overlay"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "Overlay" } }
"##;

pub(super) const FLOW_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.flow_drop"
version = 1
display_name = "Flow Drop Layout"

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

pub(super) const LOCAL_COMPONENT_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.local_component_drop"
version = 1
display_name = "Local Component Drop Layout"

[root]
node = "card"

[components.CardShell]
root = "card_root"

[components.CardShell.slots.header]
required = false
multiple = true

[components.CardShell.slots.body]
required = false
multiple = true

[nodes.card]
kind = "component"
component = "CardShell"
control_id = "CardHost"

[nodes.card_root]
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "header_slot" }, { child = "body_slot" }]

[nodes.header_slot]
kind = "slot"
slot_name = "header"

[nodes.body_slot]
kind = "slot"
slot_name = "body"
"##;

pub(super) const EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.external_widget_drop"
version = 1
display_name = "External Widget Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "asset://ui/common/toolbar_shell.ui#ToolbarShell"
control_id = "ToolbarHost"
"##;

pub(super) const IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar_shell"
version = 1
display_name = "Toolbar Shell"

[root]
node = "toolbar_root"

[components.ToolbarShell]
root = "toolbar_root"

[components.ToolbarShell.slots.leading]
required = false
multiple = true

[components.ToolbarShell.slots.trailing]
required = false
multiple = true

[nodes.toolbar_root]
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 8.0 } }
children = [{ child = "leading_slot" }, { child = "trailing_slot" }]

[nodes.leading_slot]
kind = "slot"
slot_name = "leading"

[nodes.trailing_slot]
kind = "slot"
slot_name = "trailing"
"##;

pub(super) const LOW_SEMANTIC_COMPONENT_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.low_semantic_component_drop"
version = 1
display_name = "Low Semantic Component Drop Layout"

[root]
node = "host"

[components.ThreeSlotShell]
root = "three_slot_root"

[components.ThreeSlotShell.slots.slot_a]
required = false
multiple = true

[components.ThreeSlotShell.slots.slot_b]
required = false
multiple = true

[components.ThreeSlotShell.slots.slot_c]
required = false
multiple = true

[nodes.host]
kind = "component"
component = "ThreeSlotShell"
control_id = "ThreeSlotHost"

[nodes.three_slot_root]
kind = "native"
type = "HorizontalBox"
control_id = "ThreeSlotRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 6.0 } }
children = [{ child = "slot_a_mount" }, { child = "slot_b_mount" }, { child = "slot_c_mount" }]

[nodes.slot_a_mount]
kind = "slot"
slot_name = "slot_a"

[nodes.slot_b_mount]
kind = "slot"
slot_name = "slot_b"

[nodes.slot_c_mount]
kind = "slot"
slot_name = "slot_c"
"##;
