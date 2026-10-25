use std::collections::BTreeMap;

use super::{
    can_connect_material_graph, compile_material_graph, search_material_graph_node_palette,
    validate_material_graph_node_creation, MaterialGraphDiagnostic, MaterialGraphDiagnosticTarget,
    MaterialGraphNodeSearch,
};
use zircon_editor::core::editor_authoring_extension::GraphNodeDescriptor;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Parameter,
};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn material_graph_creation_accepts_registered_runtime_node_kinds() {
    let graph = graph();
    let kinds = [
        ("output", Node::Output),
        (
            "texture_sample",
            Node::TextureSample {
                texture: reference("res://textures/rock.png"),
            },
        ),
        ("scalar_parameter", scalar("Roughness", 0.5)),
        (
            "vector_parameter",
            Node::VectorParameter {
                name: "Base Color".into(),
                default: [0.1, 0.2, 0.3, 1.0],
            },
        ),
        ("add", Node::Add),
        ("multiply", Node::Multiply),
    ];
    let mut resolutions = 0;
    for (key, kind) in kinds {
        let node = node(key, kind);
        assert!(
            validate_material_graph_node_creation(&graph, &action(key), &node, |_| {
                resolutions += 1;
                Ok(ResourceKind::Texture)
            })
            .is_empty()
        );
    }
    assert_eq!(resolutions, 1);
    assert!(graph.nodes.is_empty());
}

#[test]
fn material_graph_creation_rejects_stale_actions_and_kind_mismatch() {
    let graph = graph();
    let node = node("new", Node::Add);
    for descriptor in [
        GraphNodeDescriptor::new("function", "Function", "Material"),
        GraphNodeDescriptor::new("add", "Renamed outside the registry", "Math"),
        action("multiply"),
    ] {
        let errors = validate_material_graph_node_creation(&graph, &descriptor, &node, |_| {
            panic!("invalid action must not resolve assets")
        });
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
    }
}

#[test]
fn material_graph_creation_rejects_empty_duplicate_and_reserved_identities() {
    let mut graph = graph();
    graph.nodes.push(node("existing", Node::Add));
    graph.links.push(MaterialGraphLinkAsset {
        from_node: "reserved".into(),
        from_pin: "value".into(),
        to_node: "existing".into(),
        to_pin: "a".into(),
    });
    let before = graph.clone();
    for id in ["", "  \u{3000}", "existing", "reserved"] {
        assert!(!validate(&graph, &node(id, Node::Add), "add").is_empty());
    }
    assert_eq!(graph, before);
    graph.nodes.push(node("existing", Node::Multiply));
    assert!(!validate(&graph, &node("new", Node::Add), "add").is_empty());
}

#[test]
fn material_graph_creation_preserves_finite_graph_local_positions() {
    let graph = graph();
    let mut node = node("exact_source_id", Node::Add);
    node.position = [-123.25, 4096.5];
    assert!(validate(&graph, &node, "add").is_empty());
    assert_eq!(node.position, [-123.25, 4096.5]);
    assert_eq!(node.id, "exact_source_id");
    for position in [
        [f32::NAN, 0.0],
        [0.0, f32::INFINITY],
        [f32::NEG_INFINITY, 0.0],
    ] {
        node.position = position;
        assert!(!validate(&graph, &node, "add").is_empty());
    }
}

#[test]
fn material_graph_creation_output_is_unique_but_does_not_require_compile_readiness() {
    let mut graph = graph();
    assert!(validate(&graph, &node("output", Node::Output), "output").is_empty());
    graph.nodes.push(node("output", Node::Output));
    assert!(!validate(&graph, &node("second", Node::Output), "output").is_empty());
    // An unconnected output or math input remains editable.
    assert!(validate(&graph, &node("math", Node::Multiply), "multiply").is_empty());
    assert!(graph.links.is_empty());
}

#[test]
fn material_graph_creation_keeps_parameter_names_and_rejects_invalid_defaults() {
    let graph = graph();
    let name = "岩石粗糙度 / exact authored name with spaces";
    let node = node("材质参数_1", scalar(name, -0.25));
    assert!(validate(&graph, &node, "scalar_parameter").is_empty());
    assert_eq!(node.kind, scalar(name, -0.25));
    for kind in [
        scalar(" \u{3000}", 0.5),
        scalar(name, f32::NAN),
        Node::VectorParameter {
            name: name.into(),
            default: [0.0, 1.0, f32::INFINITY, 1.0],
        },
    ] {
        let key = if matches!(kind, Node::VectorParameter { .. }) {
            "vector_parameter"
        } else {
            "scalar_parameter"
        };
        assert!(!validate(&graph, &node_creation_candidate(kind), key).is_empty());
    }
}

#[test]
fn material_graph_creation_parameter_name_reuse_requires_the_same_type() {
    let mut graph = graph();
    graph.nodes.push(node("original", scalar("shared", 0.1)));
    assert!(validate(
        &graph,
        &node("another", scalar("shared", 0.9)),
        "scalar_parameter"
    )
    .is_empty());
    let candidate = node(
        "vector",
        Node::VectorParameter {
            name: "shared".into(),
            default: [1.0; 4],
        },
    );
    let errors = validate(&graph, &candidate, "vector_parameter");
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].target,
        MaterialGraphDiagnosticTarget::Node {
            node_id: "original".into()
        }
    );
    assert!(graph.parameters.is_empty());
}

