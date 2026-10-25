use std::collections::BTreeMap;

use super::{
    can_connect_material_graph, compile_material_graph, compile_material_graph_operation,
    compile_material_graph_with_diagnostics, validate_material_graph,
    validate_material_graph_with_diagnostics, MaterialGraphCompileReport, MaterialGraphDiagnostic,
    MaterialGraphDiagnosticTarget as Target,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Parameter,
};

#[test]
fn material_graph_missing_required_inputs_keep_actual_schema_keys() {
    let mut graph = graph();
    let id = "节点 / a.b`";
    add_node(&mut graph, id, Node::Add);
    let before = graph.clone();
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    for pin in ["a", "b"] {
        assert_eq!(
            diagnostic(
                &diagnostics,
                &format!("material graph node `{id}` is missing required input `{pin}`"),
            )
            .target,
            input(id, pin),
        );
    }
    assert_eq!(
        graph, before,
        "validation must preserve the authoring document"
    );
}

#[test]
fn material_graph_unknown_pin_targets_its_node_without_inventing_a_pin() {
    let mut graph = graph();
    graph.links[0].from_pin = "display label".into();
    graph.links[0].to_pin = "Base Color".into();
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph has no output pin `tint.display label`"
        )
        .target,
        node("tint"),
    );
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph has no input pin `output.Base Color`"
        )
        .target,
        node("output"),
    );
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph output `output` is missing required input `base_color`"
        )
        .target,
        input("output", "base_color"),
    );
}

#[test]
fn material_graph_ambiguous_and_empty_node_ids_never_become_jump_targets() {
    for id in ["tint", "   "] {
        let mut graph = graph();
        add_node(&mut graph, id, Node::Add);
        let diagnostics = validate_material_graph_with_diagnostics(&graph);
        for pin in ["a", "b"] {
            assert_eq!(
                diagnostic(
                    &diagnostics,
                    &format!("material graph node `{id}` is missing required input `{pin}`")
                )
                .target,
                Target::Graph,
            );
        }
        let identity_message = if id == "tint" {
            "material graph has duplicate node `tint`"
        } else {
            "material graph node id must not be empty"
        };
        assert_eq!(
            diagnostic(&diagnostics, identity_message).target,
            Target::Graph
        );
    }
}

#[test]
fn material_graph_missing_source_does_not_invent_a_node_but_keeps_real_input() {
    let mut graph = graph();
    graph.links[0].from_node = "deleted.node".into();
    let diagnostics = compile_material_graph_with_diagnostics(&graph).unwrap_err();
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph link references missing source node `deleted.node`"
        )
        .target,
        Target::Graph,
    );
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph output `output` base_color input is disconnected"
        )
        .target,
        input("output", "base_color"),
    );
}

#[test]
fn material_graph_distinct_default_locations_preserve_legacy_dedup_and_shader_order() {
    let mut graph = graph();
    for id in ["bad_a", "bad_b"] {
        add_node(
            &mut graph,
            id,
            Node::ScalarParameter {
                name: "shared".into(),
                default: f32::INFINITY,
            },
        );
    }
    graph.shader = None;
    let message = "material graph parameter `shared` has a non-finite default";
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0],
        MaterialGraphDiagnostic {
            message: message.into(),
            target: node("bad_a")
        }
    );
    assert_eq!(
        diagnostics[1],
        MaterialGraphDiagnostic {
            message: message.into(),
            target: node("bad_b")
        }
    );
    assert_eq!(validate_material_graph(&graph), vec![message]);

    let shader_message =
        "material graph `Diagnostic regression` has no shader target for MaterialAsset compile";
    let compiled = compile_material_graph_with_diagnostics(&graph).unwrap_err();
    assert_eq!(&compiled[..2], &diagnostics);
    assert_eq!(
        compiled[2],
        MaterialGraphDiagnostic {
            message: shader_message.into(),
            target: Target::Graph
        }
    );
    assert_eq!(
        compile_material_graph(&graph).unwrap_err(),
        vec![message, shader_message]
    );
}

#[test]
fn material_graph_parameter_override_locations_use_map_keys() {
    let mut graph = graph();
    let key = "unused.参数 / `";
    graph
        .parameters
        .insert(key.into(), Parameter::Scalar(f32::INFINITY));
    graph
        .parameters
        .insert("tint".into(), Parameter::Scalar(0.5));
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert_eq!(
        diagnostic(
            &diagnostics,
            &format!("material graph parameter `{key}` has a non-finite override")
        )
        .target,
        Target::Parameter { name: key.into() },
    );
    assert_eq!(
        diagnostic(
            &diagnostics,
            "material graph parameter `tint` has an incompatible override type"
        )
        .target,
        Target::Parameter {
            name: "tint".into()
        },
    );
}

#[test]
fn material_graph_scalar_type_failure_points_to_real_source_output() {
    let mut graph = graph();
    let candidate = link("tint", "value", "output", "metallic");
    let message = "node `tint` does not produce a scalar";
    assert_eq!(
        can_connect_material_graph(&graph, &candidate).unwrap_err(),
        message
    );
    graph.links.push(candidate);
    let diagnostics = compile_material_graph_with_diagnostics(&graph).unwrap_err();
    assert_eq!(
        diagnostics,
        vec![MaterialGraphDiagnostic {
            message: message.into(),
            target: Target::Output {
                node_id: "tint".into(),
                pin_name: "value".into()
            },
        }]
    );
    assert_eq!(compile_material_graph(&graph).unwrap_err(), vec![message]);
}

