use std::collections::BTreeMap;

use super::{compile_material_graph, compile_material_graph_operation};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Parameter,
};

#[test]
fn material_graph_shared_dag_compiles_all_outputs_without_exponential_expansion() {
    let mut graph = graph();
    add_scalar(&mut graph, "one", 1.0);
    let mut previous = "one".to_string();
    for index in 0..64 {
        let id = format!("square_{index}");
        add_node(&mut graph, &id, Node::Multiply);
        connect(&mut graph, &previous, &id, "a");
        connect(&mut graph, &previous, &id, "b");
        previous = id;
    }
    for pin in ["base_color", "emissive", "metallic", "roughness"] {
        connect(&mut graph, &previous, "output", pin);
    }

    let material = compile_material_graph(&graph).expect("shared DAG is finite and well typed");
    let descriptor = material.standard_material_descriptor();
    assert_eq!(descriptor.base_color, [1.0; 4]);
    assert_eq!(descriptor.emissive, [1.0; 3]);
    assert_eq!(descriptor.metallic, 1.0);
    assert_eq!(descriptor.roughness, 1.0);
    assert_eq!(graph.nodes.len(), 66);
    assert_eq!(graph.links.len(), 132);
}

#[test]
fn material_graph_deep_expression_compiles_without_recursive_stack_growth() {
    let mut graph = graph();
    add_scalar(&mut graph, "one", 1.0);
    add_node(
        &mut graph,
        "tint",
        Node::VectorParameter {
            name: "tint".into(),
            default: [0.125, 0.25, 0.5, 1.0],
        },
    );
    let mut previous = "tint".to_string();
    for index in 0..4096 {
        let id = format!("multiply_{index}");
        add_node(&mut graph, &id, Node::Multiply);
        connect(&mut graph, &previous, &id, "a");
        connect(&mut graph, "one", &id, "b");
        previous = id;
    }
    connect(&mut graph, &previous, "output", "base_color");
    connect(&mut graph, &previous, "output", "emissive");

    let material = compile_material_graph(&graph).expect("deep valid expression");
    assert_eq!(material.base_color, [0.125, 0.25, 0.5, 1.0]);
    assert_eq!(material.emissive, [0.125, 0.25, 0.5]);
}

#[test]
fn material_graph_evaluation_cache_does_not_survive_a_parameter_edit() {
    let mut graph = graph();
    add_scalar(&mut graph, "gain", 0.5);
    add_node(&mut graph, "square", Node::Multiply);
    connect(&mut graph, "gain", "square", "a");
    connect(&mut graph, "gain", "square", "b");
    for pin in ["base_color", "emissive", "metallic", "roughness"] {
        connect(&mut graph, "square", "output", pin);
    }
    let before = graph.clone();
    let first = compile_material_graph(&graph).unwrap();
    assert_eq!(first.base_color, [0.25, 0.25, 0.25, 1.0]);
    assert_eq!(first.roughness, 0.25);
    assert_eq!(
        graph, before,
        "compile must not edit the authoring document"
    );

    graph
        .parameters
        .insert("gain".into(), Parameter::Scalar(0.25));
    let edited = graph.clone();
    let second = compile_material_graph(&graph).unwrap();
    assert_eq!(second.base_color, [0.0625, 0.0625, 0.0625, 1.0]);
    assert_eq!(second.emissive, [0.0625; 3]);
    assert_eq!(second.metallic, 0.0625);
    assert_eq!(second.roughness, 0.0625);
    assert_eq!(graph, edited);
    assert_eq!(first.roughness, 0.25);
}

#[test]
fn material_graph_disconnected_overflow_is_evaluated_only_after_output_connection() {
    let mut graph = graph();
    add_scalar(&mut graph, "one", 1.0);
    add_scalar(&mut graph, "large", f32::MAX);
    add_node(&mut graph, "overflow", Node::Add);
    connect(&mut graph, "large", "overflow", "a");
    connect(&mut graph, "large", "overflow", "b");
    connect(&mut graph, "one", "output", "base_color");
    assert!(compile_material_graph(&graph).is_ok());

    connect(&mut graph, "overflow", "output", "roughness");
    let report = compile_material_graph_operation(&graph);
    assert!(report.material.is_none());
    assert!(report.diagnostics.iter().any(|message| {
        message.contains("output.roughness")
            && message.contains("node `overflow` produced a non-finite scalar")
    }));
    graph.links.pop();
    graph.links[2].from_node = "overflow".into();
    let report = compile_material_graph_operation(&graph);
    assert!(report.material.is_none());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("`add` produced a non-finite value")));
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/evaluation.material_graph.toml").unwrap(),
        name: "Evaluation regression".into(),
        shader: Some(AssetReference::from_locator(
            AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        )),
        nodes: vec![MaterialGraphNodeAsset {
            id: "output".into(),
            position: [0.0; 2],
            kind: Node::Output,
        }],
        links: Vec::new(),
        parameters: BTreeMap::new(),
    }
}

fn add_scalar(graph: &mut MaterialGraphAsset, id: &str, default: f32) {
    add_node(
        graph,
        id,
        Node::ScalarParameter {
            name: id.into(),
            default,
        },
    );
}

fn add_node(graph: &mut MaterialGraphAsset, id: &str, kind: Node) {
    graph.nodes.push(MaterialGraphNodeAsset {
        id: id.into(),
        position: [0.0; 2],
        kind,
    });
}

fn connect(graph: &mut MaterialGraphAsset, source: &str, target: &str, pin: &str) {
    graph.links.push(MaterialGraphLinkAsset {
        from_node: source.into(),
        from_pin: "value".into(),
        to_node: target.into(),
        to_pin: pin.into(),
    });
}
