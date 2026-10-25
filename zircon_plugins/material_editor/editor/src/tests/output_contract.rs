use std::collections::BTreeMap;

use super::{compile_material_graph, compile_material_graph_operation, validate_material_graph};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, MaterialGraphAsset, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
    MaterialGraphParameterAsset as Parameter, ZMaterialDocument,
};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};

#[test]
fn material_graph_pbr_outputs_reach_renderer_descriptor_and_persisted_material() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "roughness", 0.25);
    add_scalar(&mut graph, "scale", 2.0);
    add_scalar(&mut graph, "metallic", 0.6);
    add_node(&mut graph, "product", Node::Multiply);
    connect(&mut graph, "roughness", "value", "product", "a");
    connect(&mut graph, "scale", "value", "product", "b");
    connect(&mut graph, "product", "value", "output", "roughness");
    connect(&mut graph, "metallic", "value", "output", "metallic");
    graph
        .parameters
        .insert("roughness".into(), Parameter::Scalar(0.4));
    let normal = reference("res://textures/rock_normal.png");
    add_node(
        &mut graph,
        "normal",
        Node::TextureSample {
            texture: normal.clone(),
        },
    );
    connect(&mut graph, "normal", "color", "output", "normal");
    add_node(
        &mut graph,
        "emissive",
        Node::VectorParameter {
            name: "emissive".into(),
            default: [1.5, 0.25, 0.0, 1.0],
        },
    );
    connect(&mut graph, "emissive", "value", "output", "emissive");

    let material = compile_material_graph(&graph).expect("supported PBR graph");
    let descriptor = material.standard_material_descriptor();
    assert_eq!(descriptor.base_color, [0.2, 0.4, 0.8, 1.0]);
    assert_eq!(descriptor.roughness, 0.8);
    assert_eq!(descriptor.metallic, 0.6);
    assert_eq!(descriptor.normal_texture, Some(normal));
    assert_eq!(descriptor.emissive, [1.5, 0.25, 0.0]);

    let text = material
        .to_project_toml_string(|reference| {
            let locator = reference.locator.to_string();
            Ok(PersistedAssetReference::project(
                AssetRef::try_new(
                    reference.uuid,
                    RelPath::parse(locator.strip_prefix("res://").expect("project fixture"))
                        .unwrap(),
                    None,
                )
                .unwrap(),
            ))
        })
        .expect("project material document");
    let saved = MaterialAsset::from_zmaterial_document(
        ZMaterialDocument::from_project_toml_str(&text, |reference| {
            let project = reference.project_ref().expect("project fixture reference");
            Ok(AssetReference::new(
                project.guid(),
                AssetUri::parse(&format!("res://{}", project.path_hint())).unwrap(),
            ))
        })
        .expect("reopened project material document"),
    );
    assert_eq!(saved.standard_material_descriptor(), descriptor);
}

#[test]
fn material_graph_texture_emissive_preserves_texture_and_nonzero_factor() {
    let mut graph = base_graph();
    let texture = reference("res://textures/emissive.png");
    add_node(
        &mut graph,
        "emissive",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    connect(&mut graph, "emissive", "color", "output", "emissive");

    let material = compile_material_graph(&graph).expect("texture-backed emissive");
    assert_eq!(material.emissive_texture, Some(texture));
    assert_eq!(material.standard_material_descriptor().emissive, [1.0; 3]);
}

#[test]
fn material_graph_optional_pbr_outputs_preserve_unconnected_defaults() {
    let material = compile_material_graph(&base_graph()).expect("base-color-only graph");
    assert_eq!(material.metallic, 0.0);
    assert_eq!(material.roughness, 1.0);
    assert_eq!(material.emissive, [0.0; 3]);
    assert!(material.normal_texture.is_none());
    assert!(material.emissive_texture.is_none());
}

#[test]
fn material_graph_rejects_unknown_output_pin_instead_of_silently_ignoring_it() {
    let mut graph = base_graph();
    connect(&mut graph, "base", "value", "output", "roughnes");
    assert_error(&graph, "no input pin `output.roughnes`");
}

#[test]
fn material_graph_rejects_unknown_source_pin_instead_of_evaluating_the_node() {
    let mut graph = base_graph();
    graph.links[0].from_pin = "missing".into();
    assert_error(&graph, "no output pin `base.missing`");
}

#[test]
fn material_graph_rejects_connections_in_the_wrong_direction() {
    let mut graph = base_graph();
    connect(&mut graph, "output", "base_color", "base", "value");
    assert_error(&graph, "no output pin `output.base_color`");
    assert_error(&graph, "no input pin `base.value`");
}

#[test]
fn material_graph_validation_detects_cycles_in_disconnected_components() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "gain", 1.0);
    add_node(&mut graph, "first", Node::Add);
    add_node(&mut graph, "second", Node::Multiply);
    connect(&mut graph, "second", "value", "first", "a");
    connect(&mut graph, "gain", "value", "first", "b");
    connect(&mut graph, "first", "value", "second", "a");
    connect(&mut graph, "gain", "value", "second", "b");

    assert!(validate_material_graph(&graph)
        .iter()
        .any(|message| message.contains("contains a cycle")));
    assert_error(&graph, "contains a cycle");
}