#[test]
fn material_graph_creation_override_checks_preserve_the_existing_parameter_map() {
    let mut graph = graph();
    graph
        .parameters
        .insert("roughness".into(), Parameter::Scalar(0.7));
    let candidate = node("new", scalar("roughness", 0.25));
    let before = graph.clone();
    assert!(validate(&graph, &candidate, "scalar_parameter").is_empty());
    assert_eq!(graph, before);
    for value in [
        Parameter::Vector([0.0; 4]),
        Parameter::Scalar(f32::INFINITY),
    ] {
        graph.parameters.insert("roughness".into(), value);
        let errors = validate(&graph, &candidate, "scalar_parameter");
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].target,
            MaterialGraphDiagnosticTarget::Parameter {
                name: "roughness".into()
            }
        );
    }
}

#[test]
fn material_graph_creation_texture_uses_the_complete_reference_and_actual_resolver() {
    let graph = graph();
    let reference = reference("res://textures/registered_without_an_extension");
    let candidate = node(
        "texture",
        Node::TextureSample {
            texture: reference.clone(),
        },
    );
    let before = candidate.clone();
    let mut calls = 0;
    let errors = validate_material_graph_node_creation(
        &graph,
        &action("texture_sample"),
        &candidate,
        |actual| {
            calls += 1;
            assert_eq!(actual, &reference);
            Ok(ResourceKind::Texture)
        },
    );
    assert!(errors.is_empty());
    assert_eq!(calls, 1);
    assert_eq!(candidate, before);
}

#[test]
fn material_graph_creation_texture_resolution_failure_does_not_mutate_the_graph() {
    let graph = graph();
    let before = graph.clone();
    let mut candidate = node(
        "texture",
        Node::TextureSample {
            texture: reference("res://materials/wrong_kind.png"),
        },
    );
    for result in [
        Ok(ResourceKind::Material),
        Err("asset UUID/locator does not resolve in the active catalog".into()),
    ] {
        let errors = validate_material_graph_node_creation(
            &graph,
            &action("texture_sample"),
            &candidate,
            |_| result.clone(),
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert_eq!(graph, before);
    }
    candidate.position = [f32::NAN, 0.0];
    assert!(!validate_material_graph_node_creation(
        &graph,
        &action("texture_sample"),
        &candidate,
        |_| panic!("invalid geometry must not query the catalog"),
    )
    .is_empty());
}

#[test]
fn material_graph_creation_search_candidate_roundtrips_and_reaches_the_pbr_compiler() {
    let mut graph = graph();
    graph.shader = Some(reference("res://shaders/pbr.wgsl"));
    graph.nodes.push(node(
        "base",
        Node::VectorParameter {
            name: "Base Color".into(),
            default: [0.2, 0.4, 0.8, 1.0],
        },
    ));
    graph.nodes.push(node("output", Node::Output));
    graph.links.push(MaterialGraphLinkAsset {
        from_node: "base".into(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: "base_color".into(),
    });
    let before = graph.clone();
    let mut search = MaterialGraphNodeSearch::new("scalar");
    let descriptor = search.activate().expect("real registered search action");
    let mut candidate = node("roughness_1", scalar("岩石粗糙度", 0.35));
    candidate.position = [-32.0, 128.5];
    assert!(
        validate_material_graph_node_creation(&graph, &descriptor, &candidate, |_| {
            panic!("scalar creation must not resolve resources")
        })
        .is_empty()
    );
    assert_eq!(graph, before);

    // Build the runtime DTO for codec/compiler compatibility only. This is not
    // a host edit, a history transaction or proof of a renderer frame.
    graph.nodes.push(candidate.clone());
    let link = MaterialGraphLinkAsset {
        from_node: candidate.id.clone(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: "roughness".into(),
    };
    assert!(can_connect_material_graph(&graph, &link).is_ok());
    graph.links.push(link);
    let saved = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    assert_eq!(reopened, graph);
    assert_eq!(reopened.nodes.last(), Some(&candidate));
    let material = compile_material_graph(&reopened).expect("real supported PBR graph");
    assert_eq!(material.standard_material_descriptor().roughness, 0.35);
    assert!(!search.is_open());
    assert!(search.activate().is_none());
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/node_creation.material_graph.toml").unwrap(),
        name: "Node creation / 节点创建".into(),
        shader: None,
        nodes: Vec::new(),
        links: Vec::new(),
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

fn node_creation_candidate(kind: Node) -> MaterialGraphNodeAsset {
    node("new", kind)
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
        .find(|descriptor| descriptor.id() == id)
        .unwrap()
}

fn validate(
    graph: &MaterialGraphAsset,
    candidate: &MaterialGraphNodeAsset,
    action_id: &str,
) -> Vec<MaterialGraphDiagnostic> {
    validate_material_graph_node_creation(graph, &action(action_id), candidate, |_| {
        panic!("non-texture creation must not resolve resources")
    })
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
