use std::collections::BTreeMap;

use super::{
    can_connect_material_graph, compile_material_graph, MaterialGraphConnectionAction as Action,
    MaterialGraphValueType as ValueType,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};

#[test]
fn material_graph_connection_scalar_broadcast_and_direct_normal_match_compiler() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "gain", 0.25);
    let texture = reference("res://textures/normal.png");
    add_node(
        &mut graph,
        "normal",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    let before = graph.clone();
    let candidate = link("gain", "value", "output", "base_color");
    let plan = can_connect_material_graph(&graph, &candidate).unwrap();
    assert!(matches!(plan.action, Action::Replace(edge) if std::ptr::eq(edge, &graph.links[0])));
    assert_eq!(plan.source_type, ValueType::Scalar);
    assert!(!plan.has_undetermined_types);
    assert_eq!(
        graph, before,
        "preflight must not change the authoring document"
    );

    let mut prospective = with_candidate(&graph, &candidate);
    let normal = link("normal", "color", "output", "normal");
    assert_eq!(
        can_connect_material_graph(&prospective, &normal)
            .unwrap()
            .action,
        Action::Connect
    );
    prospective.links.push(normal);
    let material = compile_material_graph(&prospective).unwrap();
    assert_eq!(material.base_color, [0.25, 0.25, 0.25, 1.0]);
    assert_eq!(material.normal_texture, Some(texture));
}

#[test]
fn material_graph_connection_rejects_vector_and_texture_for_scalar_outputs() {
    let mut graph = base_graph();
    add_node(
        &mut graph,
        "texture",
        Node::TextureSample {
            texture: reference("res://textures/a.png"),
        },
    );
    for candidate in [
        link("base", "value", "output", "roughness"),
        link("texture", "color", "output", "metallic"),
    ] {
        assert_rejected_by_preflight_and_compiler(&graph, &candidate, "does not produce a scalar");
    }
}

#[test]
fn material_graph_connection_rejects_texture_arithmetic_and_computed_normals() {
    let mut graph = base_graph();
    add_node(
        &mut graph,
        "texture",
        Node::TextureSample {
            texture: reference("res://textures/a.png"),
        },
    );
    add_scalar(&mut graph, "gain", 1.0);
    add_node(&mut graph, "sum", Node::Add);
    graph.links.push(link("gain", "value", "sum", "a"));
    graph.links.push(link("gain", "value", "sum", "b"));
    let candidate = link("texture", "color", "sum", "a");
    assert_rejected_by_preflight_and_compiler(
        &graph,
        &candidate,
        "cannot combine texture-backed inputs in v1",
    );
    let candidate = link("sum", "value", "output", "normal");
    assert_rejected_by_preflight_and_compiler(
        &graph,
        &candidate,
        "`normal` requires a texture sample in v1",
    );
}

#[test]
fn material_graph_connection_replacement_can_repair_a_cycle_before_validation() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "gain", 0.5);
    add_node(&mut graph, "first", Node::Add);
    add_node(&mut graph, "second", Node::Multiply);
    graph.links.extend([
        link("second", "value", "first", "a"),
        link("gain", "value", "first", "b"),
        link("first", "value", "second", "a"),
        link("gain", "value", "second", "b"),
        link("second", "value", "output", "roughness"),
    ]);
    assert!(compile_material_graph(&graph).is_err());
    let before = graph.clone();
    let candidate = link("gain", "value", "first", "a");
    let plan = can_connect_material_graph(&graph, &candidate).unwrap();
    assert!(matches!(plan.action, Action::Replace(edge) if std::ptr::eq(edge, &graph.links[1])));
    assert_eq!(graph, before);
    let material = compile_material_graph(&with_candidate(&graph, &candidate)).unwrap();
    assert_eq!(material.roughness, 0.5);
}

#[test]
fn material_graph_connection_distinguishes_unchanged_and_ambiguous_inputs() {
    let mut graph = base_graph();
    assert_eq!(
        can_connect_material_graph(&graph, &graph.links[0])
            .unwrap()
            .action,
        Action::Unchanged
    );
    add_scalar(&mut graph, "gain", 1.0);
    graph
        .links
        .push(link("gain", "value", "output", "base_color"));
    let before = graph.clone();
    let error = can_connect_material_graph(&graph, &graph.links[0]).unwrap_err();
    assert!(error.contains("multiple links target `output.base_color`"));
    assert_eq!(graph, before);
}

#[test]
fn material_graph_connection_rejects_invalid_endpoints_self_links_and_cycles() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "gain", 1.0);
    add_node(&mut graph, "first", Node::Add);
    add_node(&mut graph, "second", Node::Multiply);
    graph.links.extend([
        link("gain", "value", "first", "a"),
        link("gain", "value", "first", "b"),
        link("first", "value", "second", "a"),
        link("gain", "value", "second", "b"),
    ]);
    for (candidate, reason) in [
        (
            link("absent", "value", "output", "roughness"),
            "missing source node",
        ),
        (link("gain", "value", "absent", "a"), "missing target node"),
        (
            link("gain", "label", "output", "roughness"),
            "no output pin",
        ),
        (link("gain", "value", "output", "roughnes"), "no input pin"),
        (
            link("output", "base_color", "gain", "value"),
            "no output pin",
        ),
        (link("first", "value", "first", "a"), "self connection"),
        (link("second", "value", "first", "a"), "contains a cycle"),
    ] {
        let before = graph.clone();
        let error = can_connect_material_graph(&graph, &candidate).unwrap_err();
        assert!(error.contains(reason), "expected {reason:?}, got {error:?}");
        assert_eq!(graph, before);
    }
}

