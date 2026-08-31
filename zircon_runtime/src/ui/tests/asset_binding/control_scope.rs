use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::template::{UiAssetError, UiBindingExpression, UiTemplateNode};

const REPEATED_COMPONENT_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_scope.repeated"
version = 3

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "global_source"
kind = "native"
type = "Label"
control_id = "GlobalSource"
props = { text = "Global" }

[[root.children]]
[root.children.node]
node_id = "first_instance"
kind = "component"
component = "ScopedRow"
control_id = "First"
params = { value = "First value" }

[[root.children.node.bindings]]
id = "First/caller"
event = "Click"
route = "Route.Caller"

[root.children.node.bindings.action]
route = "Route.Caller"

[root.children.node.bindings.action.payload]
value = "=control.GlobalSource.prop.text"

[[root.children]]
[root.children.node]
node_id = "second_instance"
kind = "component"
component = "ScopedRow"
control_id = "Second"
params = { value = "Second value" }

[components.ScopedRow.params.value]
type = "string"

[components.ScopedRow.root]
node_id = "row_root"
kind = "native"
type = "Button"
control_id = "RowRoot"
props = { text = "Emit" }

[[components.ScopedRow.root.bindings]]
id = "ScopedRow/onClick"
event = "Click"
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action]
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action.payload]
value = "=control.Value.prop.text"

[[components.ScopedRow.root.children]]
[components.ScopedRow.root.children.node]
node_id = "value"
kind = "native"
type = "Label"
control_id = "Value"
props = { text = "$param.value" }
"##;

const NESTED_COMPONENT_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_scope.nested"
version = 3

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "first_wrapper"
kind = "component"
component = "Wrapper"
control_id = "FirstWrapper"
params = { value = "Nested first" }

[[root.children]]
[root.children.node]
node_id = "second_wrapper"
kind = "component"
component = "Wrapper"
control_id = "SecondWrapper"
params = { value = "Nested second" }

[components.Wrapper.params.value]
type = "string"

[components.Wrapper.root]
node_id = "inner_row"
kind = "component"
component = "ScopedRow"
params = { value = "$param.value" }

[components.ScopedRow.params.value]
type = "string"

[components.ScopedRow.root]
node_id = "row_root"
kind = "native"
type = "Button"
control_id = "RowRoot"
props = { text = "Emit" }

[[components.ScopedRow.root.bindings]]
id = "ScopedRow/onClick"
event = "Click"
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action]
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action.payload]
value = "=control.Value.prop.text"
root_value = "=control.RowRoot.prop.text"

[[components.ScopedRow.root.children]]
[components.ScopedRow.root.children.node]
node_id = "value"
kind = "native"
type = "Label"
control_id = "Value"
props = { text = "$param.value" }
"##;

#[test]
fn component_control_scope_routes_repeated_instances_to_their_own_payloads() {
    let instance = compile(REPEATED_COMPONENT_LAYOUT);
    let first = node_with_control_id(&instance.root, "First");
    let second = node_with_control_id(&instance.root, "Second");
    let first_source = first.children.first().expect("first instance source");
    let second_source = second.children.first().expect("second instance source");

    assert_ne!(first_source.control_id, second_source.control_id);
    assert_binding_reads_control(first, "ScopedRow/onClick", first_source);
    assert_binding_reads_control(second, "ScopedRow/onClick", second_source);

    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("runtime.ui.control_scope.repeated"),
        &instance,
    )
    .unwrap();
    assert_eq!(
        action_payload(&surface, "First", "ScopedRow/onClick", "value"),
        UiValue::String("First value".to_string())
    );
    assert_eq!(
        action_payload(&surface, "Second", "ScopedRow/onClick", "value"),
        UiValue::String("Second value".to_string())
    );
    assert_eq!(
        action_payload(&surface, "First", "First/caller", "value"),
        UiValue::String("Global".to_string()),
        "bindings authored on the instance node must remain in the caller scope"
    );
}

#[test]
fn component_control_scope_composes_for_nested_instances() {
    let instance = compile(NESTED_COMPONENT_LAYOUT);
    let first = node_with_control_id(&instance.root, "FirstWrapper");
    let second = node_with_control_id(&instance.root, "SecondWrapper");
    let first_source = first.children.first().expect("nested first source");
    let second_source = second.children.first().expect("nested second source");

    assert_ne!(first_source.control_id, second_source.control_id);
    assert_binding_reads_control(first, "ScopedRow/onClick", first_source);
    assert_binding_reads_control(second, "ScopedRow/onClick", second_source);
    assert_binding_payload_reads_control(first, "ScopedRow/onClick", "root_value", first);
    assert_binding_payload_reads_control(second, "ScopedRow/onClick", "root_value", second);
}

#[test]
fn compiler_rejects_duplicate_control_ids_after_expansion() {
    let source = r#"
[asset]
kind = "layout"
id = "editor.binding.control_scope.duplicate"
version = 3

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
control_id = "Duplicate"

[[root.children]]
[root.children.node]
node_id = "child"
kind = "native"
type = "Label"
control_id = "Duplicate"
"#;
    let document = UiAssetLoader::load_toml_str(source).unwrap();

    let error = UiDocumentCompiler::default()
        .with_component_registry(UiComponentDescriptorRegistry::editor_showcase())
        .compile(&document)
        .expect_err("duplicate control ids must fail compilation");

    assert_eq!(
        error,
        UiAssetError::InvalidDocument {
            asset_id: "editor.binding.control_scope.duplicate".to_string(),
            detail: "compiled template contains duplicate control ids: Duplicate".to_string(),
        }
    );
}

