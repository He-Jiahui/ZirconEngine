use crate::ui::template::{
    UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder, UiTemplateSurfaceBuilder,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::UiTreeId,
    layout::{UiFrame, UiSize},
    template::{parse_component_reference, UiAssetError},
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

#[test]
fn component_reference_parser_requires_one_non_empty_component_fragment() {
    assert_eq!(
        parse_component_reference("asset://ui/common/toolbar_icon.ui#ToolbarIcon").unwrap(),
        ("asset://ui/common/toolbar_icon.ui", "ToolbarIcon")
    );

    for reference in [
        "asset://ui/common/toolbar_icon.ui",
        "#ToolbarIcon",
        "asset://ui/common/toolbar_icon.ui#",
        "asset://ui/common/toolbar_icon.ui#ToolbarIcon#Unexpected",
    ] {
        assert!(matches!(
            parse_component_reference(reference),
            Err(UiAssetError::InvalidDocument { ref detail, .. })
                if detail.contains("component references")
        ));
    }
}

#[test]
fn prototype_store_builder_rejects_component_imports_with_multiple_fragments() {
    let prototype = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "widget"
id = "asset://ui/tests/invalid_imported_component.ui"
version = 3

[imports]
widgets = ["asset://ui/common/toolbar_icon.ui#ToolbarIcon#Unexpected"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Container"
"##,
    )
    .expect("invalid component import fixture should parse before store validation");
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(prototype);

    let error = builder
        .build()
        .expect_err("prototype store should reject an ambiguous component import");
    assert!(matches!(
        error,
        UiAssetError::InvalidDocument { ref detail, .. }
            if detail.contains("exactly one non-empty #Component suffix")
    ));
}

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

const INSTANCE_PROPS_WIDGET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.tests.instance_props_label"
version = 1
display_name = "Instance Props Label"

[components.InstancePropsLabel]
style_scope = "open"

[components.InstancePropsLabel.root]
node_id = "instance_props_label_root"
kind = "native"
type = "Label"
props = { text = "Default", foreground_color = "#aaaaaa", selected = false, layout_padding_left = 2.0 }
layout = { width = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" }, height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } }
"##;

const INSTANCE_PROPS_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.instance_props_layout"
version = 1
display_name = "Instance Props Layout"

[imports]
widgets = ["asset://ui/tests/instance_props_label.ui#InstancePropsLabel"]

[root]
node_id = "label_instance"
kind = "reference"
component_ref = "asset://ui/tests/instance_props_label.ui#InstancePropsLabel"
control_id = "InstancePropsLabel"
props = { text = "Instance", foreground_color = "#ff0000", selected = true }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" } }
"##;

const INSTANCE_STYLE_OVERRIDE_WIDGET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.tests.instance_style_override_label"
version = 1
display_name = "Instance Style Override Label"

[components.InstanceStyleOverrideLabel]
style_scope = "open"

[components.InstanceStyleOverrideLabel.root]
node_id = "instance_style_override_label_root"
kind = "native"
type = "Label"
classes = ["instance-label"]
props = { text = "Default", foreground_color = "#aaaaaa" }
"##;

const INSTANCE_STYLE_OVERRIDE_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.instance_style_override_style"
version = 1
display_name = "Instance Style Override Style"

[[stylesheets]]
id = "instance_style_override_style"

[[stylesheets.rules]]
selector = ".instance-label"
set = { self = { foreground_color = "#d8e3e7", text_tone = "primary" } }
"##;

const INSTANCE_STYLE_OVERRIDE_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.instance_style_override_layout"
version = 1
display_name = "Instance Style Override Layout"

[imports]
widgets = ["asset://ui/tests/instance_style_override_label.ui#InstanceStyleOverrideLabel"]
styles = ["asset://ui/tests/instance_style_override_style.ui"]

[root]
node_id = "label_instance"
kind = "reference"
component_ref = "asset://ui/tests/instance_style_override_label.ui#InstanceStyleOverrideLabel"
control_id = "InstanceStyleOverrideLabel"
style_overrides = { self = { foreground_color = "#ff0000", text_tone = "error" } }
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
fn ui_document_compiler_applies_reference_instance_props_to_expanded_root() {
    let widget = UiAssetLoader::load_toml_str(INSTANCE_PROPS_WIDGET_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(INSTANCE_PROPS_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            "asset://ui/tests/instance_props_label.ui#InstancePropsLabel",
            widget,
        )
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(root.control_id.as_deref(), Some("InstancePropsLabel"));
    assert_eq!(
        root.attributes.get("text").and_then(toml::Value::as_str),
        Some("Instance")
    );
    assert_eq!(
        root.attributes
            .get("foreground_color")
            .and_then(toml::Value::as_str),
        Some("#ff0000")
    );
    assert_eq!(
        root.attributes
            .get("selected")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        root.attributes
            .get("layout")
            .and_then(|layout| layout.get("width"))
            .and_then(|width| width.get("preferred"))
            .and_then(toml::Value::as_float),
        Some(96.0)
    );
    assert_eq!(
        root.attributes
            .get("layout_padding_left")
            .and_then(toml::Value::as_float),
        Some(2.0),
        "component defaults that were not overridden should survive"
    );
}

#[test]
fn ui_document_compiler_applies_reference_instance_style_overrides_after_stylesheets() {
    let widget = UiAssetLoader::load_toml_str(INSTANCE_STYLE_OVERRIDE_WIDGET_TOML).unwrap();
    let style = UiAssetLoader::load_toml_str(INSTANCE_STYLE_OVERRIDE_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(INSTANCE_STYLE_OVERRIDE_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            "asset://ui/tests/instance_style_override_label.ui#InstanceStyleOverrideLabel",
            widget,
        )
        .unwrap()
        .register_style_import("asset://ui/tests/instance_style_override_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(
        root.control_id.as_deref(),
        Some("InstanceStyleOverrideLabel")
    );
    assert_eq!(
        root.style_overrides
            .get("foreground_color")
            .and_then(toml::Value::as_str),
        Some("#ff0000")
    );
    assert_eq!(
        root.attributes
            .get("foreground_color")
            .and_then(toml::Value::as_str),
        Some("#ff0000")
    );
    assert_eq!(
        root.attributes
            .get("text_tone")
            .and_then(toml::Value::as_str),
        Some("error")
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
fn ui_document_compiler_rejects_component_slot_fills_outside_the_declared_accept_set() {
    let widget = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "widget"
id = "ui.tests.slot_host"
version = 1
display_name = "Slot Host"

[components.SlotHost]
style_scope = "open"

[components.SlotHost.slots.content]
accepts = ["Label"]

[components.SlotHost.root]
node_id = "slot_host_root"
kind = "native"
type = "Panel"

[[components.SlotHost.root.children]]
[components.SlotHost.root.children.node]
node_id = "content_slot"
kind = "slot"
slot_name = "content"
"##,
    )
    .unwrap();
    let layout = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "ui.tests.slot_host_layout"
version = 1
display_name = "Slot Host Layout"

[imports]
widgets = ["asset://ui/tests/slot_host.ui#SlotHost"]

[root]
node_id = "slot_host"
kind = "reference"
component_ref = "asset://ui/tests/slot_host.ui#SlotHost"

[[root.children]]
mount = "content"
[root.children.node]
node_id = "button_fill"
kind = "native"
type = "Button"
"##,
    )
    .unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/tests/slot_host.ui#SlotHost", widget)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("a Button fill must not satisfy a Label-only component slot");

    assert!(matches!(
        error,
        UiAssetError::SlotDoesNotAcceptComponent {
            ref component,
            ref slot_name,
            ref child_component,
        } if component == "SlotHost" && slot_name == "content" && child_component == "Button"
    ));
}

#[test]
fn ui_document_compiler_uses_the_reference_identity_for_reference_slot_fills() {
    let widget = UiAssetLoader::load_toml_str(TOOLBAR_ICON_WIDGET_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "ui.tests.reference_slot_identity"
version = 1
display_name = "Reference Slot Identity"

[imports]
widgets = ["asset://ui/common/toolbar_icon.ui#ToolbarIcon"]

[components.SlotHost]
style_scope = "open"

[components.SlotHost.slots.content]
accepts = ["Label"]

[components.SlotHost.root]
node_id = "slot_host_root"
kind = "native"
type = "Panel"

[[components.SlotHost.root.children]]
[components.SlotHost.root.children.node]
node_id = "content_slot"
kind = "slot"
slot_name = "content"

[components.Label]
style_scope = "open"

[components.Label.root]
node_id = "label_root"
kind = "native"
type = "Label"

[root]
node_id = "slot_host"
kind = "component"
component = "SlotHost"

[[root.children]]
mount = "content"
[root.children.node]
node_id = "reference_fill"
kind = "reference"
component = "Label"
component_ref = "asset://ui/common/toolbar_icon.ui#ToolbarIcon"
"##,
    )
    .unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/toolbar_icon.ui#ToolbarIcon", widget)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("a reference fill must not use an unrelated component field to bypass accepts");
    assert!(matches!(
        error,
        UiAssetError::SlotDoesNotAcceptComponent {
            ref component,
            ref slot_name,
            ref child_component,
        } if component == "SlotHost" && slot_name == "content" && child_component == "ToolbarIcon"
    ));
}

#[test]
fn ui_document_compiler_rejects_component_references_with_multiple_fragments() {
    let widget = UiAssetLoader::load_toml_str(TOOLBAR_ICON_WIDGET_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "ui.tests.invalid_component_reference"
version = 1
display_name = "Invalid Component Reference"

[imports]
widgets = ["asset://ui/common/toolbar_icon.ui#ToolbarIcon#Unexpected"]

[root]
node_id = "toolbar_icon"
kind = "reference"
component_ref = "asset://ui/common/toolbar_icon.ui#ToolbarIcon#Unexpected"
"##,
    )
    .unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            "asset://ui/common/toolbar_icon.ui#ToolbarIcon#Unexpected",
            widget,
        )
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("component references must have one component fragment");

    assert!(matches!(
        error,
        UiAssetError::InvalidDocument { ref detail, .. }
            if detail.contains("exactly one non-empty #Component suffix")
    ));
}

#[test]
fn prototype_slot_accepts_resolve_component_reference_children_from_the_caller_asset() {
    let slot_host = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "widget"
id = "asset://ui/tests/prototype_slot_host.ui"
version = 3
display_name = "Prototype Slot Host"

[components.SlotHost]
root = "slot_host_root"

[components.SlotHost.slots.content]
accepts = ["LabelFill"]

[nodes.slot_host_root]
kind = "native"
type = "Panel"
children = [{ child = "content_slot" }]

[nodes.content_slot]
kind = "slot"
slot_name = "content"
"##,
    )
    .unwrap();
    let label_fill = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "widget"
id = "asset://ui/tests/prototype_label_fill.ui"
version = 3
display_name = "Prototype Label Fill"

[components.LabelFill]
root = "label_fill_root"

[nodes.label_fill_root]
kind = "native"
type = "Label"
"##,
    )
    .unwrap();
    let button_fill = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "widget"
id = "asset://ui/tests/prototype_button_fill.ui"
version = 3
display_name = "Prototype Button Fill"

[components.ButtonFill]
root = "button_fill_root"

[nodes.button_fill_root]
kind = "native"
type = "Button"
"##,
    )
    .unwrap();
    let accepted_layout = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "layout"
id = "asset://ui/tests/prototype_slot_accepts_ok.ui"
version = 3
display_name = "Prototype Slot Accepts OK"

[imports]
widgets = [
  "asset://ui/tests/prototype_slot_host.ui#SlotHost",
  "asset://ui/tests/prototype_label_fill.ui#LabelFill",
]

[root]
node = "slot_host"

[nodes.slot_host]
kind = "reference"
component_ref = "asset://ui/tests/prototype_slot_host.ui#SlotHost"
children = [{ child = "label_fill", mount = "content" }]

[nodes.label_fill]
kind = "reference"
component_ref = "asset://ui/tests/prototype_label_fill.ui#LabelFill"
"##,
    )
    .unwrap();
    let rejected_layout = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "layout"
