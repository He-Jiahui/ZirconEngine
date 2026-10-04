pub(super) const LOCAL_THEME_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_theme"
version = 1
display_name = "Replay Theme"

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Replay Theme" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "$accent" } }
"##;

pub(super) const STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_style_rules"
version = 1
display_name = "Replay Style Rules"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
id = "primary"
selector = ".primary"
set = { self = { text = "Default" } }

[[stylesheets.rules]]
id = "primary_hover"
selector = ".primary:hover"
set = { self = { text = "Hover" } }

[[stylesheets.rules]]
id = "primary_disabled"
selector = ".primary:disabled"
set = { self = { text = "Disabled" } }
"##;

pub(super) const STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_style_rule_insert"
version = 1
display_name = "Replay Style Rule Insert"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
"##;

pub(super) const WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_widget_promote"
version = 1
display_name = "Replay Widget Promote"

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
"##;

pub(super) const EXISTING_EXTERNAL_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.replay_theme_existing"
version = 1
display_name = "Existing Replay Theme"

[tokens]
accent = "#223344"

[[stylesheets]]
id = "existing_theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "$accent" } }
"##;

pub(super) const EXISTING_EXTERNAL_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.widgets.save_button_existing"
version = 1
display_name = "Existing Save Button"

[root]
node = "button_root"

[components.SaveButton]
root = "button_root"

[nodes.button_root]
kind = "native"
type = "Button"
control_id = "ToolbarButton"
props = { text = "Existing Save" }
"##;

pub(super) const THEME_RULE_VECTOR_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_theme_rule_vector"
version = 1
display_name = "Replay Theme Rule Vector"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme" } }

[[stylesheets.rules]]
selector = "#SaveButton"
set = { self = { text = "Keep Local" } }
"##;

pub(super) const THEME_RULE_VECTOR_IMPORTED_THEME_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme" } }
"##;

pub(super) const BINDING_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_binding_payload"
version = 1
display_name = "Replay Binding Payload"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "menu_action.workbench.project.save" }]
"##;
