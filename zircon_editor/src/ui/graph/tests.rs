use super::{
    GraphAlignment, GraphCanvasState, GraphDeltaCommand, GraphEdgeView, GraphEditContext,
    GraphMutationEffect, GraphNodeBounds, GraphNodeView, GraphPinDirection, GraphPinView,
    GraphPoint, GraphPortRef, GraphRouteStyle, StructureConstraint, aligned_node_moves,
    default_connection_verdict, required_input_diagnostics, route_connection,
};
use crate::core::editing::engine::{EditCommand, EditCommandError, EditContext, SelectionSnapshot};
use crate::core::gateway::EditorRuntimeGatewayHandle;
use std::any::Any;
use std::convert::Infallible;

fn node(
    id: &str,
    position: GraphPoint,
    inputs: &[(&str, &str, bool)],
    outputs: &[(&str, &str)],
) -> GraphNodeView<String> {
    GraphNodeView {
        id: id.to_string(),
        display_name: id.to_string(),
        bounds: GraphNodeBounds::new(position, GraphPoint::new(120.0, 64.0)),
        inputs: inputs
            .iter()
            .map(|(name, value_type, required)| GraphPinView::input(*name, *value_type, *required))
            .collect(),
        outputs: outputs
            .iter()
            .map(|(name, value_type)| GraphPinView::output(*name, *value_type))
            .collect(),
        attachments: Vec::new(),
    }
}

fn edge(from_node: &str, from_pin: &str, to_node: &str, to_pin: &str) -> GraphEdgeView<String> {
    GraphEdgeView::new(
        GraphPortRef::output(from_node.to_string(), from_pin),
        GraphPortRef::input(to_node.to_string(), to_pin),
    )
}

#[test]
fn default_schema_validates_direction_type_and_dag_cycles() {
    let nodes = vec![
        node(
            "left",
            GraphPoint::ZERO,
            &[("in", "animation.pose", false)],
            &[("out", "animation.pose")],
        ),
        node(
            "right",
            GraphPoint::new(240.0, 0.0),
            &[
                ("in", "animation.pose", true),
                ("mismatch", "animation.scalar", false),
            ],
            &[("out", "animation.pose")],
        ),
    ];
    let connection = edge("left", "out", "right", "in");

    assert!(
        default_connection_verdict(&nodes, &[], &connection, StructureConstraint::Dag,)
            .is_allowed()
    );

    let backwards = edge("right", "out", "left", "in");
    assert!(
        default_connection_verdict(
            &nodes,
            &[connection.clone()],
            &backwards,
            StructureConstraint::Dag,
        )
        .is_rejected()
    );

    let mismatched = edge("left", "out", "right", "mismatch");
    assert!(
        default_connection_verdict(&nodes, &[], &mismatched, StructureConstraint::FreeGraph,)
            .is_rejected()
    );
}

#[test]
fn tree_schema_rejects_a_second_parent_for_the_same_input() {
    let nodes = vec![
        node("left", GraphPoint::ZERO, &[], &[("out", "flow")]),
        node(
            "right",
            GraphPoint::new(0.0, 120.0),
            &[],
            &[("out", "flow")],
        ),
        node(
            "child",
            GraphPoint::new(240.0, 0.0),
            &[("in", "flow", true)],
            &[],
        ),
    ];
    let first = edge("left", "out", "child", "in");
    let second = edge("right", "out", "child", "in");

    assert!(
        default_connection_verdict(&nodes, &[first], &second, StructureConstraint::Tree)
            .is_rejected()
    );
}

#[test]
fn canvas_zoom_preserves_the_graph_point_under_the_pointer_and_marquee_selects_intersections() {
    let nodes = vec![
        node("first", GraphPoint::new(10.0, 10.0), &[], &[]),
        node("second", GraphPoint::new(220.0, 10.0), &[], &[]),
    ];
    let mut canvas = GraphCanvasState::default();
    let pointer = GraphPoint::new(160.0, 120.0);
    let before = canvas.screen_to_graph(pointer);

    canvas.zoom_about_screen_point(pointer, 2.0);

    let after = canvas.screen_to_graph(pointer);
    assert!((after.x - before.x).abs() < 0.001);
    assert!((after.y - before.y).abs() < 0.001);
    let selected = canvas.replace_selection_from_marquee(
        &nodes,
        GraphNodeBounds::from_corners(GraphPoint::ZERO, GraphPoint::new(150.0, 100.0)),
    );
    assert_eq!(selected, ["first".to_string()].into_iter().collect());
}

#[test]
fn canvas_drag_emits_deterministic_graph_local_position_deltas() {
    let nodes = vec![
        node("first", GraphPoint::new(10.0, 10.0), &[], &[]),
        node("second", GraphPoint::new(220.0, 10.0), &[], &[]),
    ];
    let mut canvas = GraphCanvasState::default();
    canvas.replace_selection(["first".to_string()]);

    let drag = canvas
        .begin_node_drag(&nodes, GraphPoint::new(20.0, 20.0))
        .expect("the selected node starts a drag");
    let moves = drag.moved_nodes(GraphPoint::new(55.0, 45.0));

    assert_eq!(
        moves,
        vec![super::GraphNodeMove {
            node_id: "first".to_string(),
            position: GraphPoint::new(45.0, 35.0),
        }]
    );
}

