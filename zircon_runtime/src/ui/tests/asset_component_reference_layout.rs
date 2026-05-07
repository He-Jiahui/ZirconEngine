use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::UiTreeId,
    layout::{UiFrame, UiSize},
};

const TOOLBAR_ICON_WIDGET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar_icon"
version = 1
display_name = "Toolbar Icon"

[components.ToolbarIcon]
style_scope = "open"

[components.ToolbarIcon.params.text]
type = "string"
default = ""

[components.ToolbarIcon.params.height]
type = "number"
default = 40.0

[components.ToolbarIcon.root]
node_id = "toolbar_icon_root"
kind = "native"
type = "IconButton"
props = { text = "$param.text", input_interactive = true, input_clickable = true, input_hoverable = true, input_focusable = true, layout_min_width = 40.0, layout_min_height = "$param.height" }
layout = { width = { stretch = "Stretch" }, height = { min = "$param.height" } }
"##;

const TOOLBAR_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.viewport.toolbar"
version = 1
display_name = "Viewport Toolbar"

[imports]
widgets = ["asset://ui/common/toolbar_icon.ui#ToolbarIcon"]

[root]
node_id = "toolbar_root"
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }

[[root.children]]
[root.children.node]
node_id = "move_tool"
kind = "reference"
component_ref = "asset://ui/common/toolbar_icon.ui#ToolbarIcon"
control_id = "MoveTool"
params = { text = "Move", height = 20.0 }
layout = { width = { min = 72.0, preferred = 72.0, max = 72.0, stretch = "Fixed" }, height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } }
"##;

const TOOLBAR_ACTION_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.viewport.toolbar_action"
version = 1
display_name = "Viewport Toolbar Action"

[imports]
widgets = ["asset://ui/common/toolbar_icon.ui#ToolbarIcon"]

[root]
node_id = "move_tool"
kind = "reference"
component_ref = "asset://ui/common/toolbar_icon.ui#ToolbarIcon"
control_id = "MoveTool"
params = { text = "Move", height = 20.0 }

[[root.bindings]]
id = "MoveTool/onClick"
event = "Click"
route = "Toolbar.Move"

[root.bindings.action]
route = "Toolbar.Move"
action = "ActivateTool"

[root.bindings.action.payload]
tool = "move"
"##;

const ROLE_TOKEN_WIDGET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.tests.role_button"
version = 1
display_name = "Role Button"

[tokens]
material_radius_control = 5.0
material_button_radius = "$material_radius_control"
material_density_default_control_height = 40.0
material_control_height = "$material_density_default_control_height"

[components.RoleButton]
style_scope = "open"

[components.RoleButton.params.text]
type = "string"
default = "Role"

[components.RoleButton.root]
node_id = "role_button_root"
kind = "native"
type = "Button"
classes = ["material-button"]
props = { text = "$param.text", corner_radius = "$material_button_radius", layout_min_height = "$material_control_height" }
layout = { width = { stretch = "Stretch" }, height = { min = "$material_control_height" } }
"##;

const ROLE_TOKEN_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.role_style"
version = 1
display_name = "Role Style"

[tokens]
material_accent = "#4c7dd5"
material_color_primary = "$material_accent"
material_border_width_hairline = 1.0
material_focus_ring_width = "$material_border_width_hairline"
material_radius_control = 5.0
material_button_radius = "$material_radius_control"
material_font_size_body = 12.0

[[stylesheets]]
id = "role_style"

[[stylesheets.rules]]
selector = ".material-button"
set = { self = { background = { color = "$material_color_primary" }, border = { width = "$material_focus_ring_width", radius = "$material_button_radius" }, font = { size = "$material_font_size_body" } } }
"##;

const ROLE_TOKEN_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.role_layout"
version = 1
display_name = "Role Layout"

[imports]
widgets = ["asset://ui/tests/role_button.ui#RoleButton"]
styles = ["asset://ui/tests/role_style.ui"]

[root]
node_id = "role_button"
kind = "reference"
component_ref = "asset://ui/tests/role_button.ui#RoleButton"
control_id = "RoleButton"
"##;

#[test]
fn ui_document_compiler_applies_reference_instance_layout_to_expanded_root() {
    let widget = UiAssetLoader::load_toml_str(TOOLBAR_ICON_WIDGET_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(TOOLBAR_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/toolbar_icon.ui#ToolbarIcon", widget)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reference.layout"),
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(200.0, 20.0)).unwrap();

    let move_tool = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MoveTool")
        })
        .expect("reference instance should expand to a real toolbar icon node");

    assert_eq!(
        move_tool.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 72.0, 20.0)
    );
    assert_eq!(
        move_tool
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("layout")
            .and_then(|layout| layout.get("width"))
            .and_then(|width| width.get("preferred"))
            .and_then(toml::Value::as_float),
        Some(72.0)
    );
}

#[test]
fn ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root() {
    let widget = UiAssetLoader::load_toml_str(TOOLBAR_ICON_WIDGET_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(TOOLBAR_ACTION_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/toolbar_icon.ui#ToolbarIcon", widget)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(root.control_id.as_deref(), Some("MoveTool"));
    assert_eq!(root.bindings.len(), 1);
    let binding = &root.bindings[0];
    assert_eq!(binding.id, "MoveTool/onClick");
    assert_eq!(binding.event, UiEventKind::Click);
    assert_eq!(binding.route.as_deref(), Some("Toolbar.Move"));
    let action = binding
        .action
        .as_ref()
        .expect("reference instance callback action should survive root expansion");
    assert_eq!(action.route.as_deref(), Some("Toolbar.Move"));
    assert_eq!(action.action.as_deref(), Some("ActivateTool"));
    assert_eq!(
        action.payload.get("tool").and_then(toml::Value::as_str),
        Some("move")
    );
}

#[test]
fn ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles() {
    let widget = UiAssetLoader::load_toml_str(ROLE_TOKEN_WIDGET_TOML).unwrap();
    let style = UiAssetLoader::load_toml_str(ROLE_TOKEN_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(ROLE_TOKEN_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/tests/role_button.ui#RoleButton", widget)
        .unwrap()
        .register_style_import("asset://ui/tests/role_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(
        root.attributes
            .get("corner_radius")
            .and_then(toml::Value::as_float),
        Some(5.0),
        "component props should resolve nested Material role token aliases"
    );
    assert_eq!(
        root.attributes
            .get("layout_min_height")
            .and_then(toml::Value::as_float),
        Some(40.0),
        "layout metric props should resolve density role aliases"
    );
    assert_eq!(
        root.attributes
            .get("background")
            .and_then(|background| background.get("color"))
            .and_then(toml::Value::as_str),
        Some("#4c7dd5"),
        "style rules should resolve palette role aliases"
    );
    assert_eq!(
        root.attributes
            .get("border")
            .and_then(|border| border.get("width"))
            .and_then(toml::Value::as_float),
        Some(1.0),
        "style rules should resolve focus/border width role aliases"
    );
    assert_eq!(
        root.attributes
            .get("border")
            .and_then(|border| border.get("radius"))
            .and_then(toml::Value::as_float),
        Some(5.0),
        "style rules should resolve radius role aliases"
    );
    assert_eq!(
        root.attributes
            .get("font")
            .and_then(|font| font.get("size"))
            .and_then(toml::Value::as_float),
        Some(12.0),
        "style rules should resolve typography role aliases"
    );
}
