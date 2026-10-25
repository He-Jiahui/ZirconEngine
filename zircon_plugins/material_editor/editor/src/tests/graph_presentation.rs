use std::collections::BTreeMap;

use super::{
    compile_material_graph, validate_material_graph_with_diagnostics,
    MaterialGraphConnectionAction, MaterialGraphDiagnosticTarget, MaterialGraphPresentation,
    MaterialGraphValueType,
};
use zircon_editor::ui::graph::{
    GraphCanvasState, GraphNodePresentation, GraphPinDirection, GraphPoint, GraphPortRef,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};

#[test]
fn material_graph_presentation_consumes_all_real_kinds_and_compiler_types() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(std::ptr::eq(presentation.nodes(), graph.nodes.as_slice()));
    assert!(std::ptr::eq(presentation.links(), graph.links.as_slice()));
    assert!(presentation.type_diagnostic().is_none());
    let expected = [
        ("output", "M_岩石_真实投影", 5, 0, None),
        (
            "base",
            "岩石颜色_中文参数及一个很长的显示名称",
            0,
            1,
            Some(MaterialGraphValueType::Vector),
        ),
        ("gain", "Gain", 0, 1, Some(MaterialGraphValueType::Scalar)),
        (
            "tint",
            "Multiply",
            2,
            1,
            Some(MaterialGraphValueType::Vector),
        ),
        ("sum", "Add", 2, 1, Some(MaterialGraphValueType::Scalar)),
        (
            "normal",
            "Texture Sample",
            0,
            1,
            Some(MaterialGraphValueType::Texture),
        ),
    ];
    for (id, title, input_count, output_count, value_type) in expected {
        let view = presentation
            .node_view(id, GraphPoint::new(230.0, 140.0))
            .unwrap();
        assert_eq!(view.id, id);
        assert_eq!(view.display_name, title);
        assert_eq!(view.inputs.len(), input_count);
        assert_eq!(view.outputs.len(), output_count);
        assert_eq!(presentation.output_type(id).unwrap(), value_type);
        assert!(view.attachments.is_empty());
        assert!(view
            .inputs
            .iter()
            .all(|pin| pin.direction == GraphPinDirection::Input));
        assert!(view
            .outputs
            .iter()
            .all(|pin| pin.direction == GraphPinDirection::Output));
    }
    let tint = presentation
        .node_view("tint", GraphPoint::new(230.0, 140.0))
        .unwrap();
    assert_eq!(tint.inputs[0].name, "a");
    assert_eq!(tint.inputs[0].value_type, "vec4");
    assert_eq!(tint.inputs[1].name, "b");
    assert_eq!(tint.inputs[1].value_type, "float");
    assert_eq!(tint.outputs[0].name, "value");
    assert_eq!(tint.outputs[0].value_type, "vec4");
    let material = compile_material_graph(&graph).unwrap();
    assert_eq!(material.base_color, [0.05, 0.1, 0.2, 1.0]);
    assert_eq!(
        material.normal_texture,
        Some(reference("res://textures/normal.texture.toml"))
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_uses_existing_selection_and_one_logical_transform() {
    let mut graph = graph();
    graph.nodes[1].position = [-600.0, 80.0];
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let measured = GraphPoint::new(276.0, 164.0);
    let view = presentation.node_view("base", measured).unwrap();
    assert_eq!(view.bounds.origin, GraphPoint::new(-600.0, 80.0));
    assert_eq!(view.bounds.size, measured);
    let mut canvas = GraphCanvasState::<String>::default();
    canvas.replace_selection(["base".into()]);
    canvas.pan_by(GraphPoint::new(104.0, -20.0));
    assert!(canvas.zoom_about_screen_point(GraphPoint::new(50.0, 60.0), 5.0));
    let displayed = GraphNodePresentation::from_view(&view, canvas.selection());
    assert_eq!(displayed.node_id, "base");
    assert!(displayed.selected);
    assert_eq!(displayed.title, "岩石颜色_中文参数及一个很长的显示名称");
    let screen = canvas.graph_to_screen(view.bounds.origin);
    let back = canvas.screen_to_graph(screen);
    assert!((back.x - view.bounds.origin.x).abs() < 0.001);
    assert!((back.y - view.bounds.origin.y).abs() < 0.001);
    assert_eq!(view.bounds.size, measured);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_keeps_incomplete_math_visible_and_reports_real_readiness() {
    let mut graph = graph();
    graph
        .links
        .retain(|edge| edge.to_node != "sum" || edge.to_pin != "b");
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(presentation.type_diagnostic().is_none());
    let view = presentation
        .node_view("sum", GraphPoint::new(230.0, 140.0))
        .unwrap();
    assert!(view.inputs.iter().all(|pin| pin.required));
    assert_eq!(view.inputs[0].value_type, "float");
    assert_eq!(view.inputs[1].value_type, "undetermined");
    assert_eq!(view.outputs[0].value_type, "undetermined");
    assert!(validate_material_graph_with_diagnostics(&graph)
        .iter()
        .any(|diagnostic| {
            diagnostic.target
                == MaterialGraphDiagnosticTarget::Input {
                    node_id: "sum".into(),
                    pin_name: "b".into(),
                }
        }));
    assert!(compile_material_graph(&graph).is_err());
}

#[test]
fn material_graph_presentation_keeps_ill_typed_wire_selectable_for_repair() {
    let mut graph = graph();
    graph
        .links
        .push(link("base", "value", "output", "roughness"));
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert_eq!(
        presentation.type_diagnostic().unwrap().target,
        MaterialGraphDiagnosticTarget::Output {
            node_id: "base".into(),
            pin_name: "value".into()
        }
    );
    let edge = presentation.edge_view(graph.links.len() - 1).unwrap();
    assert!(std::ptr::eq(edge.source, graph.links.last().unwrap()));
    assert_eq!(edge.source_index, graph.links.len() - 1);
    assert_eq!(edge.view.from, GraphPortRef::output("base".into(), "value"));
    assert_eq!(
        edge.view.to,
        GraphPortRef::input("output".into(), "roughness")
    );
    assert!(presentation
        .node_view("output", GraphPoint::new(230.0, 220.0))
        .is_ok());
    // With a rejected type pass, dynamic math is conservatively undetermined.
    assert_eq!(
        presentation.output_type("tint").unwrap(),
        Some(MaterialGraphValueType::Undetermined)
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_ambiguous_input_keeps_each_occurrence_and_unknown_type() {
    for duplicate_source in ["gain", "base"] {
        for reversed in [false, true] {
            let mut graph = graph();
            graph
                .links
                .push(link(duplicate_source, "value", "tint", "a"));
            let last = graph.links.len() - 1;
            if reversed {
                graph.links.swap(0, last);
            }
            let before = graph.clone();
            let presentation = MaterialGraphPresentation::new(&graph).unwrap();
            assert_eq!(
                presentation.type_diagnostic().unwrap().target,
                MaterialGraphDiagnosticTarget::Input {
                    node_id: "tint".into(),
                    pin_name: "a".into(),
                }
            );
            let view = presentation
                .node_view("tint", GraphPoint::new(230.0, 140.0))
                .unwrap();
            assert_eq!(view.inputs[0].name, "a");
            assert_eq!(view.inputs[0].value_type, "undetermined");
            assert_eq!(view.inputs[1].value_type, "float");
            assert_eq!(view.outputs[0].value_type, "undetermined");

            // Even identical endpoints identify two original occurrences in
            // this source borrow; neither is hidden or silently deduplicated.
            let first = presentation.edge_view(0).unwrap();
            let second = presentation.edge_view(last).unwrap();
            assert_eq!(first.source_index, 0);
            assert_eq!(second.source_index, last);
            assert!(std::ptr::eq(first.source, &graph.links[0]));
            assert!(std::ptr::eq(second.source, &graph.links[last]));
            assert!(!std::ptr::eq(first.source, second.source));
            assert_eq!(first.view.to, GraphPortRef::input("tint".into(), "a"));
            assert_eq!(first.view.to, second.view.to);
            assert_eq!(presentation.links(), before.links.as_slice());
            assert!(compile_material_graph(&graph).is_err());
            assert_eq!(graph, before);
        }
    }
}

#[test]
fn material_graph_presentation_rejects_unmeasured_or_unrepresentable_bounds() {
    let graph = graph();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    for size in [
        GraphPoint::new(0.0, 100.0),
        GraphPoint::new(-1.0, 100.0),
        GraphPoint::new(100.0, 0.0),
        GraphPoint::new(100.0, -1.0),
        GraphPoint::new(f32::NAN, 100.0),
        GraphPoint::new(100.0, f32::NAN),
        GraphPoint::new(f32::INFINITY, 100.0),
        GraphPoint::new(100.0, f32::NEG_INFINITY),
    ] {
        assert_eq!(
            presentation.node_view("base", size).unwrap_err().target,
            MaterialGraphDiagnosticTarget::Node {
                node_id: "base".into()
            }
        );
    }
    for origin in [
        [f32::NAN, 0.0],
        [0.0, f32::INFINITY],
        [f32::MAX, 0.0],
        [0.0, f32::MAX],
    ] {
        let mut malformed = graph.clone();
        malformed.nodes[1].position = origin;
        let presentation = MaterialGraphPresentation::new(&malformed).unwrap();
        assert!(presentation
            .node_view("base", GraphPoint::new(230.0, 140.0))
            .is_err());
    }
    let mut overflowing = graph.clone();
    overflowing.nodes[1].position = [f32::MAX / 2.0, 0.0];
    let presentation = MaterialGraphPresentation::new(&overflowing).unwrap();
    assert!(presentation
        .node_view("base", GraphPoint::new(f32::MAX, 100.0))
        .is_err());
}

#[test]
fn material_graph_presentation_rejects_stale_and_ambiguous_source_identities() {
    let mut graph = graph();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(presentation
        .node_view("stale", GraphPoint::new(230.0, 140.0))
        .is_err());
    assert!(presentation.output_type("stale").is_err());
    assert!(presentation.edge_view(graph.links.len()).is_err());
    graph.nodes[1].id = graph.nodes[0].id.clone();
    assert!(MaterialGraphPresentation::new(&graph).is_err());
    graph.nodes[1].id = " \t".into();
    assert!(MaterialGraphPresentation::new(&graph).is_err());
}

#[test]
fn material_graph_presentation_diagnoses_missing_endpoints_without_fabricating_geometry() {
    let mut graph = graph();
    graph
        .links
        .push(link("absent", "value", "output", "emissive"));
    graph
        .links
        .push(link("base", "wrong-output", "output", "emissive"));
    graph
        .links
        .push(link("base", "value", "output", "wrong-input"));
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(presentation.type_diagnostic().is_some());
    for index in graph.links.len() - 3..graph.links.len() {
        assert!(presentation.edge_view(index).is_err());
    }
    let valid = presentation.edge_view(0).unwrap();
    assert!(std::ptr::eq(valid.source, &graph.links[0]));
    assert!(presentation
        .node_view("base", GraphPoint::new(230.0, 140.0))
        .is_ok());
    assert_eq!(presentation.links().len(), graph.links.len());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_roundtrip_preserves_identity_with_equal_captions() {
    let mut graph = graph();
    let mut another = graph.nodes[1].clone();
    another.id = "参数_另一稳定节点".into();
    another.position = [-1200.0, 340.0];
    graph.nodes.push(another);
    let source = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&source).unwrap();
    assert_eq!(reopened, graph);
    let presentation = MaterialGraphPresentation::new(&reopened).unwrap();
    let first = presentation
        .node_view("base", GraphPoint::new(300.0, 140.0))
        .unwrap();
    let second = presentation
        .node_view("参数_另一稳定节点", GraphPoint::new(300.0, 140.0))
        .unwrap();
    assert_eq!(first.display_name, second.display_name);
    assert_ne!(first.id, second.id);
    let mut canvas = GraphCanvasState::<String>::default();
    canvas.replace_selection([second.id.clone()]);
    assert!(!GraphNodePresentation::from_view(&first, canvas.selection()).selected);
    assert!(GraphNodePresentation::from_view(&second, canvas.selection()).selected);
    assert_eq!(
        presentation.node(&second.id).unwrap().position,
        [-1200.0, 340.0]
    );
}

#[test]
fn material_graph_presentation_ports_use_material_rules_and_exact_replacement() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let texture = GraphPortRef::output("normal".into(), "color");
    let base_color = GraphPortRef::input("output".into(), "base_color");
    let plan = presentation.connection_plan(&texture, &base_color).unwrap();
    assert_eq!(plan.source_type, MaterialGraphValueType::Texture);
    let MaterialGraphConnectionAction::Replace(old) = plan.action else {
        panic!("occupied base color must retain its original edge");
    };
    assert!(std::ptr::eq(old, &graph.links[2]));
    assert!(presentation
        .connection_plan(&texture, &GraphPortRef::input("tint".into(), "b"))
        .is_err());
    assert!(presentation.connection_plan(&base_color, &texture).is_err());
    assert!(presentation
        .connection_plan(
            &texture,
            &GraphPortRef::input("output".into(), "Base Color")
        )
        .is_err());
    let same = presentation
        .connection_plan(&GraphPortRef::output("tint".into(), "value"), &base_color)
        .unwrap();
    assert_eq!(same.action, MaterialGraphConnectionAction::Unchanged);
    let scalar = presentation
        .connection_plan(
            &GraphPortRef::output("gain".into(), "value"),
            &GraphPortRef::input("output".into(), "roughness"),
        )
        .unwrap();
    assert_eq!(scalar.action, MaterialGraphConnectionAction::Connect);
    assert_eq!(scalar.source_type, MaterialGraphValueType::Scalar);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_empty_source_is_visible_without_invented_nodes_or_edges() {
    let mut graph = graph();
    graph.nodes.clear();
    graph.links.clear();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(presentation.nodes().is_empty());
    assert!(presentation.links().is_empty());
    assert!(presentation.type_diagnostic().is_none());
    assert!(presentation
        .node_view("output", GraphPoint::new(230.0, 220.0))
        .is_err());
    assert!(presentation.edge_view(0).is_err());
    assert!(compile_material_graph(&graph).is_err());
    assert_eq!(graph, before);
}

pub(super) fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/projection.material_graph.toml").unwrap(),
        name: "M_岩石_真实投影".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: vec![
            node("output", [560.0, 80.0], Node::Output),
            node(
                "base",
                [0.0, 80.0],
                Node::VectorParameter {
                    name: "岩石颜色_中文参数及一个很长的显示名称".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            ),
            node(
                "gain",
                [0.0, 260.0],
                Node::ScalarParameter {
                    name: "Gain".into(),
                    default: 0.25,
                },
            ),
            node("tint", [300.0, 0.0], Node::Multiply),
            node("sum", [0.0, 440.0], Node::Add),
            node(
                "normal",
                [300.0, 440.0],
                Node::TextureSample {
                    texture: reference("res://textures/normal.texture.toml"),
                },
            ),
        ],
        links: vec![
            link("base", "value", "tint", "a"),
            link("gain", "value", "tint", "b"),
            link("tint", "value", "output", "base_color"),
            link("gain", "value", "sum", "a"),
            link("gain", "value", "sum", "b"),
            link("normal", "color", "output", "normal"),
        ],
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, position: [f32; 2], kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position,
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

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
