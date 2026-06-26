use std::fs;

use crate::asset::assets::{ui_asset_references, ui_v2_asset_references};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImporter, AssetKind, AssetUri, ImportedAsset, UiIconAsset, UiIconSource, UiIconSourceKind,
    UiLayoutAsset, UiStyleAsset, UiThemeAsset, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset,
    UiWidgetAsset,
};
use zircon_runtime_interface::resource::{ResourceKind, ResourceMarker};
use zircon_runtime_interface::ui::style::UiRgbaColor;
use zircon_runtime_interface::ui::template::UiAssetKind;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

const LAYOUT_UI_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 1
display_name = "UI Asset Editor"

[imports]
widgets = ["res://ui/common/button.ui.toml#ToolbarButton"]
styles = ["res://ui/theme/editor.ui.toml"]

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
classes = []
bindings = []
children = []

[root.params]

[root.props]

[root.style_overrides.self]

[root.style_overrides.slot]

[components]
"#;

const WIDGET_UI_TOML: &str = r#"
[asset]
kind = "widget"
id = "ui.common.button"
version = 1
display_name = "Toolbar Button"

[root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = []
bindings = []
children = []

[root.params]

[root.props]

[root.style_overrides.self]

[root.style_overrides.slot]

[components.ToolbarButton]
style_scope = "closed"

[components.ToolbarButton.root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = []
bindings = []
children = []

[components.ToolbarButton.root.params]

[components.ToolbarButton.root.props]

[components.ToolbarButton.root.style_overrides.self]

[components.ToolbarButton.root.style_overrides.slot]

[components.ToolbarButton.params]

[components.ToolbarButton.slots]
"#;

const STYLE_UI_TOML: &str = r#"
[asset]
kind = "style"
id = "ui.theme.editor"
version = 1
display_name = "Editor Theme"

[imports]
widgets = []
styles = []

[tokens]

[components]

[[stylesheets]]
id = "editor"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Styled" } }
"#;

const RESOURCE_REFERENCE_UI_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.resource_reference_graph"
version = 3
display_name = "Resource Reference Graph"

[imports]
widgets = ["res://ui/common/button.ui.toml#ToolbarButton"]
styles = ["res://ui/theme/editor.ui.toml"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
  { kind = "image", uri = "res://textures/logo.png", fallback = { mode = "optional" } },
]

[tokens]
hero_icon = "res://textures/logo.png"

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { icon = "res://textures/root-icon.png" }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = "Label"
set = { self = { background_image = "res://textures/theme-bg.png" } }
"#;

const V2_VIEW_UI_TOML: &str = r#"
[asset]
kind = "view"
id = "runtime.ui.panel"
version = 2
display_name = "Runtime Panel"

[imports]
widgets = ["res://ui/common/button.v2.ui.toml#ToolbarButton"]
styles = ["res://ui/theme/editor_material.v2.ui.toml"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
]

[root]
node = "root"

[nodes.root]
component = "Text"
control_id = "PanelRoot"
props = { text = "Panel" }
"#;

const V2_COMPONENT_UI_TOML: &str = r#"
[asset]
kind = "component"
id = "runtime.ui.components"
version = 2
display_name = "Runtime Components"

[nodes.button_root]
component = "Button"
control_id = "ToolbarButtonRoot"
props = { text = "Action" }

[components.ToolbarButton]
root = "button_root"
"#;

const V2_STYLE_UI_TOML: &str = r##"
[asset]
kind = "style"
id = "runtime.ui.material"
version = 2
display_name = "Runtime Material"

[[stylesheets]]
id = "runtime_material"

[[stylesheets.rules]]
selector = "Text"
set = { foreground = { color = "#ffffff" } }
"##;

const THEME_UI_TOML: &str = r#"
id = "zircon.test.dark"

[palette]
accent = { red = 0.1, green = 0.2, blue = 0.3, alpha = 1.0 }

[[typography]]
variant = "body"
family = "Inter"
size = 13.0
weight = 400
line_height = 1.45
"#;

const ICON_UI_TOML: &str = r##"
semantic_id = "icons/run"
default_size = 18.0

[source]
kind = "svg_asset"
uri = "res://ui/icons/run.svg"
"##;

mod fixture_validation;
mod importer;
mod project_manager;
mod references;
mod wrappers;

fn importer_with_first_wave_plugin_fixtures() -> AssetImporter {
    let mut importer = AssetImporter::default();
    importer
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    importer
}

fn legacy_v2_component_toml() -> &'static str {
    r#"
[asset]
kind = "component"
id = "legacy.component"
version = 2
display_name = "Legacy Component"

[components.ToolbarButton]
root = "button_root"

[nodes.button_root]
component = "Button"
control_id = "ToolbarButtonRoot"
props = { text = "Action" }
"#
}