#[test]
fn material_graph_texture_math_failure_points_to_the_actual_offending_input() {
    for pin in ["a", "b"] {
        let mut graph = graph();
        add_node(
            &mut graph,
            "texture",
            Node::TextureSample {
                texture: reference("res://textures/tint.png"),
            },
        );
        add_node(&mut graph, "math", Node::Multiply);
        graph.links.push(link("texture", "color", "math", pin));
        graph.links.push(link(
            "tint",
            "value",
            "math",
            if pin == "a" { "b" } else { "a" },
        ));
        let diagnostics = compile_material_graph_with_diagnostics(&graph).unwrap_err();
        assert_eq!(
            diagnostics,
            vec![MaterialGraphDiagnostic {
                message:
                    "material graph `multiply` node cannot combine texture-backed inputs in v1"
                        .into(),
                target: input("math", pin),
            }]
        );
    }
}

#[test]
fn material_graph_normal_type_failure_keeps_the_real_normal_input() {
    let mut graph = graph();
    graph.links.push(link("tint", "value", "output", "normal"));
    assert_eq!(
        compile_material_graph_with_diagnostics(&graph).unwrap_err(),
        vec![MaterialGraphDiagnostic {
            message: "material graph `normal` requires a texture sample in v1".into(),
            target: input("output", "normal"),
        }]
    );
}

#[test]
fn material_graph_color_overflow_retains_source_even_when_message_has_no_node_id() {
    let mut graph = overflow_graph();
    let id = "节点 / overflow.b`";
    graph.links[0] = link(id, "value", "output", "base_color");
    let before = graph.clone();
    let message = "material graph `multiply` produced a non-finite value";
    assert!(validate_material_graph(&graph).is_empty());
    assert_eq!(
        compile_material_graph_with_diagnostics(&graph).unwrap_err(),
        vec![MaterialGraphDiagnostic {
            message: message.into(),
            target: node(id),
        }]
    );
    assert_eq!(compile_material_graph(&graph).unwrap_err(), vec![message]);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_scalar_output_context_preserves_upstream_overflow_target() {
    let mut graph = overflow_graph();
    let id = "节点 / overflow.b`";
    graph.links.push(link(id, "value", "output", "metallic"));
    let message = format!(
        "material graph output `output.metallic`: node `{id}` produced a non-finite scalar"
    );
    assert_eq!(
        compile_material_graph_with_diagnostics(&graph).unwrap_err(),
        vec![MaterialGraphDiagnostic {
            message: message.clone(),
            target: node(id),
        }]
    );
    assert_eq!(compile_material_graph(&graph).unwrap_err(), vec![message]);
}

#[test]
fn material_graph_cycle_remains_graph_level_without_guessing_a_failing_node() {
    let mut graph = graph();
    add_node(&mut graph, "math", Node::Add);
    graph.links.push(link("math", "value", "math", "a"));
    graph.links.push(link("tint", "value", "math", "b"));
    assert_eq!(
        compile_material_graph_with_diagnostics(&graph).unwrap_err(),
        vec![MaterialGraphDiagnostic {
            message: "material graph contains a cycle".into(),
            target: Target::Graph,
        }]
    );
}

#[test]
fn material_graph_located_compile_preserves_material_and_old_report_struct_literal() {
    let graph = graph();
    let before = graph.clone();
    let material = compile_material_graph_with_diagnostics(&graph).unwrap();
    assert_eq!(compile_material_graph(&graph).unwrap(), material);
    assert_eq!(
        compile_material_graph_operation(&graph),
        MaterialGraphCompileReport {
            diagnostics: Vec::new(),
            material: Some(material),
        }
    );
    assert!(validate_material_graph_with_diagnostics(&graph).is_empty());
    assert_eq!(graph, before);
}

fn diagnostic<'a>(
    diagnostics: &'a [MaterialGraphDiagnostic],
    message: &str,
) -> &'a MaterialGraphDiagnostic {
    diagnostics
        .iter()
        .find(|entry| entry.message == message)
        .expect("actual validation diagnostic")
}

fn node(node_id: &str) -> Target {
    Target::Node {
        node_id: node_id.into(),
    }
}

fn input(node_id: &str, pin_name: &str) -> Target {
    Target::Input {
        node_id: node_id.into(),
        pin_name: pin_name.into(),
    }
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/diagnostics.material_graph.toml").unwrap(),
        name: "Diagnostic regression".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: vec![
            MaterialGraphNodeAsset {
                id: "tint".into(),
                position: [0.0; 2],
                kind: Node::VectorParameter {
                    name: "tint".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            },
            MaterialGraphNodeAsset {
                id: "output".into(),
                position: [0.0; 2],
                kind: Node::Output,
            },
        ],
        links: vec![link("tint", "value", "output", "base_color")],
        parameters: BTreeMap::new(),
    }
}

fn overflow_graph() -> MaterialGraphAsset {
    let mut graph = graph();
    add_node(
        &mut graph,
        "large",
        Node::ScalarParameter {
            name: "large".into(),
            default: f32::MAX,
        },
    );
    add_node(
        &mut graph,
        "two",
        Node::ScalarParameter {
            name: "two".into(),
            default: 2.0,
        },
    );
    add_node(&mut graph, "节点 / overflow.b`", Node::Multiply);
    graph
        .links
        .push(link("large", "value", "节点 / overflow.b`", "a"));
    graph
        .links
        .push(link("two", "value", "节点 / overflow.b`", "b"));
    graph
}

fn add_node(graph: &mut MaterialGraphAsset, id: &str, kind: Node) {
    graph.nodes.push(MaterialGraphNodeAsset {
        id: id.into(),
        position: [0.0; 2],
        kind,
    });
}

fn link(source: &str, from_pin: &str, target: &str, to_pin: &str) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: source.into(),
        from_pin: from_pin.into(),
        to_node: target.into(),
        to_pin: to_pin.into(),
    }
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