#[test]
fn material_graph_rejects_incompatible_parameter_overrides() {
    let mut graph = base_graph();
    graph
        .parameters
        .insert("base".into(), Parameter::Scalar(0.5));
    assert_error(&graph, "parameter `base` has an incompatible override type");
    graph.parameters.clear();
    add_scalar(&mut graph, "roughness", 0.5);
    graph
        .parameters
        .insert("roughness".into(), Parameter::Vector([0.5; 4]));
    assert_error(
        &graph,
        "parameter `roughness` has an incompatible override type",
    );
}

#[test]
fn material_graph_rejects_non_finite_defaults_and_overrides() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "roughness", f32::NAN);
    assert_error(&graph, "parameter `roughness` has a non-finite default");
    graph.nodes.pop();
    graph
        .parameters
        .insert("base".into(), Parameter::Vector([f32::INFINITY; 4]));
    assert_error(&graph, "parameter `base` has a non-finite override");
}

#[test]
fn material_graph_reports_vector_arithmetic_overflow_before_publishing_material() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "large", f32::MAX);
    add_scalar(&mut graph, "two", 2.0);
    add_node(&mut graph, "product", Node::Multiply);
    connect(&mut graph, "large", "value", "product", "a");
    connect(&mut graph, "two", "value", "product", "b");
    graph.links[0].from_node = "product".into();
    assert_error(&graph, "produced a non-finite value");
}

#[test]
fn material_graph_reports_scalar_arithmetic_overflow_before_publishing_material() {
    let mut graph = base_graph();
    add_scalar(&mut graph, "large", f32::MAX);
    add_scalar(&mut graph, "two", 2.0);
    add_node(&mut graph, "product", Node::Multiply);
    connect(&mut graph, "large", "value", "product", "a");
    connect(&mut graph, "two", "value", "product", "b");
    connect(&mut graph, "product", "value", "output", "roughness");
    assert_error(&graph, "produced a non-finite scalar");
}

#[test]
fn material_graph_reports_unrepresentable_normal_expressions() {
    let mut graph = base_graph();
    connect(&mut graph, "base", "value", "output", "normal");
    assert_error(&graph, "`normal` requires a texture sample in v1");
}

#[test]
fn material_graph_rejects_vector_and_texture_inputs_for_scalar_outputs() {
    let mut graph = base_graph();
    connect(&mut graph, "base", "value", "output", "roughness");
    assert_error(&graph, "does not produce a scalar");
    graph.links.pop();
    add_node(
        &mut graph,
        "texture",
        Node::TextureSample {
            texture: reference("res://textures/a.png"),
        },
    );
    connect(&mut graph, "texture", "color", "output", "metallic");
    assert_error(&graph, "does not produce a scalar");
}

fn assert_error(graph: &MaterialGraphAsset, expected: &str) {
    let report = compile_material_graph_operation(graph);
    assert!(
        report.material.is_none(),
        "invalid graph must not publish a material"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|message| message.contains(expected)),
        "expected {expected:?}, got {:?}",
        report.diagnostics
    );
}

fn base_graph() -> MaterialGraphAsset {
    let mut graph = MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/pbr.material_graph.toml").unwrap(),
        name: "岩石 / PBR".into(),
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
    connect(&mut graph, "base", "value", "output", "base_color");
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

fn connect(
    graph: &mut MaterialGraphAsset,
    source: &str,
    from_pin: &str,
    target: &str,
    to_pin: &str,
) {
    graph.links.push(MaterialGraphLinkAsset {
        from_node: source.into(),
        from_pin: from_pin.into(),
        to_node: target.into(),
        to_pin: to_pin.into(),
    });
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