#[test]
fn material_graph_connection_tracks_incomplete_math_and_downstream_scalar_constraints() {
    let mut graph = base_graph();
    add_node(&mut graph, "sum", Node::Add);
    let downstream = link("sum", "value", "output", "roughness");
    let plan = can_connect_material_graph(&graph, &downstream).unwrap();
    assert_eq!(plan.source_type, ValueType::Undetermined);
    assert!(plan.has_undetermined_types);
    graph.links.push(downstream);
    assert!(
        compile_material_graph(&graph).is_err(),
        "connectability is not compile readiness"
    );
    let vector_input = link("base", "value", "sum", "a");
    let before = graph.clone();
    let error = can_connect_material_graph(&graph, &vector_input).unwrap_err();
    assert!(error.contains("node `sum` does not produce a scalar"));
    assert_eq!(graph, before);

    add_scalar(&mut graph, "gain", 0.25);
    let scalar_input = link("gain", "value", "sum", "a");
    let plan = can_connect_material_graph(&graph, &scalar_input).unwrap();
    assert!(plan.has_undetermined_types);
    graph.links.push(scalar_input);
    let final_input = link("gain", "value", "sum", "b");
    assert!(
        !can_connect_material_graph(&graph, &final_input)
            .unwrap()
            .has_undetermined_types
    );
    graph.links.push(final_input);
    assert_eq!(compile_material_graph(&graph).unwrap().roughness, 0.5);
}

#[test]
fn material_graph_connection_preserves_vector_math_and_source_fanout() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "gain", 0.25);
    add_node(&mut graph, "sum", Node::Add);
    graph.links.push(link("base", "value", "sum", "a"));
    let candidate = link("gain", "value", "sum", "b");
    let plan = can_connect_material_graph(&graph, &candidate).unwrap();
    assert_eq!(plan.action, Action::Connect);
    assert_eq!(plan.source_type, ValueType::Scalar);
    assert!(!plan.has_undetermined_types);
    graph.links.push(candidate);
    let output = link("sum", "value", "output", "base_color");
    let plan = can_connect_material_graph(&graph, &output).unwrap();
    assert_eq!(plan.source_type, ValueType::Vector);
    let prospective = with_candidate(&graph, &output);
    let material = compile_material_graph(&prospective).unwrap();
    assert_eq!(material.base_color, [0.45, 0.65, 1.05, 2.0]);
    assert!(prospective
        .links
        .iter()
        .any(|edge| edge.from_node == "base" && edge.to_node == "sum"));
    assert!(prospective
        .links
        .iter()
        .any(|edge| edge.from_node == "gain" && edge.to_node == "sum"));
}

#[test]
fn material_graph_connection_rejects_duplicate_or_blank_node_identity() {
    let mut graph = base_graph();
    let candidate = graph.links[0].clone();
    graph.nodes.push(graph.nodes[0].clone());
    assert!(can_connect_material_graph(&graph, &candidate)
        .unwrap_err()
        .contains("duplicate node `base`"));
    graph.nodes.last_mut().unwrap().id = " ".into();
    assert!(can_connect_material_graph(&graph, &candidate)
        .unwrap_err()
        .contains("node id must not be empty"));
}

fn assert_rejected_by_preflight_and_compiler(
    graph: &MaterialGraphAsset,
    candidate: &Link,
    reason: &str,
) {
    let before = graph.clone();
    let error = can_connect_material_graph(graph, candidate).unwrap_err();
    assert!(error.contains(reason), "expected {reason:?}, got {error:?}");
    assert_eq!(*graph, before);
    let diagnostics = compile_material_graph(&with_candidate(graph, candidate)).unwrap_err();
    assert!(
        diagnostics.iter().any(|error| error.contains(reason)),
        "expected {reason:?}, got {diagnostics:?}"
    );
}

// Test fixture projection only: production application needs retained revision,
// prepare, one permit and an inverse in the host's existing transaction history.
fn with_candidate(graph: &MaterialGraphAsset, candidate: &Link) -> MaterialGraphAsset {
    let mut prospective = graph.clone();
    prospective
        .links
        .retain(|edge| edge.to_node != candidate.to_node || edge.to_pin != candidate.to_pin);
    prospective.links.push(candidate.clone());
    prospective
}

fn base_graph() -> MaterialGraphAsset {
    let mut graph = MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/connections.material_graph.toml").unwrap(),
        name: "连线 / Material".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: Vec::new(),
        links: Vec::new(),
        parameters: BTreeMap::new(),
    };
    add_node(
        &mut graph,
        "base",
        Node::VectorParameter {
            name: "base".into(),
            default: [0.2, 0.4, 0.8, 1.0],
        },
    );
    add_node(&mut graph, "output", Node::Output);
    graph
        .links
        .push(link("base", "value", "output", "base_color"));
    graph
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

fn link(source: &str, from_pin: &str, target: &str, to_pin: &str) -> Link {
    Link {
        from_node: source.into(),
        from_pin: from_pin.into(),
        to_node: target.into(),
        to_pin: to_pin.into(),
    }
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
