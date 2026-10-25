use super::graph_presentation_tests::graph;
use super::{
    compile_material_graph, MaterialGraphDiagnostic, MaterialGraphDiagnosticTarget,
    MaterialGraphPresentation,
};
use zircon_editor::ui::graph::{GraphCanvasState, GraphNodeBounds, GraphPoint};

#[test]
fn material_graph_bounds_measures_every_original_node_without_changing_author_data() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let mut measured_ids = Vec::new();
    let bounds = presentation
        .graph_bounds(|node| {
            assert!(graph.nodes.iter().any(|source| std::ptr::eq(source, node)));
            measured_ids.push(node.id.clone());
            Ok(if node.id == "output" {
                GraphPoint::new(230.0, 220.0)
            } else if node.id == "base" {
                GraphPoint::new(300.0, 140.0)
            } else {
                GraphPoint::new(230.0, 140.0)
            })
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        measured_ids,
        graph
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bounds,
        GraphNodeBounds::new(GraphPoint::ZERO, GraphPoint::new(790.0, 580.0))
    );
    assert_eq!(graph, before);
    assert!(compile_material_graph(&graph).is_ok());
}

#[test]
fn material_graph_bounds_empty_source_has_no_fabricated_fit_or_measurement() {
    let mut graph = graph();
    graph.nodes.clear();
    graph.links.clear();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert_eq!(
        presentation
            .graph_bounds(|_| panic!("empty graph has no size to measure"))
            .unwrap(),
        None
    );
    assert!(compile_material_graph(&graph).is_err());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_bounds_missing_retained_measurement_returns_original_error() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let error = MaterialGraphDiagnostic {
        message: "retained geometry generation no longer contains this node".into(),
        target: MaterialGraphDiagnosticTarget::Node {
            node_id: "base".into(),
        },
    };
    let mut measured_ids = Vec::new();
    let result = presentation.graph_bounds(|node| {
        measured_ids.push(node.id.as_str());
        if node.id == "base" {
            Err(error.clone())
        } else {
            Ok(GraphPoint::new(230.0, 140.0))
        }
    });
    assert_eq!(result.unwrap_err(), error);
    assert_eq!(measured_ids, ["output", "base"]);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_bounds_invalid_measurements_are_not_silently_skipped() {
    let graph = graph();
    let before = graph.clone();
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
        let error = presentation
            .graph_bounds(|node| {
                Ok(if node.id == "base" {
                    size
                } else {
                    GraphPoint::new(230.0, 140.0)
                })
            })
            .unwrap_err();
        assert_eq!(
            error.target,
            MaterialGraphDiagnosticTarget::Node {
                node_id: "base".into()
            }
        );
        assert_eq!(error, presentation.node_view("base", size).unwrap_err());
    }
    assert_eq!(graph, before);
}

#[test]
fn material_graph_bounds_rejects_invalid_positions_and_unrepresentable_union() {
    for position in [
        [f32::NAN, 0.0],
        [0.0, f32::INFINITY],
        [f32::MAX, 0.0],
        [0.0, f32::MAX],
    ] {
        let mut graph = graph();
        graph.nodes[1].position = position;
        let presentation = MaterialGraphPresentation::new(&graph).unwrap();
        assert_eq!(
            presentation
                .graph_bounds(|_| Ok(GraphPoint::new(230.0, 140.0)))
                .unwrap_err()
                .target,
            MaterialGraphDiagnosticTarget::Node {
                node_id: "base".into()
            }
        );
    }
    let mut overflowing = graph();
    overflowing.nodes[1].position = [f32::MAX / 2.0, 0.0];
    let presentation = MaterialGraphPresentation::new(&overflowing).unwrap();
    assert_eq!(
        presentation
            .graph_bounds(|node| Ok(if node.id == "base" {
                GraphPoint::new(f32::MAX, 140.0)
            } else {
                GraphPoint::new(230.0, 140.0)
            }))
            .unwrap_err()
            .target,
        MaterialGraphDiagnosticTarget::Node {
            node_id: "base".into()
        }
    );
    let mut graph = graph();
    graph.nodes.truncate(2);
    graph.links.clear();
    graph.nodes[0].position = [-f32::MAX * 0.75, 0.0];
    graph.nodes[1].position = [f32::MAX * 0.75, 0.0];
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let measured = GraphPoint::new(f32::MAX * 0.125, 100.0);
    assert!(presentation.node_view("output", measured).is_ok());
    assert!(presentation.node_view("base", measured).is_ok());
    assert_eq!(
        presentation
            .graph_bounds(|_| Ok(measured))
            .unwrap_err()
            .target,
        MaterialGraphDiagnosticTarget::Graph
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_bounds_rounds_union_outward_instead_of_omitting_far_node() {
    let mut graph = graph();
    graph.nodes.truncate(2);
    graph.links.clear();
    graph.nodes[0].position = [-1.0, 0.0];
    graph.nodes[1].position = [16_777_214.0, 0.0];
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let bounds = presentation
        .graph_bounds(|node| {
            Ok(GraphPoint::new(
                if node.id == "base" { 2.0 } else { 1.0 },
                10.0,
            ))
        })
        .unwrap()
        .unwrap();
    assert_eq!(bounds.origin, GraphPoint::new(-1.0, 0.0));
    assert!(f64::from(bounds.size.x) >= 16_777_217.0);
    assert!(bounds.right() >= 16_777_216.0);
    let mut canvas = GraphCanvasState::<String>::default();
    let fit = canvas
        .fit_graph_bounds(bounds, GraphPoint::new(447.0, 400.0), 10.0)
        .unwrap();
    assert!(!fit.fully_visible);
    assert_eq!(canvas.zoom());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_bounds_frames_real_positions_with_existing_canvas_and_one_transform() {
    let mut graph = graph();
    for node in &mut graph.nodes {
        node.position[0] -= 600.0;
        node.position[1] += 80.0;
    }
    // Framing remains available when authoring data is incomplete, without
    // treating the missing compiler input as successfully repaired.
    graph
        .links
        .retain(|link| link.to_node != "sum" || link.to_pin != "b");
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let bounds = presentation
        .graph_bounds(|_| Ok(GraphPoint::new(270.0, 140.0)))
        .unwrap()
        .unwrap();
    for width in [895.0, 629.0, 447.0] {
        let mut canvas = GraphCanvasState::<String>::default();
        canvas.replace_selection(["base".into()]);
        canvas.pan_by(GraphPoint::new(104.0, -20.0));
        let selection_before = canvas.selection().clone();
        let viewport = GraphPoint::new(width, 322.0);
        let fit = canvas.fit_graph_bounds(bounds, viewport, 16.0).unwrap();
        assert!(fit.fully_visible);
        assert_eq!(canvas.selection(), &selection_before);
        for node in &graph.nodes {
            let view = presentation
                .node_view(&node.id, GraphPoint::new(270.0, 140.0))
                .unwrap();
            let first = canvas.graph_to_screen(view.bounds.origin);
            let last =
                canvas.graph_to_screen(GraphPoint::new(view.bounds.right(), view.bounds.bottom()));
            assert!(
                first.x >= 16.0 && first.y >= 16.0 && last.x <= width - 16.0 && last.y <= 306.0
            );
            let inverse = canvas.screen_to_graph(first);
            assert!((inverse.x - view.bounds.origin.x).abs() < 0.001);
            assert!((inverse.y - view.bounds.origin.y).abs() < 0.001);
        }
    }
    assert!(compile_material_graph(&graph).is_err());
    assert_eq!(graph, before);
}
