use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_node_creation, search_material_graph_node_palette,
    MaterialGraphConnectionAction as Action, MaterialGraphDiagnostic,
    MaterialGraphDiagnosticTarget, MaterialGraphNodeCreationPlan,
    MaterialGraphValueType as ValueType,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn material_graph_autowire_unconnected_creation_borrows_the_candidate() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("new_math", Node::Add);
    let result = propose(&graph, &candidate, "add", None).unwrap();
    assert!(std::ptr::eq(result.node(), &candidate));
    assert!(result.link().is_none());
    assert!(result.connection().is_none());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_autowire_new_output_to_existing_input_reaches_scalar_compilation() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("roughness_1", scalar("Roughness", 0.35));
    let wire = link("roughness_1", "value", "output", "roughness");
    let result = propose(&graph, &candidate, "scalar_parameter", Some(&wire)).unwrap();
    let connection = result.connection().unwrap();
    assert_eq!(connection.action, Action::Connect);
    assert_eq!(connection.source_type, ValueType::Scalar);
    assert!(!connection.has_undetermined_types);
    assert!(std::ptr::eq(result.link().unwrap(), &wire));
    let after = projected_runtime_dto(&graph, result);
    assert_eq!(graph, before);
    assert_eq!(
        compile_material_graph(&after)
            .unwrap()
            .standard_material_descriptor()
            .roughness,
        0.35
    );
}

#[test]
fn material_graph_autowire_occupied_input_retains_the_exact_old_edge() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node(
        "tint",
        Node::VectorParameter {
            name: "Tint".into(),
            default: [0.8, 0.1, 0.2, 1.0],
        },
    );
    let wire = link("tint", "value", "output", "base_color");
    let result = propose(&graph, &candidate, "vector_parameter", Some(&wire)).unwrap();
    let connection = result.connection().unwrap();
    assert!(
        matches!(connection.action, Action::Replace(old) if std::ptr::eq(old, &graph.links[0]))
    );
    assert_eq!(connection.source_type, ValueType::Vector);
    assert_eq!(graph, before);
    let after = projected_runtime_dto(&graph, result);
    assert_eq!(after.links, [wire]);
    assert_eq!(
        compile_material_graph(&after).unwrap().base_color,
        [0.8, 0.1, 0.2, 1.0]
    );
}

#[test]
fn material_graph_autowire_existing_output_to_new_math_allows_incomplete_inputs() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("sum", Node::Add);
    let wire = link("gain", "value", "sum", "a");
    let result = propose(&graph, &candidate, "add", Some(&wire)).unwrap();
    let connection = result.connection().unwrap();
    assert_eq!(connection.source_type, ValueType::Scalar);
    assert!(connection.has_undetermined_types);
    assert_eq!(connection.action, Action::Connect);
    let mut after = projected_runtime_dto(&graph, result);
    assert_eq!(graph, before);
    after.links.extend([
        link("gain", "value", "sum", "b"),
        link("sum", "value", "output", "roughness"),
    ]);
    assert_eq!(compile_material_graph(&after).unwrap().roughness, 0.5);
}

#[test]
fn material_graph_autowire_rejects_unrelated_self_and_missing_endpoints() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("new_math", Node::Add);
    for wire in [
        link("base", "value", "output", "base_color"),
        link("new_math", "value", "new_math", "a"),
        link("missing", "value", "new_math", "a"),
    ] {
        let errors = propose(&graph, &candidate, "add", Some(&wire)).unwrap_err();
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert_eq!(graph, before);
    }
}

#[test]
fn material_graph_autowire_rejects_wrong_pins_types_and_creation_errors() {
    let graph = graph();
    let before = graph.clone();
    let cases = [
        (
            node(
                "vector",
                Node::VectorParameter {
                    name: "Vector".into(),
                    default: [1.0; 4],
                },
            ),
            "vector_parameter",
            link("vector", "value", "output", "roughness"),
        ),
        (
            node("new_scalar", scalar("Scalar", 0.2)),
            "scalar_parameter",
            link("new_scalar", "Value display name", "output", "roughness"),
        ),
        (
            node("gain", scalar("Different parameter", 0.2)),
            "scalar_parameter",
            link("gain", "value", "output", "roughness"),
        ),
    ];
    for (candidate, key, wire) in cases {
        assert!(propose(&graph, &candidate, key, Some(&wire)).is_err());
        assert_eq!(graph, before);
    }
}

