use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use toml::Value;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const THEME_SUMMARY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.theme_summary"
version = 1
display_name = "Theme Summary"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Theme Summary" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "Theme Summary Local" } }
"##;

const IMPORTED_THEME_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
border = "#223344"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Imported Theme" } }
"##;

const IMPORTED_THEME_COLLISION_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

const IMPORTED_THEME_MERGE_PREVIEW_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[imports]
styles = ["res://ui/theme/base_tokens.zui"]

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

const IMPORTED_THEME_RULE_DIFF_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme", background.color = "$accent" } }
"##;

const DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_dedupe"
version = 1
display_name = "Theme Dedupe"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#223344"
panel = "$accent"

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
set = { self = { text = "$panel" } }
"##;

const MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_multi_cascade"
version = 1
display_name = "Theme Multi Cascade"

[imports]
styles = [
  "res://ui/theme/shared_a.zui",
  "res://ui/theme/shared_b.zui",
]

[tokens]
accent = "#5588ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "CascadeButton"
props = { text = "Cascade" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Local Theme" } }
"##;

const IMPORTED_THEME_CASCADE_A_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_a"
version = 1
display_name = "Shared Theme A"

[tokens]
accent = "#112233"

[[stylesheets]]
id = "shared_theme_a"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme A" } }
"##;

const IMPORTED_THEME_CASCADE_B_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_b"
version = 1
display_name = "Shared Theme B"

[tokens]
accent = "#334455"

[[stylesheets]]
id = "shared_theme_b"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme B" } }
"##;

mod cascade_refactors;
mod comparison;
mod sources;
