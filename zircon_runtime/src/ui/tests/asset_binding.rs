use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::{
    collect_asset_binding_report, UiAssetLoader, UiDocumentCompiler, UiRuntimeCompiledAssetArtifact,
};
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiPropSchema, UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::template::{
    UiBindingDiagnosticCode, UiBindingDiagnosticSeverity, UiBindingExpression,
    UiCompiledAssetPackageProfile,
};

mod apply_report_performance;
mod compiled_program;
mod control_scope;
mod default_interaction_schema;
mod naming_contract;
mod schema_naming;
mod telemetry_performance;

const VALID_BINDING_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.valid"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Valid"

[root.bindings.action]
route = "Route.Valid"

[root.bindings.action.payload]
status = "clean"

[[root.bindings.targets]]
target = { kind = "prop", name = "text" }
expression = '"Bound"'

[[root.bindings.targets]]
target = { kind = "class", name = "highlighted" }
expression = 'prop.text == "Ready"'

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = "true"

[[root.bindings.targets]]
target = { kind = "enabled" }
expression = 'prop.text != ""'

[[root.bindings.targets]]
target = { kind = "action_payload", name = "status" }
expression = '"clean"'
"##;

const INVALID_PROP_TARGET_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.invalid_target"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "prop", name = "missing" }
expression = '"Bound"'
"##;

const INVALID_VALUE_KIND_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.invalid_kind"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "prop", name = "text" }
expression = "true"
"##;

const UNRESOLVED_REF_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.unresolved"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = 'prop.missing == "Ready"'
"##;

const UNSUPPORTED_OPERATOR_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.unsupported_operator"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = 'prop.text > "Ready"'
"##;

const PREVIEW_PAYLOAD_EXPRESSION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.payload_expression"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[root.bindings.action]
route = "Route.Invalid"

[root.bindings.action.payload]
status = "=prop.missing"
"##;

const EDITOR_PREVIEW_FUNCTION_PAYLOAD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.preview_function_payload"
version = 3