#[test]
fn material_graph_autowire_replacement_can_repair_an_existing_cycle() {
    let mut graph = graph();
    graph
        .nodes
        .extend([node("first", Node::Add), node("second", Node::Multiply)]);
    graph.links.extend([
        link("second", "value", "first", "a"),
        link("gain", "value", "first", "b"),
        link("first", "value", "second", "a"),
        link("gain", "value", "second", "b"),
        link("second", "value", "output", "roughness"),
    ]);
    assert!(compile_material_graph(&graph).is_err());
    let before = graph.clone();
    let candidate = node("seed", scalar("Seed", 0.5));
    let wire = link("seed", "value", "first", "a");
    let result = propose(&graph, &candidate, "scalar_parameter", Some(&wire)).unwrap();
    assert!(matches!(result.connection().unwrap().action,
        Action::Replace(old) if std::ptr::eq(old, &graph.links[1])));
    let after = projected_runtime_dto(&graph, result);
    assert_eq!(compile_material_graph(&after).unwrap().roughness, 0.1875);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_autowire_texture_requires_resolution_and_keeps_normal_reference() {
    let graph = graph();
    let before = graph.clone();
    let texture = reference("res://textures/rock_normal.png");
    let candidate = node(
        "normal",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    let wire = link("normal", "color", "output", "normal");
    let action = search_material_graph_node_palette("texture_sample").remove(0);
    let mut calls = 0;
    let result =
        plan_material_graph_node_creation(&graph, &action, &candidate, Some(&wire), |actual| {
            calls += 1;
            assert_eq!(actual, &texture);
            Ok(ResourceKind::Texture)
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(result.connection().unwrap().source_type, ValueType::Texture);
    let after = projected_runtime_dto(&graph, result);
    assert_eq!(
        compile_material_graph(&after).unwrap().normal_texture,
        Some(texture)
    );
    assert!(
        plan_material_graph_node_creation(&graph, &action, &candidate, Some(&wire), |_| Err(
            "not in the current catalog".into()
        ),)
        .is_err()
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_autowire_roundtrip_preserves_node_and_edge_schema_keys() {
    let graph = graph();
    let mut candidate = node("基色_节点_1", scalar("岩石基色", 0.6));
    candidate.position = [-16.25, 128.5];
    let wire = link("基色_节点_1", "value", "output", "base_color");
    let result = propose(&graph, &candidate, "scalar_parameter", Some(&wire)).unwrap();
    let after = projected_runtime_dto(&graph, result);
    let text = after.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&text).unwrap();
    assert_eq!(reopened, after);
    assert_eq!(reopened.nodes.last(), Some(&candidate));
    assert_eq!(reopened.links.last(), Some(&wire));
    assert_eq!(
        compile_material_graph(&reopened).unwrap().base_color,
        [0.6, 0.6, 0.6, 1.0]
    );
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/autowire.material_graph.toml").unwrap(),
        name: "Autowire / 自动连线".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: vec![
            node(
                "base",
                Node::VectorParameter {
                    name: "Base Color".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            ),
            node("gain", scalar("Gain", 0.25)),
            node("output", Node::Output),
        ],
        links: vec![link("base", "value", "output", "base_color")],
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position: [0.0; 2],
        kind,
    }
}

fn scalar(name: &str, default: f32) -> Node {
    Node::ScalarParameter {
        name: name.into(),
        default,
    }
}

fn link(from: &str, from_pin: &str, to: &str, to_pin: &str) -> Link {
    Link {
        from_node: from.into(),
        from_pin: from_pin.into(),
        to_node: to.into(),
        to_pin: to_pin.into(),
    }
}

fn propose<'a>(
    graph: &'a MaterialGraphAsset,
    candidate: &'a MaterialGraphNodeAsset,
    key: &str,
    wire: Option<&'a Link>,
) -> Result<MaterialGraphNodeCreationPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    let action = search_material_graph_node_palette(key)
        .into_iter()
        .find(|action| action.id() == key)
        .unwrap();
    plan_material_graph_node_creation(graph, &action, candidate, wire, |_| {
        panic!("non-texture candidates must not resolve assets")
    })
}

/// Unit-only DTO projection for real codec/compiler compatibility. This does not
/// implement or prove a host transaction, undo/redo, renderer or retained permit.
fn projected_runtime_dto(
    graph: &MaterialGraphAsset,
    plan: MaterialGraphNodeCreationPlan<'_>,
) -> MaterialGraphAsset {
    let mut after = graph.clone();
    after.nodes.push(plan.node().clone());
    if let Some(connection) = plan.connection() {
        match connection.action {
            Action::Replace(old) => {
                let count = after.links.len();
                after.links.retain(|edge| edge != old);
                assert_eq!(after.links.len(), count - 1);
            }
            Action::Connect => {}
            Action::Unchanged => panic!("fresh node autowire must not be an unchanged edge"),
        }
    }
    if let Some(link) = plan.link() {
        after.links.push(link.clone());
    }
    after
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