#[test]
fn orthogonal_routing_is_stable_and_preserves_endpoints() {
    let source = GraphPoint::new(120.0, 32.0);
    let target = GraphPoint::new(300.0, 180.0);
    let route = route_connection(source, target, GraphRouteStyle::Orthogonal);

    assert_eq!(route.points().first(), Some(&source));
    assert_eq!(route.points().last(), Some(&target));
    assert!(
        route
            .points()
            .windows(2)
            .all(|points| points[0] != points[1])
    );
    assert!(
        route
            .points()
            .windows(2)
            .all(|points| points[0].x == points[1].x || points[0].y == points[1].y)
    );
}

#[test]
fn required_input_diagnostics_only_report_unconnected_required_pins() {
    let nodes = vec![
        node("source", GraphPoint::ZERO, &[], &[("out", "flow")]),
        node(
            "target",
            GraphPoint::new(180.0, 0.0),
            &[("required", "flow", true), ("optional", "flow", false)],
            &[],
        ),
    ];

    let diagnostics = required_input_diagnostics(&nodes, &[]);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].port.direction, GraphPinDirection::Input);
    assert_eq!(diagnostics[0].port.pin_name, "required");

    assert!(
        required_input_diagnostics(&nodes, &[edge("source", "out", "target", "required")])
            .is_empty()
    );
}

#[test]
fn alignment_uses_group_edges_and_preserves_each_node_extent() {
    let nodes = vec![
        node("first", GraphPoint::new(10.0, 20.0), &[], &[]),
        node("second", GraphPoint::new(80.0, 60.0), &[], &[]),
        node("third", GraphPoint::new(180.0, 100.0), &[], &[]),
    ];
    let mut canvas = GraphCanvasState::default();
    canvas.replace_selection([
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ]);

    let center_moves = aligned_node_moves(&nodes, canvas.selection(), GraphAlignment::Center);
    let bottom_moves = aligned_node_moves(&nodes, canvas.selection(), GraphAlignment::Bottom);

    assert_eq!(center_moves.len(), 3);
    assert!(center_moves.iter().all(|move_| move_.position.x == 95.0));
    assert_eq!(bottom_moves.len(), 2);
    assert!(bottom_moves.iter().all(|move_| move_.position.y == 100.0));
}

#[test]
fn graph_delta_command_round_trips_through_the_shared_edit_command_contract() {
    let mut context = TestGraphContext {
        graph_id: "primary".to_string(),
        graph: TestGraph::default(),
    };
    let mut command = GraphDeltaCommand::<TestGraph, TestGraphContext, String>::new(
        "Move graph nodes",
        "primary".to_string(),
        7,
    );

    EditCommand::apply(&mut command, &mut context).unwrap();
    assert_eq!(context.graph.value, 7);

    let mut wrong_context = TestGraphContext {
        graph_id: "other".to_string(),
        graph: TestGraph::default(),
    };
    assert!(EditCommand::revert(&mut command, &mut wrong_context).is_err());
    assert_eq!(wrong_context.graph.value, 0);

    EditCommand::revert(&mut command, &mut context).unwrap();
    assert_eq!(context.graph.value, 0);

    EditCommand::apply(&mut command, &mut context).unwrap();
    assert_eq!(context.graph.value, 7);
}

#[derive(Default)]
struct TestGraph {
    value: i32,
}

impl super::GraphModel for TestGraph {
    type NodeId = String;
    type Delta = i32;
    type Error = Infallible;

    fn nodes(&self) -> Vec<GraphNodeView<Self::NodeId>> {
        Vec::new()
    }

    fn edges(&self) -> Vec<GraphEdgeView<Self::NodeId>> {
        Vec::new()
    }

    fn palette(&self) -> &crate::core::editor_authoring_extension::GraphNodePaletteDescriptor {
        unreachable!("the transaction adapter does not request palette data")
    }

    fn can_connect(
        &self,
        _from: GraphPortRef<Self::NodeId>,
        _to: GraphPortRef<Self::NodeId>,
    ) -> super::ConnectVerdict {
        unreachable!("the transaction adapter does not request connection validation")
    }

    fn structure_constraint(&self) -> StructureConstraint {
        StructureConstraint::FreeGraph
    }

    fn apply(
        &mut self,
        delta: Self::Delta,
    ) -> Result<GraphMutationEffect<Self::Delta>, Self::Error> {
        self.value += delta;
        Ok(GraphMutationEffect::Applied { inverse: -delta })
    }
}

struct TestGraphContext {
    graph_id: String,
    graph: TestGraph,
}

impl EditContext for TestGraphContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        unreachable!("graph transaction tests do not use the runtime gateway")
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl GraphEditContext<TestGraph, String> for TestGraphContext {
    fn graph_model_mut(&mut self, target: &String) -> Result<&mut TestGraph, EditCommandError> {
        if target == &self.graph_id {
            Ok(&mut self.graph)
        } else {
            Err(EditCommandError::TargetMissing {
                target: target.clone(),
            })
        }
    }
}