#[test]
fn component_control_scope_qualifies_one_thousand_instances_linearly() {
    const INSTANCE_COUNT: usize = 1_000;
    let source = scale_layout(INSTANCE_COUNT);
    let started = Instant::now();
    let instance = compile(&source);
    let elapsed = started.elapsed();

    let mut scoped_ids = BTreeSet::new();
    let mut referenced_ids = BTreeSet::new();
    let mut stack = vec![&instance.root];
    while let Some(node) = stack.pop() {
        if let Some(control_id) = node.control_id.as_deref() {
            if control_id.starts_with("__zircon_component_instance_") {
                scoped_ids.insert(control_id.to_string());
            }
        }
        for binding in &node.bindings {
            if let Some(expression) = binding
                .action
                .as_ref()
                .and_then(|action| action.payload.get("value"))
                .and_then(toml::Value::as_str)
                .and_then(|source| UiBindingExpression::parse(source).ok())
            {
                collect_control_refs(&expression, &mut referenced_ids);
            }
        }
        stack.extend(node.children.iter());
    }

    assert_eq!(scoped_ids.len(), INSTANCE_COUNT);
    assert_eq!(referenced_ids, scoped_ids);
    assert!(
        elapsed < Duration::from_secs(5),
        "1,000-instance control qualification took {elapsed:?}"
    );
    eprintln!(
        "PERF-RUNTIME74-CONTROL-SCOPE component_instances={INSTANCE_COUNT} unique_scoped_control_ids={} resolved_control_refs={} global_duplicate_fallbacks=0 elapsed_us={}",
        scoped_ids.len(),
        referenced_ids.len(),
        elapsed.as_micros()
    );
}

fn compile(source: &str) -> crate::ui::template::UiTemplateInstance {
    let document = UiAssetLoader::load_toml_str(source).unwrap();
    UiDocumentCompiler::default()
        .with_component_registry(UiComponentDescriptorRegistry::editor_showcase())
        .compile(&document)
        .unwrap()
        .into_template_instance()
}

fn node_with_control_id<'a>(node: &'a UiTemplateNode, control_id: &str) -> &'a UiTemplateNode {
    find_node_with_control_id(node, control_id)
        .unwrap_or_else(|| panic!("missing control {control_id}"))
}

fn find_node_with_control_id<'a>(
    node: &'a UiTemplateNode,
    control_id: &str,
) -> Option<&'a UiTemplateNode> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node_with_control_id(child, control_id))
}

fn assert_binding_reads_control(owner: &UiTemplateNode, binding_id: &str, source: &UiTemplateNode) {
    assert_binding_payload_reads_control(owner, binding_id, "value", source);
}

fn assert_binding_payload_reads_control(
    owner: &UiTemplateNode,
    binding_id: &str,
    payload_key: &str,
    source: &UiTemplateNode,
) {
    let binding = owner
        .bindings
        .iter()
        .find(|binding| binding.id == binding_id)
        .expect("scoped binding");
    let expression = binding
        .action
        .as_ref()
        .and_then(|action| action.payload.get(payload_key))
        .and_then(toml::Value::as_str)
        .and_then(|source| UiBindingExpression::parse(source).ok())
        .expect("compiled control expression");
    assert_eq!(
        expression,
        UiBindingExpression::ControlPropRef {
            control_id: source
                .control_id
                .clone()
                .expect("qualified source control id"),
            property: "text".to_string(),
        }
    );
}

fn action_payload(
    surface: &crate::ui::surface::UiSurface,
    control_id: &str,
    binding_id: &str,
    key: &str,
) -> UiValue {
    let (node_id, binding) = surface
        .tree
        .nodes
        .iter()
        .find_map(|(node_id, node)| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata.control_id.as_deref() == Some(control_id)).then(|| {
                (
                    *node_id,
                    metadata
                        .bindings
                        .iter()
                        .find(|binding| binding.id == binding_id)
                        .cloned()
                        .expect("binding on control"),
                )
            })
        })
        .expect("surface control");
    surface
        .template_action_for_binding(node_id, &binding)
        .and_then(|action| action.payload.get(key).cloned())
        .expect("resolved action payload")
}

fn collect_control_refs(expression: &UiBindingExpression, out: &mut BTreeSet<String>) {
    match expression {
        UiBindingExpression::ControlPropRef { control_id, .. } => {
            out.insert(control_id.clone());
        }
        UiBindingExpression::Equals(lhs, rhs)
        | UiBindingExpression::NotEquals(lhs, rhs)
        | UiBindingExpression::And(lhs, rhs)
        | UiBindingExpression::Or(lhs, rhs) => {
            collect_control_refs(lhs, out);
            collect_control_refs(rhs, out);
        }
        UiBindingExpression::Not(value) => collect_control_refs(value, out),
        UiBindingExpression::Literal(_)
        | UiBindingExpression::ParamRef(_)
        | UiBindingExpression::PropRef(_) => {}
    }
}

fn scale_layout(instance_count: usize) -> String {
    let mut source = r##"
[asset]
kind = "layout"
id = "editor.binding.control_scope.scale"
version = 3

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
control_id = "Root"

[components.ScopedRow.root]
node_id = "row_root"
kind = "native"
type = "Button"

[[components.ScopedRow.root.bindings]]
id = "ScopedRow/onClick"
event = "Click"
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action]
route = "Route.Scoped"

[components.ScopedRow.root.bindings.action.payload]
value = "=control.Value.prop.text"

[[components.ScopedRow.root.children]]
[components.ScopedRow.root.children.node]
node_id = "value"
kind = "native"
type = "Label"
control_id = "Value"
props = { text = "Value" }
"##
    .to_string();
    for index in 0..instance_count {
        source.push_str(&format!(
            r##"

[[root.children]]
[root.children.node]
node_id = "instance_{index}"
kind = "component"
component = "ScopedRow"
control_id = "Instance{index}"
"##
        ));
    }
    source
}
