use toml::Value;

use zircon_runtime::ui::template::{UiAssetLoader, UiDocumentCompiler};

#[test]
fn component_slot_layout_resolves_component_tokens_params_and_caller_overrides() {
    let row_asset = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "widget"
id = "ui.common.property_row"
version = 2
display_name = "Property Row"

[tokens]
value_slot_stretch = "Stretch"

[components.PropertyRow]
root = "root"
slots = { value = { multiple = false } }

[components.PropertyRow.params.value_height]
type = "number"
default = 28.0

[nodes.root]
kind = "native"
type = "HorizontalBox"
layout = { container = { kind = "HorizontalBox", gap = 4.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
children = [{ child = "value_slot" }]

[nodes.value_slot]
kind = "slot"
slot_name = "value"
layout = { width = { stretch = "$value_slot_stretch" }, height = { min = "$param.value_height", preferred = "$param.value_height", max = "$param.value_height", stretch = "Fixed" } }
"#,
    )
    .unwrap();
    let layout_asset = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "editor.property_row_test"
version = 2
display_name = "Property Row Test"

[imports]
widgets = ["asset://ui/common/property_row.ui#PropertyRow"]

[root]
node = "root"

[nodes.root]
kind = "reference"
component_ref = "asset://ui/common/property_row.ui#PropertyRow"
control_id = "PropertyRowHost"
params = { value_height = 30.0 }
children = [{ child = "field", mount = "value", slot = { layout = { width = { weight = 2.0 } } } }]

[nodes.field]
kind = "native"
type = "TextField"
control_id = "PropertyValueField"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
"#,
    )
    .unwrap();

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/property_row.ui#PropertyRow", row_asset)
        .unwrap();
    let instance = compiler
        .compile(&layout_asset)
        .unwrap()
        .into_template_instance();
    let value_field = instance
        .root
        .children
        .iter()
        .find(|child| child.control_id.as_deref() == Some("PropertyValueField"))
        .expect("value field should replace the component slot placeholder");
    let layout = value_field
        .slot_attributes
        .get("layout")
        .expect("mounted value field should inherit placeholder layout");

    assert_eq!(
        layout
            .get("width")
            .and_then(|width| width.get("stretch"))
            .and_then(Value::as_str),
        Some("Stretch")
    );
    assert_eq!(
        layout
            .get("height")
            .and_then(|height| height.get("min"))
            .and_then(Value::as_float),
        Some(30.0)
    );
    assert_eq!(
        layout
            .get("height")
            .and_then(|height| height.get("preferred"))
            .and_then(Value::as_float),
        Some(30.0)
    );
    assert_eq!(
        layout
            .get("width")
            .and_then(|width| width.get("weight"))
            .and_then(Value::as_float),
        Some(2.0),
        "caller-authored mount leaves must override resolved component defaults"
    );
}