[components.Preview.root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[components.Preview.root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Valid"

[components.Preview.root.bindings.action]
route = "Route.Valid"

[components.Preview.root.bindings.action.payload]
status = "=concat(\"param.title\", StatusLabel.text, \" / \", self.text)"

[root]
node_id = "preview"
kind = "component"
component = "Preview"
"##;

const PARAM_REF_COMPONENT_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.param_ref"
version = 3

[components.Status.params.visible]
type = "bool"
default = true

[components.Status.params.label]
type = "string"
default = "Default"

[components.Status.params.ratio]
type = "float"
default = 2

[components.Status.params.tint]
type = "color"
default = "#ff00aa\b\f"

[components.Status.params.offset]
type = "vec2"
default = [1, 2]

[components.Status.params.modes]
type = "flags"
default = ["read", "write"]

[components.Status.root]
node_id = "status_root"
kind = "native"
type = "RangeField"
control_id = "StatusRoot"
props = { value = 2.0 }

[[components.Status.root.bindings]]
id = "Status/onChange"
event = "Change"
route = "Route.Status"

[[components.Status.root.bindings.targets]]
target = { kind = "visibility" }
expression = "param.visible"

[components.Status.root.bindings.action]
route = "Route.Status"

[components.Status.root.bindings.action.payload]
visible = "=param.visible"
label = "=param.label"
changed = "=param.visible == true"
dynamic = "=param.visible || (prop.value == 2.0)"
ratio = "=param.ratio"
ratio_dynamic = "=param.ratio == prop.value"
offset = "=param.offset"
modes = "=param.modes"

[[components.Status.root.children]]
[components.Status.root.children.node]
node_id = "color"
kind = "native"
type = "ColorField"
props = { value = "#ff00aa\b\f" }

[[components.Status.root.children.node.bindings]]
id = "Status/onColorChange"
event = "Change"
route = "Route.StatusColor"

[[components.Status.root.children.node.bindings.targets]]
target = { kind = "visibility" }
expression = "param.tint == prop.value"

[components.Status.root.children.node.bindings.action]
route = "Route.StatusColor"

[components.Status.root.children.node.bindings.action.payload]
tint = "=param.tint"

[components.Shell.params.state]
type = "bool"
default = true

[components.Shell.root]
node_id = "nested_status"
kind = "component"
component = "Status"
params = { visible = "$param.state" }

[[components.Shell.root.bindings]]
id = "Shell/onChange"
event = "Change"
route = "Route.Shell"

[[components.Shell.root.bindings.targets]]
target = { kind = "enabled" }
expression = "param.state"

[components.Shell.root.bindings.action]
route = "Route.Shell"

[components.Shell.root.bindings.action.payload]
state = "=param.state"

[root]
node_id = "root"
kind = "component"
component = "Shell"
params = { state = false }
"##;

const BOOLEAN_OPERATORS_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.boolean_operators"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Valid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = '=!(prop.text == "") && (prop.text == "Ready" || false)'
"##;

// Keeps unit coverage beside the binding validator in addition to the public integration gate.
fn control_prop_ref_layout(expression: &str) -> String {
    format!(
        r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Container"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "source"
kind = "native"
type = "Label"
control_id = "BindingSource"
props = {{ text = "Ready" }}

[[root.children]]
[root.children.node]
node_id = "consumer"
kind = "native"
type = "Button"
control_id = "BindingConsumer"
props = {{ text = "Apply" }}

[[root.children.node.bindings]]
id = "Consumer/onClick"
event = "Click"
route = "Route.Valid"

[[root.children.node.bindings.targets]]
target = {{ kind = "visibility" }}
expression = {expression:?}
"##
    )
}

const CONTROL_PROP_REF_ACTION_PAYLOAD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref_payload"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Container"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "source"
kind = "native"
type = "Label"
control_id = "BindingSource"
props = { text = "Ready" }

[[root.children]]
[root.children.node]
node_id = "entity_source"
kind = "native"
type = "TreeView"
control_id = "BindingEntitySource"
props = { selected_index = 7 }

[[root.children]]
[root.children.node]
node_id = "force_source"
kind = "native"
type = "Checkbox"
control_id = "BindingForceSource"
props = { checked = true }

[[root.children]]
[root.children.node]
node_id = "consumer"
kind = "native"
type = "Button"
control_id = "BindingConsumer"
props = { text = "Apply" }

[[root.children.node.bindings]]
id = "Consumer/onClick"
event = "Click"
route = "Route.Valid"

[root.children.node.bindings.action]
route = "Route.Valid"

[root.children.node.bindings.action.payload]
surface_entity = "=control.BindingEntitySource.prop.selected_index"
force_full_rebuild = "=control.BindingForceSource.prop.checked"
status = "=control.BindingSource.prop.text"
"##;

const CONTROL_PROP_REF_COMPONENT_SCOPE_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref_scope"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "RootConsumer"
props = { text = "Apply" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = 'control.ComponentSource.prop.checked'

[components.ScopedSource.root]
node_id = "component_source"
kind = "native"
type = "Checkbox"
control_id = "ComponentSource"
props = { checked = true }
"##;

const DESCRIPTOR_AUTHORITY_UNKNOWN_PROP_TARGET_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.descriptor_unknown_target"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready", texxt = "Typo" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "prop", name = "texxt" }
expression = '"Typo"'
"##;

const DESCRIPTOR_AUTHORITY_UNKNOWN_PROP_REF_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.descriptor_unknown_ref"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready", texxt = "Typo" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = 'prop.texxt == "Typo"'
"##;

const MISSING_ACTION_PAYLOAD_TARGET_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.missing_payload_target"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[root.bindings.action]
route = "Route.Invalid"

[root.bindings.action.payload]
status = "clean"

[[root.bindings.targets]]
target = { kind = "action_payload", name = "missing" }
expression = '"clean"'
"##;

fn visibility_expression_layout(expression: &str) -> String {
    format!(
        r##"
[asset]
kind = "layout"
id = "editor.binding.unsupported_operator.dynamic"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = {{ text = "Ready" }}

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = {{ kind = "visibility" }}
expression = {expression:?}
"##
    )
}

#[test]
fn asset_binding_accepts_valid_prop_class_visibility_enabled_and_payload_targets() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let compiler = UiDocumentCompiler::default().with_component_registry(registry.clone());

    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
    compiler.compile(&document).unwrap();
}

#[test]
fn asset_binding_deserializes_compact_target_assignments() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let binding = &document.root.as_ref().unwrap().bindings[0];

    assert_eq!(binding.targets.len(), 5);
    assert_eq!(binding.targets[0].target.name.as_deref(), Some("text"));
    assert_eq!(binding.targets[0].expression, "\"Bound\"");
}

#[test]
fn asset_binding_accepts_registered_custom_prop_targets() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let mut registry = UiComponentDescriptorRegistry::new();
    registry
        .register(
            UiComponentDescriptor::new(
                "Button",
                "Binding Button",
                UiComponentCategory::Input,
                "button",
            )
            .with_prop(UiPropSchema::new("text", UiValueKind::String)),
        )
        .unwrap();

    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
}

