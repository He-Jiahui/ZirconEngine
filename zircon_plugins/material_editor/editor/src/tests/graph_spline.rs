use super::graph_presentation_tests::graph;
use super::{
    compile_material_graph, validate_material_graph_with_diagnostics,
    MaterialGraphDiagnosticTarget, MaterialGraphPresentation,
};
use zircon_editor::ui::graph::{GraphCanvasState, GraphPinDirection, GraphPoint, GraphPortRef};
use zircon_runtime::asset::MaterialGraphLinkAsset as Link;

#[test]
fn material_graph_presentation_spline_uses_ue_forward_policy_and_zero_tangent() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let canvas = GraphCanvasState::default();
    for (target, tension) in [
        (GraphPoint::new(12.0, 0.0), 12.0),
        (GraphPoint::new(100.0, -30.0), 130.0),
        (GraphPoint::new(2000.0, 3000.0), 2000.0),
        (GraphPoint::new(0.0, -1200.0), 1000.0),
        (GraphPoint::ZERO, 0.0),
    ] {
        let spline = presentation
            .edge_spline_view(0, &canvas, GraphPoint::ZERO, target)
            .unwrap();
        assert_eq!(spline.source, GraphPoint::ZERO);
        assert_eq!(spline.target, target);
        assert_eq!(spline.travel_tangent, GraphPoint::new(tension, 0.0));
        assert!(std::ptr::eq(spline.edge.source, &graph.links[0]));
        assert_eq!(spline.edge.source_index, 0);
        assert_eq!(
            spline.edge.view.from,
            GraphPortRef::output("base".into(), "value")
        );
        assert_eq!(spline.edge.view.to, GraphPortRef::input("tint".into(), "a"));
    }
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_spline_uses_backward_weights_without_reversing_pins() {
    let graph = graph();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    for (target, tension) in [
        (GraphPoint::new(-30.0, -12.0), 78.0),
        (GraphPoint::new(-800.0, 600.0), 700.0),
        (GraphPoint::new(-800.0, 12.0), 418.0),
    ] {
        let spline = presentation
            .edge_spline_view(0, &GraphCanvasState::default(), GraphPoint::ZERO, target)
            .unwrap();
        assert_eq!(spline.source, GraphPoint::ZERO);
        assert_eq!(spline.target, target);
        assert_eq!(spline.travel_tangent, GraphPoint::new(tension, 0.0));
        assert_eq!(spline.edge.view.from.direction, GraphPinDirection::Output);
        assert_eq!(spline.edge.view.to.direction, GraphPinDirection::Input);
    }
}

