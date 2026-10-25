use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_node_creation, search_material_graph_node_palette,
    suggest_material_graph_autowire, MaterialGraphConnectionAction,
    MaterialGraphDiagnosticTarget as Target,
};
use zircon_editor::{
    core::editor_authoring_extension::GraphNodeDescriptor, ui::graph::GraphPortRef,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn material_graph_autowire_pin_uses_first_registered_input_and_real_compiler() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("sum", Node::Add);
    let anchor = GraphPortRef::output("gain".into(), "value");
    let action = action("add");
    let wire = suggest_material_graph_autowire(&graph, &action, &candidate, &anchor, |_| {
        panic!("math must not resolve a texture")
    })
    .unwrap()
    .unwrap();
    assert_eq!(wire, link("gain", "value", "sum", "a"));
    let plan = plan_material_graph_node_creation(&graph, &action, &candidate, Some(&wire), |_| {
        panic!("math must not resolve a texture")
    })
    .unwrap();
    assert_eq!(
        plan.connection().unwrap().action,
        MaterialGraphConnectionAction::Connect
    );
    assert!(plan.connection().unwrap().has_undetermined_types);
    assert_eq!(graph, before);

    // Unit-only projection into the real DTO/compiler, not a host edit/history.
    let mut after = graph.clone();
    after.nodes.push(candidate);
    after.links.extend([
        wire,
        link("gain", "value", "sum", "b"),
        link("sum", "value", "output", "roughness"),
    ]);
    assert_eq!(
        compile_material_graph(&after)
            .unwrap()
            .standard_material_descriptor()
            .roughness,
        0.5
    );
}