#[test]
fn asset_binding_accepts_boolean_operators_parentheses_and_leading_equals() {
    let document = UiAssetLoader::load_toml_str(BOOLEAN_OPERATORS_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
}

#[test]
fn param_ref_compile_resolves_nested_params_and_artifact_roundtrip() {
    let document = UiAssetLoader::load_toml_str(PARAM_REF_COMPONENT_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let compiler = UiDocumentCompiler::default().with_component_registry(registry.clone());

    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
    let artifact = compiler
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let decoded =
        UiRuntimeCompiledAssetArtifact::from_bytes(&artifact.to_bytes().unwrap()).unwrap();
    let binding = &decoded.compiled.root.bindings[0];
    let action = binding.action.as_ref().unwrap();

    assert_eq!(binding.targets[0].expression, "false");
    assert_eq!(
        action.payload.get("visible"),
        Some(&toml::Value::Boolean(false))
    );
    assert_eq!(
        action.payload.get("label").and_then(toml::Value::as_str),
        Some("Default")
    );
    assert_eq!(
        action.payload.get("changed"),
        Some(&toml::Value::Boolean(false))
    );
    assert_eq!(
        action.payload.get("dynamic").and_then(toml::Value::as_str),
        Some("=(false || (prop.value == 2.0))")
    );
    assert_eq!(
        action.payload.get("ratio").and_then(toml::Value::as_float),
        Some(2.0)
    );
    assert_eq!(
        action
            .payload
            .get("ratio_dynamic")
            .and_then(toml::Value::as_str),
        Some("=(2.0 == prop.value)")
    );
    let offset_expression = action
        .payload
        .get("offset")
        .and_then(toml::Value::as_str)
        .unwrap();
    assert_eq!(
        UiBindingExpression::parse(offset_expression).unwrap(),
        UiBindingExpression::Literal(UiValue::Vec2([1.0, 2.0]))
    );
    let modes_expression = action
        .payload
        .get("modes")
        .and_then(toml::Value::as_str)
        .unwrap();
    assert_eq!(
        UiBindingExpression::parse(modes_expression).unwrap(),
        UiBindingExpression::Literal(UiValue::Flags(vec![
            "read".to_string(),
            "write".to_string(),
        ]))
    );
    let color_binding = &decoded.compiled.root.children[0].bindings[0];
    assert_eq!(
        UiBindingExpression::parse(&color_binding.targets[0].expression).unwrap(),
        UiBindingExpression::Equals(
            Box::new(UiBindingExpression::Literal(UiValue::Color(
                "#ff00aa\u{0008}\u{000c}".to_string(),
            ))),
            Box::new(UiBindingExpression::PropRef("value".to_string())),
        )
    );
    let tint_expression = color_binding
        .action
        .as_ref()
        .and_then(|action| action.payload.get("tint"))
        .and_then(toml::Value::as_str)
        .unwrap();
    assert_eq!(
        UiBindingExpression::parse(tint_expression).unwrap(),
        UiBindingExpression::Literal(UiValue::Color("#ff00aa\u{0008}\u{000c}".to_string(),))
    );
    let caller_binding = decoded
        .compiled
        .root
        .bindings
        .iter()
        .find(|binding| binding.id == "Shell/onChange")
        .unwrap();
    assert_eq!(caller_binding.targets[0].expression, "false");
    assert_eq!(
        caller_binding
            .action
            .as_ref()
            .and_then(|action| action.payload.get("state")),
        Some(&toml::Value::Boolean(false))
    );
    assert!(!toml::to_string(&decoded.compiled)
        .unwrap()
        .contains("param."));
}

#[test]
fn param_ref_compile_preserves_non_param_preview_expressions() {
    let document = UiAssetLoader::load_toml_str(EDITOR_PREVIEW_FUNCTION_PAYLOAD_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let compiled = UiDocumentCompiler::default()
        .with_component_registry(registry)
        .compile(&document)
        .unwrap();

    assert_eq!(
        compiled.template_instance().root.bindings[0]
            .action
            .as_ref()
            .and_then(|action| action.payload.get("status"))
            .and_then(toml::Value::as_str),
        Some("=concat(\"param.title\", StatusLabel.text, \" / \", self.text)")
    );
}

#[test]
fn param_ref_compile_rejects_a_missing_referenced_component_param() {
    let source = PARAM_REF_COMPONENT_LAYOUT
        .replacen("default = true", "", 1)
        .replace("params = { visible = \"$param.state\" }", "params = {}");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let compiler = UiDocumentCompiler::default().with_component_registry(registry);

    let error = compiler
        .compile(&document)
        .expect_err("a referenced component param must resolve during expansion");

    assert!(error
        .to_string()
        .contains("missing component param visible"));
}

#[test]
fn asset_binding_resolves_control_prop_refs_against_the_current_tree() {
    let source = control_prop_ref_layout("control.BindingSource.prop.text == \"Ready\"");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
}

#[test]
fn asset_binding_reports_unknown_control_prop_ref_control_ids() {
    let source = control_prop_ref_layout("control.MissingSource.prop.text == \"Ready\"");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
    assert!(report.diagnostics[0]
        .message
        .contains("unknown control MissingSource"));
}

#[test]
fn asset_binding_reports_unknown_control_prop_ref_properties() {
    let source = control_prop_ref_layout("control.BindingSource.prop.missing == \"Ready\"");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
    assert!(report.diagnostics[0]
        .message
        .contains("control BindingSource references unknown prop missing"));
}

#[test]
fn asset_binding_reports_control_prop_ref_target_kind_mismatches() {
    let source = control_prop_ref_layout("control.BindingSource.prop.text");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::InvalidValueKind
    );
}

#[test]
fn asset_binding_does_not_infer_control_prop_kinds_from_unknown_descriptors() {
    let source = control_prop_ref_layout("control.BindingSource.prop.text == \"Ready\"")
        .replace("type = \"Label\"", "type = \"MissingWidget\"");
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
}

#[test]
fn asset_binding_keeps_control_prop_refs_inside_their_component_tree() {
    let document = UiAssetLoader::load_toml_str(CONTROL_PROP_REF_COMPONENT_SCOPE_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
    assert!(report.diagnostics[0]
        .message
        .contains("unknown control ComponentSource"));
}

#[test]
fn asset_binding_validates_control_prop_refs_in_action_payloads() {
    let document = UiAssetLoader::load_toml_str(CONTROL_PROP_REF_ACTION_PAYLOAD_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
}

#[test]
fn asset_binding_descriptor_props_reject_authored_unknown_prop_targets() {
    let document =
        UiAssetLoader::load_toml_str(DESCRIPTOR_AUTHORITY_UNKNOWN_PROP_TARGET_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::InvalidTarget
    );
}

#[test]
fn asset_binding_descriptor_props_reject_authored_unknown_prop_refs() {
    let document =
        UiAssetLoader::load_toml_str(DESCRIPTOR_AUTHORITY_UNKNOWN_PROP_REF_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
}

#[test]
fn asset_binding_rejects_missing_action_payload_targets() {
    let document = UiAssetLoader::load_toml_str(MISSING_ACTION_PAYLOAD_TARGET_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::InvalidTarget
    );
}

#[test]
fn asset_binding_reports_invalid_target() {
    let document = UiAssetLoader::load_toml_str(INVALID_PROP_TARGET_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.code, UiBindingDiagnosticCode::InvalidTarget);
    assert_eq!(diagnostic.severity, UiBindingDiagnosticSeverity::Error);
    assert_eq!(diagnostic.node_id, "root");
    assert_eq!(diagnostic.binding_id, "Root/onClick");
}

#[test]
fn asset_binding_reports_invalid_value_kind() {
    let document = UiAssetLoader::load_toml_str(INVALID_VALUE_KIND_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::InvalidValueKind
    );
}

#[test]
fn asset_binding_reports_unresolved_ref() {
    let document = UiAssetLoader::load_toml_str(UNRESOLVED_REF_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
}

#[test]
fn asset_binding_reports_unsupported_operator() {
    let document = UiAssetLoader::load_toml_str(UNSUPPORTED_OPERATOR_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnsupportedOperator
    );
}

#[test]
fn asset_binding_reports_malformed_boolean_or_assignment_operators_as_unsupported() {
    for expression in [
        "prop.text = \"Ready\"",
        "prop.text & true",
        "prop.text | true",
    ] {
        let source = visibility_expression_layout(expression);
        let document = UiAssetLoader::load_toml_str(&source).unwrap();
        let registry = UiComponentDescriptorRegistry::editor_showcase();
        let report = collect_asset_binding_report(&document, &registry);

        assert_eq!(report.diagnostics.len(), 1, "{expression}");
        assert_eq!(
            report.diagnostics[0].code,
            UiBindingDiagnosticCode::UnsupportedOperator,
            "{expression}"
        );
    }
}

#[test]
fn asset_binding_validates_preview_style_action_payload_expressions() {
    let document = UiAssetLoader::load_toml_str(PREVIEW_PAYLOAD_EXPRESSION_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
}

#[test]
fn asset_binding_leaves_editor_preview_function_payloads_to_editor_preview() {
    let document = UiAssetLoader::load_toml_str(EDITOR_PREVIEW_FUNCTION_PAYLOAD_LAYOUT).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let report = collect_asset_binding_report(&document, &registry);

    assert!(report.diagnostics.is_empty());
}

#[test]
fn asset_binding_compiler_precondition_rejects_invalid_semantics() {
    let document = UiAssetLoader::load_toml_str(INVALID_PROP_TARGET_LAYOUT).unwrap();

    let error = UiDocumentCompiler::default()
        .compile(&document)
        .expect_err("invalid binding targets must fail before expansion");

    assert!(error.to_string().contains("targets unknown prop missing"));
}