#[test]
fn material_graph_presentation_spline_uses_one_canvas_transform_before_clamping() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let mut canvas = GraphCanvasState::default();
    canvas.replace_selection(["base".into()]);
    canvas.pan_by(GraphPoint::new(104.0, -20.0));
    assert!(canvas.zoom_about_screen_point(GraphPoint::new(50.0, 60.0), 5.0));
    let canvas_before = canvas.clone();
    let from = GraphPoint::new(206.5, 144.0);
    let to = GraphPoint::new(313.5, 48.0);
    let spline = presentation.edge_spline_view(0, &canvas, from, to).unwrap();
    assert_eq!(spline.source, GraphPoint::new(440.75, 156.0));
    assert_eq!(spline.target, GraphPoint::new(601.25, 12.0));
    assert_eq!(spline.travel_tangent, GraphPoint::new(304.5, 0.0));
    assert_eq!(canvas.screen_to_graph(spline.source), from);
    assert_eq!(canvas.screen_to_graph(spline.target), to);
    assert_eq!(canvas, canvas_before);
    let mut zoomed = GraphCanvasState::default();
    assert!(zoomed.zoom_about_screen_point(GraphPoint::ZERO, 10.0));
    let clamped = presentation
        .edge_spline_view(0, &zoomed, GraphPoint::ZERO, GraphPoint::new(800.0, 600.0))
        .unwrap();
    assert_eq!(clamped.target, GraphPoint::new(1600.0, 1200.0));
    // Clamp the drawn delta, not the graph tangent multiplied a second time.
    assert_eq!(clamped.travel_tangent, GraphPoint::new(2000.0, 0.0));
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_spline_rejects_invalid_geometry_without_writing_view_or_source() {
    let graph = graph();
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let canvas = GraphCanvasState::default();
    let canvas_before = canvas.clone();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for component in 0..4 {
            let mut values = [0.0, 0.0, 100.0, 30.0];
            values[component] = invalid;
            let error = presentation
                .edge_spline_view(
                    0,
                    &canvas,
                    GraphPoint::new(values[0], values[1]),
                    GraphPoint::new(values[2], values[3]),
                )
                .unwrap_err();
            assert_eq!(error.target, MaterialGraphDiagnosticTarget::Graph);
        }
        let mut invalid_view = canvas.clone();
        invalid_view.pan_by(GraphPoint::new(invalid, 0.0));
        let pan_bits = invalid_view.pan().x.to_bits();
        assert!(presentation
            .edge_spline_view(
                0,
                &invalid_view,
                GraphPoint::ZERO,
                GraphPoint::new(100.0, 30.0)
            )
            .is_err());
        assert_eq!(invalid_view.pan().x.to_bits(), pan_bits);
        assert_eq!(invalid_view.zoom(), 1.0);
        assert!(invalid_view.selection().node_ids().is_empty());
    }
    let mut zoomed = canvas.clone();
    assert!(zoomed.zoom_about_screen_point(GraphPoint::ZERO, 30.0));
    assert!(presentation
        .edge_spline_view(0, &zoomed, GraphPoint::new(f32::MAX, 0.0), GraphPoint::ZERO)
        .is_err());
    assert!(presentation
        .edge_spline_view(
            0,
            &canvas,
            GraphPoint::new(-f32::MAX, 0.0),
            GraphPoint::new(f32::MAX, 30.0),
        )
        .is_err());
    assert_eq!(canvas, canvas_before);
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_spline_keeps_real_occurrences_and_repairable_type_errors() {
    let mut graph = graph();
    graph.links.push(graph.links[0].clone());
    graph
        .links
        .push(link("base", "value", "output", "roughness"));
    graph
        .links
        .push(link("base", "wrong-output", "output", "emissive"));
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let canvas = GraphCanvasState::default();
    let from = GraphPoint::new(206.5, 144.0);
    let to = GraphPoint::new(313.5, 48.0);
    for index in [0, graph.links.len() - 3, graph.links.len() - 2] {
        let spline = presentation
            .edge_spline_view(index, &canvas, from, to)
            .unwrap();
        assert_eq!(spline.edge.source_index, index);
        assert!(std::ptr::eq(spline.edge.source, &graph.links[index]));
    }
    assert!(presentation
        .edge_spline_view(graph.links.len() - 1, &canvas, from, to)
        .is_err());
    assert!(presentation
        .edge_spline_view(graph.links.len(), &canvas, from, to)
        .is_err());
    assert!(!validate_material_graph_with_diagnostics(&graph).is_empty());
    assert!(compile_material_graph(&graph).is_err());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_presentation_spline_matches_saved_penpot_forward_curve_policy() {
    let graph = graph();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    // Source39/43 pin deltas; expected Hermite-to-Bezier offset comes from the
    // actual UE defaults, not the legacy max(abs(dx)/2,48) shared route.
    for (from, to, expected_offset) in [
        (
            GraphPoint::new(451.5, 385.0),
            GraphPoint::new(824.5, 487.0),
            475.0 / 3.0,
        ),
        (
            GraphPoint::new(451.5, 385.0),
            GraphPoint::new(558.5, 289.0),
            203.0 / 3.0,
        ),
        (
            GraphPoint::new(451.5, 565.0),
            GraphPoint::new(558.5, 317.0),
            355.0 / 3.0,
        ),
    ] {
        let spline = presentation
            .edge_spline_view(0, &GraphCanvasState::default(), from, to)
            .unwrap();
        assert_eq!(spline.travel_tangent.x / 3.0, expected_offset);
        assert_eq!(spline.travel_tangent.y, 0.0);
        assert_eq!(spline.source, from);
        assert_eq!(spline.target, to);
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
