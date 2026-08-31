use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder};
use zircon_runtime_interface::ui::template::UiAssetError;

use super::instance_projection::TOOLBAR_ICON_WIDGET_TOML;

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
