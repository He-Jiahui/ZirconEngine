use super::{
    aligned_node_moves, default_connection_verdict, required_input_diagnostics, route_connection,
    GraphAlignment, GraphCanvasState, GraphDeltaCommand, GraphEdgeView, GraphEditContext,
    GraphMutationEffect, GraphNodeBounds, GraphNodeView, GraphPinDirection, GraphPinView,
    GraphPoint, GraphPortRef, GraphRouteStyle, StructureConstraint,
};
use crate::core::editing::engine::{
    EditCommand, EditCommandError, EditContext, EditWorldRoute, SelectionSnapshot,
};
use crate::core::play::WorldDomain;
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
    assert!(default_connection_verdict(
        &nodes,
        &[connection.clone()],
        &backwards,
        StructureConstraint::Dag,
    )
    .is_rejected());

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
    assert!(route
        .points()
        .windows(2)
        .all(|points| points[0] != points[1]));
    assert!(route
        .points()
        .windows(2)
        .all(|points| points[0].x == points[1].x || points[0].y == points[1].y));
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
    fn retention_revision_v1(&self) -> u64 {
        self.value as u64
    }
    fn delta_retention_v1(
        &self,
        _delta: &i32,
        meter: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<super::model::GraphDeltaProofV1, EditCommandError> {
        meter.layout(2 * std::mem::size_of::<i32>(), std::mem::align_of::<i32>())?;
        Ok(super::model::GraphDeltaProofV1 {
            revision: self.value as u64,
            retained_pair_bytes: 8,
            materialization_peak_bytes: 8,
        })
    }
    fn prepare_delta_v1(
        &self,
        delta: &i32,
    ) -> Result<super::model::PreparedGraphDeltaV1<i32>, Infallible> {
        Ok(super::model::PreparedGraphDeltaV1 {
            revision: self.value as u64,
            forward: *delta,
            inverse: -*delta,
        })
    }
    fn apply_prepared_delta_v1(
        &mut self,
        prepared: &super::model::PreparedGraphDeltaV1<i32>,
    ) -> Result<(), Infallible> {
        assert_eq!(prepared.revision, self.value as u64);
        self.value += prepared.forward;
        Ok(())
    }

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
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(world_domain))
    }

    fn activate_world_route(&mut self, _route: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn retire_world_route(&mut self, _world_domain: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn selection_retention_bound_v1(
        &self,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError> {
        SelectionSnapshot::scalar_retention_bound_v1(model, limit)
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
    fn graph_model(&self, target: &String) -> Result<&TestGraph, EditCommandError> {
        if target == &self.graph_id {
            Ok(&self.graph)
        } else {
            Err(EditCommandError::TargetMissing {
                target: target.clone(),
            })
        }
    }

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

use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, HistoryRetentionBudgetV1, HistoryRetentionDomainV1,
    RetentionPhaseV1,
};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use zircon_runtime_interface::retained_memory::{
    verified_std_model_v1, ImmutableRetainedOwnerV1, RetainedBoundBuilderV1, RetainedOwnerV1,
    RetentionContractError,
};

struct RetainedGraphV1 {
    value: Arc<Vec<u8>>,
    revision: u64,
    preparations: Arc<AtomicUsize>,
}
impl super::GraphModel for RetainedGraphV1 {
    type NodeId = String;
    type Delta = Arc<Vec<u8>>;
    type Error = Infallible;
    fn retention_revision_v1(&self) -> u64 {
        self.revision
    }
    fn delta_retention_v1(
        &self,
        delta: &Self::Delta,
        meter: &mut RetainedBoundBuilderV1,
    ) -> Result<super::model::GraphDeltaProofV1, EditCommandError> {
        let start = meter.bytes().get();
        meter.owned(delta)?;
        meter.owned(&self.value)?;
        let pair = meter.bytes().get() - start;
        Ok(super::model::GraphDeltaProofV1 {
            revision: self.revision,
            retained_pair_bytes: pair,
            materialization_peak_bytes: pair
                .checked_mul(2)
                .ok_or(RetentionContractError::Overflow)?,
        })
    }
    fn prepare_delta_v1(
        &self,
        delta: &Self::Delta,
    ) -> Result<super::model::PreparedGraphDeltaV1<Self::Delta>, Infallible> {
        self.preparations.fetch_add(1, Ordering::SeqCst);
        // A shared immutable source remains valid: materialize the writable
        // publication copy only after the engine grants this exact revision.
        let mut forward = Vec::with_capacity(delta.len());
        forward.extend_from_slice(delta.as_slice());
        Ok(super::model::PreparedGraphDeltaV1 {
            revision: self.revision,
            forward: Arc::new(forward),
            inverse: self.value.clone(),
        })
    }
    fn apply_prepared_delta_v1(
        &mut self,
        prepared: &super::model::PreparedGraphDeltaV1<Self::Delta>,
    ) -> Result<(), Infallible> {
        assert_eq!(prepared.revision, self.revision);
        self.value = prepared.forward.clone();
        self.revision += 1;
        Ok(())
    }
    fn nodes(&self) -> Vec<GraphNodeView<String>> {
        Vec::new()
    }
    fn edges(&self) -> Vec<GraphEdgeView<String>> {
        Vec::new()
    }
    fn palette(&self) -> &crate::core::editor_authoring_extension::GraphNodePaletteDescriptor {
        unreachable!()
    }
    fn can_connect(
        &self,
        _: GraphPortRef<String>,
        _: GraphPortRef<String>,
    ) -> super::ConnectVerdict {
        unreachable!()
    }
    fn structure_constraint(&self) -> StructureConstraint {
        StructureConstraint::FreeGraph
    }
    fn apply(
        &mut self,
        delta: Self::Delta,
    ) -> Result<GraphMutationEffect<Self::Delta>, Infallible> {
        let inverse = std::mem::replace(&mut self.value, delta);
        self.revision += 1;
        Ok(GraphMutationEffect::Applied { inverse })
    }
}
struct RetainedGraphContextV1 {
    graph: RetainedGraphV1,
    model_reads: Arc<AtomicUsize>,
}
impl EditContext for RetainedGraphContextV1 {
    fn capture_world_route(&self, domain: WorldDomain) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(domain))
    }
    fn activate_world_route(&mut self, _: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn retire_world_route(&mut self, _: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn selection_retention_bound_v1(
        &self,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError> {
        SelectionSnapshot::scalar_retention_bound_v1(model, limit)
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn restore_selection(&mut self, _: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl GraphEditContext<RetainedGraphV1, String> for RetainedGraphContextV1 {
    fn graph_model(&self, _: &String) -> Result<&RetainedGraphV1, EditCommandError> {
        self.model_reads.fetch_add(1, Ordering::SeqCst);
        Ok(&self.graph)
    }
    fn graph_model_mut(&mut self, _: &String) -> Result<&mut RetainedGraphV1, EditCommandError> {
        Ok(&mut self.graph)
    }
}
struct RetainedGraphTargetV1 {
    id: String,
    domain: HistoryRetentionDomainV1,
    retiring_seen: Arc<AtomicU64>,
}
impl RetainedOwnerV1 for RetainedGraphTargetV1 {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.id.retained_bound_v1(m)?;
        self.retiring_seen.retained_bound_v1(m)
        // The domain's separate mandatory ledger charges its own mutable storage.
    }
}
unsafe impl ImmutableRetainedOwnerV1 for RetainedGraphTargetV1 {}
impl Drop for RetainedGraphTargetV1 {
    fn drop(&mut self) {
        self.retiring_seen.store(
            self.domain.snapshot().unwrap().retiring_bytes,
            Ordering::SeqCst,
        );
    }
}
impl GraphEditContext<RetainedGraphV1, RetainedGraphTargetV1> for RetainedGraphContextV1 {
    fn graph_model(&self, _: &RetainedGraphTargetV1) -> Result<&RetainedGraphV1, EditCommandError> {
        self.model_reads.fetch_add(1, Ordering::SeqCst);
        Ok(&self.graph)
    }
    fn graph_model_mut(
        &mut self,
        _: &RetainedGraphTargetV1,
    ) -> Result<&mut RetainedGraphV1, EditCommandError> {
        Ok(&mut self.graph)
    }
}
fn retained_graph_engine_v1(
    initial: usize,
    capacity: usize,
) -> (
    EditorTransactionEngine,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    HistoryRetentionDomainV1,
) {
    let preparations = Arc::new(AtomicUsize::new(0));
    let reads = Arc::new(AtomicUsize::new(0));
    let domain = HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes: 32 * 1024,
        history_bytes: 64 * 1024,
        committed_bytes: 4 * 1024 * 1024,
        staging_bytes: 1024 * 1024,
    })
    .unwrap();
    let engine = EditorTransactionEngine::with_retention_domain_v1(
        RetainedGraphContextV1 {
            graph: RetainedGraphV1 {
                value: Arc::new(vec![1; initial]),
                revision: 0,
                preparations: preparations.clone(),
            },
            model_reads: reads.clone(),
        },
        capacity,
        domain.clone(),
    )
    .unwrap();
    (engine, preparations, reads, domain)
}

#[test]
fn graph_delta_prepares_the_inverse_bound_before_model_mutation() {
    let (engine, preparations, _, _) = retained_graph_engine_v1(40 * 1024, 128);
    let mut scope = engine
        .begin("graph inverse", HistoryContextId::Global)
        .unwrap();
    let result = scope.push(GraphDeltaCommand::<
        RetainedGraphV1,
        RetainedGraphContextV1,
        String,
    >::new(
        "small forward", "graph".to_string(), Arc::new(vec![2; 8])
    ));
    scope.cancel().unwrap();
    assert!(matches!(result, Err(EditCommandError::Retention(_))));
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
    assert_eq!(
        engine
            .with_context::<RetainedGraphContextV1, _>(|c| (c.graph.value.len(), c.graph.revision))
            .unwrap(),
        Some((40 * 1024, 0))
    );
}

#[test]
fn graph_history_charges_label_and_target_capacity_before_model_access() {
    let (engine, preparations, reads, _) = retained_graph_engine_v1(8, 128);
    for large_label in [true, false] {
        let mut label = String::with_capacity(if large_label { 40 * 1024 } else { 8 });
        label.push_str("graph");
        let mut target = String::with_capacity(if large_label { 8 } else { 40 * 1024 });
        target.push_str("graph");
        let mut scope = engine
            .begin("owner refusal", HistoryContextId::Global)
            .unwrap();
        let result = scope.push(GraphDeltaCommand::<
            RetainedGraphV1,
            RetainedGraphContextV1,
            String,
        >::new(label, target, Arc::new(vec![2; 8])));
        scope.cancel().unwrap();
        assert!(matches!(result, Err(EditCommandError::Retention(_))));
    }
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
}

#[test]
fn graph_history_reserves_copy_on_write_and_replay_growth_before_mutation() {
    let (engine, preparations, _, _) = retained_graph_engine_v1(32, 128);
    let source = Arc::new(vec![7; 64]);
    let source_copy = source.clone();
    let mut scope = engine
        .begin("shared source", HistoryContextId::Global)
        .unwrap();
    scope
        .push(GraphDeltaCommand::<
            RetainedGraphV1,
            RetainedGraphContextV1,
            String,
        >::new("COW", "graph".to_string(), source))
        .unwrap();
    scope.commit().unwrap();
    assert_eq!(source_copy.as_slice(), &[7; 64]);
    assert_eq!(preparations.load(Ordering::SeqCst), 1);
    engine
        .with_context_mut::<RetainedGraphContextV1, _>(|context| {
            context.graph.value = Arc::new(vec![9; 40 * 1024]);
            context.graph.revision += 1;
        })
        .unwrap();
    let before = engine.history_status(HistoryContextId::Global).unwrap();
    let refused = engine.undo(HistoryContextId::Global);
    assert!(matches!(refused, Err(EditCommandError::Retention(_))));
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap(),
        before
    );
    assert_eq!(preparations.load(Ordering::SeqCst), 1);
    assert_eq!(
        engine
            .with_context::<RetainedGraphContextV1, _>(|c| c.graph.value.len())
            .unwrap(),
        Some(40 * 1024)
    );
}

#[test]
fn graph_history_keeps_last_owner_charged_until_unlocked_drop() {
    let (engine, _, _, domain) = retained_graph_engine_v1(32, 1);
    let seen = Arc::new(AtomicU64::new(0));
    let mut scope = engine
        .begin("graph owner", HistoryContextId::Global)
        .unwrap();
    scope
        .push(GraphDeltaCommand::<
            RetainedGraphV1,
            RetainedGraphContextV1,
            RetainedGraphTargetV1,
        >::new(
            "last owner",
            RetainedGraphTargetV1 {
                id: "graph".to_string(),
                domain: domain.clone(),
                retiring_seen: seen.clone(),
            },
            Arc::new(vec![3; 1024]),
        ))
        .unwrap();
    scope.commit().unwrap();
    let mut scope = engine
        .begin("retire graph owner", HistoryContextId::Global)
        .unwrap();
    scope
        .push(GraphDeltaCommand::<
            RetainedGraphV1,
            RetainedGraphContextV1,
            String,
        >::new(
            "next", "graph".to_string(), Arc::new(vec![4; 64])
        ))
        .unwrap();
    scope.commit().unwrap();
    assert!(seen.load(Ordering::SeqCst) >= 1024);
    assert_eq!(domain.snapshot().unwrap().retiring_bytes, 0);
}
