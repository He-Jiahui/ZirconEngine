use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder};
use zircon_runtime_interface::ui::template::{UiBindingExpression, UiTemplateNode};

const FLAT_CONTROL_SCOPE_ASSET: &str = r##"
[asset]
kind = "layout"
id = "asset://ui/tests/control_scope_flat.ui"
version = 3

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "first" }, { child = "second" }]

[nodes.first]
kind = "component"
component = "Wrapper"
control_id = "First"

[nodes.second]
kind = "component"
component = "Wrapper"
control_id = "Second"

[components.Wrapper]
root = "wrapper_row"

[nodes.wrapper_row]
kind = "component"
component = "ScopedRow"

[components.ScopedRow]
root = "row_root"

[nodes.row_root]
kind = "native"
type = "Button"
control_id = "RowRoot"
children = [{ child = "value" }]

[[nodes.row_root.bindings]]
id = "ScopedRow/onClick"
event = "Click"
route = "Route.Scoped"

[nodes.row_root.bindings.action]
route = "Route.Scoped"

[nodes.row_root.bindings.action.payload]
value = "=control.Value.prop.text"
root_value = "=control.RowRoot.prop.text"

[nodes.value]
kind = "native"
type = "Label"
control_id = "Value"
props = { text = "Value" }
"##;

#[test]
fn prototype_component_control_scope_matches_tree_compiler_semantics() {
    let prototype = UiAssetLoader::load_flat_prototype_toml_str(FLAT_CONTROL_SCOPE_ASSET).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(prototype);
    let store = builder.build().unwrap();
    let instance = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/control_scope_flat.ui", &store)
        .unwrap()
        .into_template_instance();

    let first = node_with_control_id(&instance.root, "First");
    let second = node_with_control_id(&instance.root, "Second");
    let first_source = first.children.first().expect("first source");
    let second_source = second.children.first().expect("second source");

    assert_ne!(first_source.control_id, second_source.control_id);
    assert_binding_reads_control(first, "value", first_source);
    assert_binding_reads_control(second, "value", second_source);
    assert_binding_reads_control(first, "root_value", first);
    assert_binding_reads_control(second, "root_value", second);
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

fn assert_binding_reads_control(
    owner: &UiTemplateNode,
    payload_key: &str,
    source: &UiTemplateNode,
) {
    let expression = owner.bindings[0]
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