id = "asset://ui/tests/prototype_slot_accepts_rejected.ui"
version = 3
display_name = "Prototype Slot Accepts Rejected"

[imports]
widgets = [
  "asset://ui/tests/prototype_slot_host.ui#SlotHost",
  "asset://ui/tests/prototype_button_fill.ui#ButtonFill",
]

[root]
node = "slot_host"

[nodes.slot_host]
kind = "reference"
component_ref = "asset://ui/tests/prototype_slot_host.ui#SlotHost"
children = [{ child = "button_fill", mount = "content" }]

[nodes.button_fill]
kind = "reference"
component_ref = "asset://ui/tests/prototype_button_fill.ui#ButtonFill"
"##,
    )
    .unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    for prototype in [
        slot_host,
        label_fill,
        button_fill,
        accepted_layout,
        rejected_layout,
    ] {
        let _ = builder.insert(prototype);
    }
    let store = builder.build().unwrap();
    let compiler = UiDocumentCompiler::default();

    compiler
        .compile_prototype_asset("asset://ui/tests/prototype_slot_accepts_ok.ui", &store)
        .expect("the caller-owned LabelFill reference should satisfy the SlotHost contract");

    let error = compiler
        .compile_prototype_asset(
            "asset://ui/tests/prototype_slot_accepts_rejected.ui",
            &store,
        )
        .expect_err("the caller-owned ButtonFill reference must fail the SlotHost contract");
    assert!(matches!(
        error,
        UiAssetError::SlotDoesNotAcceptComponent {
            ref component,
            ref slot_name,
            ref child_component,
        } if component == "SlotHost" && slot_name == "content" && child_component == "ButtonFill"
    ));
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

#[test]
fn component_reference_inline_style_merge_borrows_overrides_without_deep_clone() {
    let style_apply = include_str!("../template/asset/compiler/style_apply.rs");
    let prototype = include_str!("../template/asset/compiler/prototype_instancer.rs");

    for source in [style_apply, prototype] {
        assert!(
            source.contains("(&mut node.attributes, &node.style_overrides)"),
            "inline style merge must borrow disjoint node fields"
        );
        assert!(
            !source.contains("let inline = node.style_overrides.clone();"),
            "inline style merge must not deep-clone the full override map"
        );
    }
}
