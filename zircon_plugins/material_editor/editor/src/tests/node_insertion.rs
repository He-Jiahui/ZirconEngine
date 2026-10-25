use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_node_insertion, search_material_graph_node_palette,
    validate_material_graph_with_diagnostics, MaterialGraphDiagnostic,
    MaterialGraphDiagnosticTarget as Target, MaterialGraphNodeInsertionPlan,
};
use zircon_editor::ui::graph::GraphPortRef;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};

#[test]
fn material_graph_insertion_borrows_exact_old_edge_and_preserves_other_fanout() {
    let mut graph = graph();
    graph
        .links
        .push(link("base", "value", "output", "emissive"));
    let before = graph.clone();
    let candidate = node("tint", Node::Multiply);
    let anchor = GraphPortRef::input("output".into(), "base_color");
    let plan = propose(&graph, &candidate, "multiply", &anchor).unwrap();
    assert!(std::ptr::eq(plan.node(), &candidate));
    assert!(std::ptr::eq(plan.removed_link(), &graph.links[0]));
    assert!(plan.reconnects_previous_source());
    assert_eq!(
        plan.links(),
        &[
            link("base", "value", "tint", "a"),
            link("tint", "value", "output", "base_color"),
        ]
    );
    assert_eq!(graph, before);

    let mut after = project(&graph, &plan);
    after.links.push(link("gain", "value", "tint", "b"));
    assert!(after.links.contains(&graph.links[1]));
    let material = compile_material_graph(&after).unwrap();
    // Preserve the existing compiler's RGB scalar broadcast with alpha 1.0.
    assert_eq!(material.base_color, [0.05, 0.1, 0.2, 1.0]);
    assert_eq!(
        material.standard_material_descriptor().emissive,
        [0.2, 0.4, 0.8]
    );
}

#[test]
fn material_graph_insertion_keeps_incomplete_math_then_roundtrips_real_pbr_value() {
    let mut graph = graph();
    graph
        .links
        .push(link("gain", "value", "output", "roughness"));
    let candidate = node("插入_粗糙度", Node::Add);
    let plan = propose(
        &graph,
        &candidate,
        "add",
        &GraphPortRef::input("output".into(), "roughness"),
    )
    .unwrap();
    assert!(plan.has_undetermined_types());
    assert_eq!(plan.links()[0].to_pin, "a");
    let mut after = project(&graph, &plan);
    assert!(validate_material_graph_with_diagnostics(&after)
        .iter()
        .any(|error| {
            error.target
                == Target::Input {
                    node_id: candidate.id.clone(),
                    pin_name: "b".into(),
                }
        }));
    after.links.push(link("gain", "value", &candidate.id, "b"));
    let reopened = MaterialGraphAsset::from_toml_str(&after.to_toml_string().unwrap()).unwrap();
    assert_eq!(reopened, after);
    assert_eq!(reopened.nodes.last(), Some(&candidate));
    assert_eq!(
        compile_material_graph(&reopened)
            .unwrap()
            .standard_material_descriptor()
            .roughness,
        0.5
    );
}

#[test]
fn material_graph_insertion_joint_scalar_constraint_uses_explicit_unwired_source_fallback() {
    let mut graph = graph();
    graph
        .links
        .push(link("base", "value", "output", "roughness"));
    let before = graph.clone();
    let candidate = node("new_math", Node::Multiply);
    let plan = propose(
        &graph,
        &candidate,
        "multiply",
        &GraphPortRef::input("output".into(), "roughness"),
    )
    .unwrap();
    // Checking only new_math -> roughness would see an undetermined node and
    // miss the vector source restored by the other edge of the same insertion.
    assert!(!plan.reconnects_previous_source());
    assert!(plan.has_undetermined_types());
    assert_eq!(
        plan.links(),
        &[link("new_math", "value", "output", "roughness")]
    );
    assert_eq!(graph, before);
    let after = project(&graph, &plan);
    let missing_inputs = validate_material_graph_with_diagnostics(&after)
        .into_iter()
        .filter(
            |error| matches!(&error.target, Target::Input { node_id, .. } if node_id == "new_math"),
        )
        .count();
    assert_eq!(missing_inputs, 2);
    assert!(after.nodes.iter().any(|node| node.id == "base"));
}

#[test]
fn material_graph_insertion_does_not_convert_texture_normal_into_math_support() {
    let mut graph = graph();
    graph.nodes.push(node(
        "normal_map",
        Node::TextureSample {
            texture: reference("res://textures/normal.texture.toml"),
        },
    ));
    graph
        .links
        .push(link("normal_map", "color", "output", "normal"));
    let before = graph.clone();
    let errors = propose(
        &graph,
        &node("new_math", Node::Add),
        "add",
        &GraphPortRef::input("output".into(), "normal"),
    )
    .unwrap_err();
    assert!(errors[0].message.contains("requires a texture sample"));
    assert_eq!(errors[0].target, input("output", "normal"));
    assert_eq!(graph, before);
    assert!(compile_material_graph(&graph)
        .unwrap()
        .normal_texture
        .is_some());
}

