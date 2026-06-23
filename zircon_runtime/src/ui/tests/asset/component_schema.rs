use super::*;

#[test]
fn ui_asset_compiler_applies_runtime_component_schema_defaults() {
    const NUMBER_FIELD_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.component.defaults"
version = 1
display_name = "Component Defaults"

[root]
node_id = "number"
kind = "native"
type = "NumberField"
control_id = "Number"
props = { value = 42.0 }
"#;

    let document = UiAssetLoader::load_toml_str(NUMBER_FIELD_TOML).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(instance.root.component.as_deref(), Some("NumberField"));
    assert_eq!(
        instance
            .root
            .attributes
            .get("value")
            .and_then(Value::as_float),
        Some(42.0)
    );
    assert_eq!(
        instance
            .root
            .attributes
            .get("min")
            .and_then(Value::as_float),
        Some(0.0)
    );
    assert_eq!(
        instance
            .root
            .attributes
            .get("max")
            .and_then(Value::as_float),
        Some(100.0)
    );
    assert_eq!(
        instance
            .root
            .attributes
            .get("step")
            .and_then(Value::as_float),
        Some(1.0)
    );
    assert_eq!(
        instance
            .root
            .attributes
            .get("large_step")
            .and_then(Value::as_float),
        Some(10.0)
    );
}

#[test]
fn ui_asset_compiler_rejects_runtime_component_props_with_wrong_type() {
    const INVALID_NUMBER_FIELD_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.component.invalid"
version = 1
display_name = "Invalid Component Prop"

[root]
node_id = "number"
kind = "native"
type = "NumberField"
control_id = "Number"
props = { value = "not numeric" }
"#;

    let document = UiAssetLoader::load_toml_str(INVALID_NUMBER_FIELD_TOML).unwrap();
    let error = UiDocumentCompiler::default()
        .compile(&document)
        .expect_err("NumberField.value should require a numeric value");

    assert!(
        error
            .to_string()
            .contains("component NumberField prop value expected Float"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_compiler_preserves_style_attributes_unknown_to_component_schema() {
    const LABEL_WITH_STYLE_PROP_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.component.style_props"
version = 1
display_name = "Component Style Props"

[root]
node_id = "label"
kind = "native"
type = "Label"
control_id = "Label"
props = { text = "Styled", color = "#ffaa00" }
"##;

    let document = UiAssetLoader::load_toml_str(LABEL_WITH_STYLE_PROP_TOML).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(
        instance.root.attributes.get("text").and_then(Value::as_str),
        Some("Styled")
    );
    assert_eq!(
        instance
            .root
            .attributes
            .get("color")
            .and_then(Value::as_str),
        Some("#ffaa00")
    );
}
