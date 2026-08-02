use toml::Value;

use crate::ui::template::{
    UiAssetDocumentRuntimeExt, UiAssetLoader, UiAssetSchemaMigrator, UiDocumentCompiler,
    UiTemplateSurfaceBuilder,
};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiVisualAssetRef},
    template::{UiAssetError, UiAssetKind},
};

mod component_schema;
mod document_compiler;
mod fixture_migration;
mod loader_validation;
mod style_rule_ids;
mod style_write_apis;

const IMPORTED_BUTTON_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.buttons"
version = 1
display_name = "Common Buttons"

[root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = ["toolbar-button"]
props = { text = "$param.label", icon = "$param.icon" }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }

[components.ToolbarButton]
style_scope = "closed"

[components.ToolbarButton.params.label]
type = "string"
default = "Toolbar"

[components.ToolbarButton.params.icon]
type = "string"
default = "ellipse-outline"

[components.ToolbarButton.root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = ["toolbar-button"]
props = { text = "$param.label", icon = "$param.icon" }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }
"##;

const IMPORTED_TOOLBAR_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar"
version = 1
display_name = "Toolbar Shell"

[root]
node_id = "toolbar_root"
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }

[[root.children]]
[root.children.node]
node_id = "leading_slot"
kind = "slot"
slot_name = "leading"

[components.ToolbarShell]
style_scope = "closed"

[components.ToolbarShell.slots.leading]
required = false
multiple = true

[components.ToolbarShell.root]
node_id = "toolbar_root"
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }

[[components.ToolbarShell.root.children]]
[components.ToolbarShell.root.children.node]
node_id = "leading_slot"
kind = "slot"
slot_name = "leading"
"##;

const IMPORTED_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.editor_base"
version = 1
display_name = "Editor Base"

[tokens]
accent = "#4488ff"
open_text = "Open Styled"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = ".toolbar > Button.primary"
set = { self = { background = { color = "$accent" }, layout = { width = { preferred = 144.0 } } } }

[[stylesheets.rules]]
selector = "#OpenButton"
set = { self = { text = "$open_text" }, slot = { layout = { height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" } } } }
"##;

const LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 2
display_name = "UI Asset Editor"

[imports]
widgets = [
  "asset://ui/common/toolbar.ui#ToolbarShell",
  "asset://ui/common/buttons.ui#ToolbarButton",
]
styles = ["asset://ui/theme/editor_base.ui"]

[tokens]
panel_gap = 12.0

[root]
node_id = "editor_root"
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 12.0 } }

[[root.children]]
[root.children.node]
node_id = "toolbar"
kind = "reference"
component_ref = "asset://ui/common/toolbar.ui#ToolbarShell"
control_id = "ToolbarHost"
classes = ["toolbar"]

[[root.children.node.children]]
mount = "leading"
slot = { layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" } } }
[root.children.node.children.node]
node_id = "open_button"
kind = "reference"
component_ref = "asset://ui/common/buttons.ui#ToolbarButton"
control_id = "OpenButton"
classes = ["primary"]
params = { label = "Open", icon = "folder-open-outline" }
style_overrides = { self = { text = "Open Override" } }

[[root.children.node.children.node.bindings]]
id = "Toolbar/Open"
event = "Click"
route = "Toolbar.Open"
"##;

const FLAT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 2
display_name = "UI Asset Editor"

[imports]
widgets = [
  "asset://ui/common/toolbar.ui#ToolbarShell",
  "asset://ui/common/buttons.ui#ToolbarButton",
]
styles = ["asset://ui/theme/editor_base.ui"]

[tokens]
panel_gap = 12.0

[root]
node = "editor_root"

[nodes.editor_root]
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 12.0 } }
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "asset://ui/common/toolbar.ui#ToolbarShell"
control_id = "ToolbarHost"
classes = ["toolbar"]
children = [{ child = "open_button", mount = "leading", slot = { layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" } } } }]

[nodes.open_button]
kind = "reference"
component_ref = "asset://ui/common/buttons.ui#ToolbarButton"
control_id = "OpenButton"
classes = ["primary"]
params = { label = "Open", icon = "folder-open-outline" }
style_overrides = { self = { text = "Open Override" } }
"##;

const SOURCE_TEMPLATE_WITHOUT_ASSET_HEADER_TOML: &str = r#"
version = 1

[root]
component = "VerticalBox"
control_id = "LegacyRoot"
attributes = { layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } } }
children = [
  { component = "Button", control_id = "LegacyButton", bindings = [{ id = "Legacy/Button", event = "Click", route = "MenuAction.OpenProject" }], attributes = { text = "Open" } }
]
"#;

const STYLE_WITH_RULE_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.rule_ids"
version = 1
display_name = "Rule Ids"

[[stylesheets]]
id = "rule_id_sheet"

[[stylesheets.rules]]
id = "primary_button_hover"
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Label" } }

[[stylesheets]]
id = "secondary_sheet"

[[stylesheets.rules]]
id = "secondary_label_rule"
selector = "Label.secondary"
set = { self = { text = "Secondary" } }
"##;