#[test]
fn material_graph_insertion_no_receiving_pin_returns_ue_replacement_and_keeps_old_node() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node(
        "replacement",
        Node::ScalarParameter {
            name: "New Value".into(),
            default: 0.8,
        },
    );
    let plan = propose(
        &graph,
        &candidate,
        "scalar_parameter",
        &GraphPortRef::input("output".into(), "base_color"),
    )
    .unwrap();
    assert!(!plan.reconnects_previous_source());
    assert_eq!(
        plan.links(),
        &[link("replacement", "value", "output", "base_color")]
    );
    assert_eq!(graph, before);
    let after = project(&graph, &plan);
    assert!(after.nodes.iter().any(|node| node == &graph.nodes[0]));
    assert_eq!(
        compile_material_graph(&after).unwrap().base_color,
        [0.8, 0.8, 0.8, 1.0]
    );
}

#[test]
fn material_graph_insertion_rejects_invalid_anchor_or_occupancy_before_resource_resolution() {
    let base = graph();
    let mut multiple = base.clone();
    multiple
        .links
        .push(link("gain", "value", "output", "base_color"));
    let mut ambiguous = base.clone();
    ambiguous.nodes.push(node("output", Node::Output));
    let cases = [
        (
            base.clone(),
            GraphPortRef::output("gain".into(), "value"),
            Target::Graph,
        ),
        (
            base.clone(),
            GraphPortRef::input("missing".into(), "base_color"),
            Target::Graph,
        ),
        (
            base.clone(),
            GraphPortRef::input("output".into(), "display label"),
            Target::Node {
                node_id: "output".into(),
            },
        ),
        (
            base.clone(),
            GraphPortRef::input("output".into(), "roughness"),
            input("output", "roughness"),
        ),
        (
            multiple,
            GraphPortRef::input("output".into(), "base_color"),
            input("output", "base_color"),
        ),
        (
            ambiguous,
            GraphPortRef::input("output".into(), "base_color"),
            Target::Graph,
        ),
    ];
    let candidate = node(
        "sample",
        Node::TextureSample {
            texture: reference("res://textures/albedo.texture.toml"),
        },
    );
    for (graph, anchor, target) in cases {
        let before = graph.clone();
        let action = search_material_graph_node_palette("texture_sample")
            .pop()
            .unwrap();
        let errors =
            plan_material_graph_node_insertion(&graph, &action, &candidate, &anchor, |_| {
                panic!("invalid source anchor must not resolve the new texture")
            })
            .unwrap_err();
        assert_eq!(errors[0].target, target);
        assert_eq!(graph, before);
    }
}

#[test]
fn material_graph_insertion_preserves_rejection_of_broken_sources_and_disconnected_cycles() {
    let mut missing = graph();
    missing.links[0].from_node = "missing".into();
    let mut wrong_pin = graph();
    wrong_pin.links[0].from_pin = "display label".into();
    let mut cyclic = graph();
    cyclic
        .nodes
        .extend([node("x", Node::Add), node("y", Node::Add)]);
    cyclic
        .links
        .extend([link("x", "value", "y", "a"), link("y", "value", "x", "a")]);
    for (graph, message) in [
        (missing, "source node is no longer present"),
        (wrong_pin, "source output no longer exists"),
        (cyclic, "cycle"),
    ] {
        let before = graph.clone();
        let errors = propose(
            &graph,
            &node("new_math", Node::Add),
            "add",
            &GraphPortRef::input("output".into(), "base_color"),
        )
        .unwrap_err();
        assert!(errors[0].message.contains(message), "{:?}", errors);
        assert_eq!(errors[0].target, input("output", "base_color"));
        assert_eq!(graph, before);
    }
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/insertion.material_graph.toml").unwrap(),
        name: "Insertion / 保留原连接".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: vec![
            node(
                "base",
                Node::VectorParameter {
                    name: "Base Color".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            ),
            node(
                "gain",
                Node::ScalarParameter {
                    name: "Gain".into(),
                    default: 0.25,
                },
            ),
            node("output", Node::Output),
        ],
        links: vec![link("base", "value", "output", "base_color")],
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position: [128.5, -24.25],
        kind,
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
    anchor: &GraphPortRef<String>,
) -> Result<MaterialGraphNodeInsertionPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    let action = search_material_graph_node_palette(key)
        .into_iter()
        .find(|action| action.id() == key)
        .unwrap();
    plan_material_graph_node_insertion(graph, &action, candidate, anchor, |_| {
        panic!("non-texture creation must not resolve assets")
    })
}

/// Unit-only projection into the actual codec/compiler DTO. No product mutation,
/// retained permit, undo/redo, rendered frame or source-bound host is simulated.
fn project(
    graph: &MaterialGraphAsset,
    plan: &MaterialGraphNodeInsertionPlan<'_>,
) -> MaterialGraphAsset {
    let mut after = graph.clone();
    let old_index = graph
        .links
        .iter()
        .position(|edge| std::ptr::eq(edge, plan.removed_link()))
        .unwrap();
    after.links.remove(old_index);
    after.nodes.push(plan.node().clone());
    after.links.extend(plan.links().iter().cloned());
    after
}

fn input(node: &str, pin: &str) -> Target {
    Target::Input {
        node_id: node.into(),
        pin_name: pin.into(),
    }
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