#[test]
fn material_graph_autowire_pin_input_uses_new_output_and_preserves_source_key() {
    let graph = graph();
    let before = graph.clone();
    let candidate = node("粗糙度/long-source-key", scalar("粗糙度", 0.35));
    let wire = suggest_material_graph_autowire(
        &graph,
        &action("scalar_parameter"),
        &candidate,
        &GraphPortRef::input("output".into(), "roughness"),
        |_| panic!("scalar must not resolve a texture"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(wire, link(&candidate.id, "value", "output", "roughness"));
    assert_eq!(graph, before);
    let mut after = graph;
    after.nodes.push(candidate);
    after.links.push(wire);
    assert_eq!(
        compile_material_graph(&after)
            .unwrap()
            .standard_material_descriptor()
            .roughness,
        0.35
    );
}

#[test]
fn material_graph_autowire_pin_vector_math_uses_domain_types_instead_of_palette_float() {
    let graph = graph();
    let candidate = node("tint", Node::Multiply);
    let action = action("multiply");
    assert_eq!(action.inputs()[0].value_type(), "float");
    let wire = suggest_material_graph_autowire(
        &graph,
        &action,
        &candidate,
        &GraphPortRef::output("base".into(), "value"),
        |_| panic!("math must not resolve a texture"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(wire, link("base", "value", "tint", "a"));
    let mut after = graph;
    after.nodes.push(candidate);
    after.links.extend([
        wire,
        link("gain", "value", "tint", "b"),
        link("tint", "value", "output", "emissive"),
    ]);
    assert_eq!(
        compile_material_graph(&after)
            .unwrap()
            .standard_material_descriptor()
            .emissive,
        [0.05, 0.1, 0.2]
    );
}

#[test]
fn material_graph_autowire_pin_texture_selects_actual_output_channel_and_rejects_math() {
    let mut graph = graph();
    graph
        .nodes
        .retain(|node| !matches!(node.kind, Node::Output));
    graph.links.clear();
    let texture = reference("res://textures/真实-albedo.texture.toml");
    graph.nodes.push(node(
        "texture",
        Node::TextureSample {
            texture: texture.clone(),
        },
    ));
    let anchor = GraphPortRef::output("texture".into(), "color");
    let candidate = node("material-output", Node::Output);
    let wire =
        suggest_material_graph_autowire(&graph, &action("output"), &candidate, &anchor, |_| {
            panic!("existing source references are not new candidate resources")
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        wire,
        link("texture", "color", "material-output", "base_color")
    );
    let before = graph.clone();
    assert!(suggest_material_graph_autowire(
        &graph,
        &action("add"),
        &node("sum", Node::Add),
        &anchor,
        |_| unreachable!()
    )
    .unwrap()
    .is_none());
    assert_eq!(graph, before);
    graph.nodes.push(candidate);
    graph.links.push(wire);
    assert_eq!(
        compile_material_graph(&graph)
            .unwrap()
            .base_color_texture
            .as_ref(),
        Some(&texture)
    );
}

#[test]
fn material_graph_autowire_pin_rejects_stale_ambiguous_and_wrong_direction_before_resolution() {
    let graph = graph();
    let candidate = node(
        "sample",
        Node::TextureSample {
            texture: reference("res://textures/albedo.texture.toml"),
        },
    );
    for (anchor, target) in [
        (
            GraphPortRef::input("missing".into(), "roughness"),
            Target::Graph,
        ),
        (
            GraphPortRef::output("output".into(), "base_color"),
            Target::Node {
                node_id: "output".into(),
            },
        ),
        (
            GraphPortRef::input("output".into(), "display label"),
            Target::Node {
                node_id: "output".into(),
            },
        ),
    ] {
        let before = graph.clone();
        let errors = suggest_material_graph_autowire(
            &graph,
            &action("texture_sample"),
            &candidate,
            &anchor,
            |_| panic!("stale source port must not resolve candidate resources"),
        )
        .unwrap_err();
        assert_eq!(errors[0].target, target);
        assert_eq!(graph, before);
    }
    let mut ambiguous = graph;
    ambiguous.nodes.push(ambiguous.nodes[0].clone());
    let errors = suggest_material_graph_autowire(
        &ambiguous,
        &action("texture_sample"),
        &candidate,
        &GraphPortRef::output("base".into(), "value"),
        |_| panic!("ambiguous source must not resolve resources"),
    )
    .unwrap_err();
    assert_eq!(errors[0].target, Target::Graph);
}

#[test]
fn material_graph_autowire_pin_occupied_input_requires_explicit_insertion_without_dropping_edge() {
    let graph = graph();
    let before = graph.clone();
    let errors = suggest_material_graph_autowire(
        &graph,
        &action("multiply"),
        &node("inserted", Node::Multiply),
        &GraphPortRef::input("output".into(), "base_color"),
        |_| unreachable!(),
    )
    .unwrap_err();
    assert_eq!(
        errors[0].target,
        Target::Input {
            node_id: "output".into(),
            pin_name: "base_color".into()
        }
    );
    assert!(errors[0].message.contains("multi-edge insertion"));
    assert_eq!(graph, before);
}

#[test]
fn material_graph_autowire_pin_no_opposite_or_compatible_pin_keeps_valid_creation_unwired() {
    let graph = graph();
    let before = graph.clone();
    assert!(suggest_material_graph_autowire(
        &graph,
        &action("scalar_parameter"),
        &node("other", scalar("Other", 0.5)),
        &GraphPortRef::output("gain".into(), "value"),
        |_| unreachable!(),
    )
    .unwrap()
    .is_none());

    let texture = reference("res://textures/albedo.texture.toml");
    let candidate = node(
        "sample",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    let mut resolved = Vec::new();
    let result = suggest_material_graph_autowire(
        &graph,
        &action("texture_sample"),
        &candidate,
        &GraphPortRef::input("output".into(), "roughness"),
        |actual| {
            resolved.push(actual.clone());
            Ok(ResourceKind::Texture)
        },
    )
    .unwrap();
    assert!(result.is_none());
    assert_eq!(resolved, [texture]);
    assert_eq!(graph, before);
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/pin-autowire.material_graph.toml").unwrap(),
        name: "Pin autowire / 端口选择".into(),
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
        position: [24.0, 48.0],
        kind,
    }
}

fn scalar(name: &str, default: f32) -> Node {
    Node::ScalarParameter {
        name: name.into(),
        default,
    }
}

fn action(id: &str) -> GraphNodeDescriptor {
    search_material_graph_node_palette(id)
        .into_iter()
        .find(|action| action.id() == id)
        .unwrap()
}

fn link(from: &str, from_pin: &str, to: &str, to_pin: &str) -> Link {
    Link {
        from_node: from.into(),
        from_pin: from_pin.into(),
        to_node: to.into(),
        to_pin: to_pin.into(),
    }
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
