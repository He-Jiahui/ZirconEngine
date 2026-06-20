use std::any::TypeId;

use crate::scene::ecs::{
    Component, QueryState, ResMutParam, ResParam, Resource, ScheduleConflictGraph,
    ScheduleConflictNode, SystemParamAccess, SystemParamConflictKind, SystemStage, SystemState,
    With, Without,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHealth(u32);

impl Component for ScheduleHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SchedulePlayer;

impl Component for SchedulePlayer {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleFrameCounter(u32);

impl Resource for ScheduleFrameCounter {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHitEvent;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleNoticeMessage;

#[test]
fn schedule_conflict_graph_reports_component_write_conflicts_in_same_stage() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let health_component = world.component_id::<ScheduleHealth>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);

    assert_eq!(graph.nodes().len(), 2);
    assert!(graph.has_conflicts());
    let edge = &graph.edges()[0];
    assert_eq!(edge.left_system_id(), "read.health");
    assert_eq!(edge.right_system_id(), "write.health");
    assert_eq!(edge.stage(), SystemStage::Update);
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Component(health_component)]
    );
    assert_eq!(graph.conflicts_for("read.health").count(), 1);
}

#[test]
fn schedule_conflict_graph_respects_disjoint_query_filters() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<&'static mut ScheduleHealth, With<SchedulePlayer>>;
    type NonPlayerHealth = QueryState<&'static mut ScheduleHealth, Without<SchedulePlayer>>;
    let player_health = SystemState::<PlayerHealth>::new(&mut world).unwrap();
    let non_player_health = SystemState::<NonPlayerHealth>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "write.player-health",
            SystemStage::Update,
            player_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.non-player-health",
            SystemStage::Update,
            non_player_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
    assert!(graph.edges().is_empty());
}

#[test]
fn schedule_conflict_graph_keeps_different_stages_independent() {
    let mut world = World::empty();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::PreUpdate,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::PostUpdate,
            write_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
}

#[test]
fn schedule_conflict_graph_reports_resource_write_conflicts() {
    let mut world = World::empty();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_counter = SystemState::<ResMutParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let counter_resource = world.resource_id::<ScheduleFrameCounter>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.frame-counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.frame-counter",
            SystemStage::Update,
            write_counter.access().clone(),
        ),
    ]);

    assert!(read_counter.access().conflicts_with(write_counter.access()));
    let edge = &graph.edges()[0];
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Resource(counter_resource)]
    );
}

#[test]
fn schedule_conflict_graph_reports_event_and_message_write_conflicts() {
    let mut event_reader = SystemParamAccess::default();
    event_reader.add_event_read::<ScheduleHitEvent>().unwrap();
    let mut event_writer = SystemParamAccess::default();
    event_writer.add_event_write::<ScheduleHitEvent>().unwrap();
    let mut message_reader = SystemParamAccess::default();
    message_reader
        .add_message_read::<ScheduleNoticeMessage>()
        .unwrap();
    let mut message_writer = SystemParamAccess::default();
    message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("read.event", SystemStage::Update, event_reader),
        ScheduleConflictNode::new("write.event", SystemStage::Update, event_writer),
        ScheduleConflictNode::new("read.message", SystemStage::Update, message_reader),
        ScheduleConflictNode::new("write.message", SystemStage::Update, message_writer),
    ]);
    let event_type = TypeId::of::<ScheduleHitEvent>();
    let message_type = TypeId::of::<ScheduleNoticeMessage>();

    assert_eq!(graph.edges().len(), 2);
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Event(event_type))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Message(message_type))
    }));
}

#[test]
fn schedule_conflict_graph_reports_event_and_message_writer_conflicts() {
    let mut first_event_writer = SystemParamAccess::default();
    first_event_writer
        .add_event_write::<ScheduleHitEvent>()
        .unwrap();
    let mut second_event_writer = SystemParamAccess::default();
    second_event_writer
        .add_event_write::<ScheduleHitEvent>()
        .unwrap();
    let mut first_message_writer = SystemParamAccess::default();
    first_message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();
    let mut second_message_writer = SystemParamAccess::default();
    second_message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("write.event.first", SystemStage::Update, first_event_writer),
        ScheduleConflictNode::new(
            "write.event.second",
            SystemStage::Update,
            second_event_writer,
        ),
        ScheduleConflictNode::new(
            "write.message.first",
            SystemStage::Update,
            first_message_writer,
        ),
        ScheduleConflictNode::new(
            "write.message.second",
            SystemStage::Update,
            second_message_writer,
        ),
    ]);
    let event_type = TypeId::of::<ScheduleHitEvent>();
    let message_type = TypeId::of::<ScheduleNoticeMessage>();

    assert_eq!(graph.edges().len(), 2);
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Event(event_type))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Message(message_type))
    }));
}

#[test]
fn schedule_conflict_graph_reports_conservative_world_access_conflicts() {
    let mut world_access = SystemParamAccess::default();
    world_access.add_conservative_world_access();
    let read_only = SystemParamAccess::default();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("runtime.context", SystemStage::Update, world_access),
        ScheduleConflictNode::new("native.read-only", SystemStage::Update, read_only),
    ]);

    assert!(graph.systems_conflict("runtime.context", "native.read-only"));
    assert_eq!(
        graph.edges()[0].conflicts(),
        &[SystemParamConflictKind::World]
    );
}

#[test]
fn schedule_conflict_graph_builds_conservative_parallel_batches() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);

    assert!(graph.systems_conflict("read.health", "write.health"));
    assert!(!graph.systems_conflict("read.counter", "write.health"));

    let graph_source = include_str!("../../ecs/schedule_conflict_graph.rs");
    assert!(graph_source.contains("use std::collections::HashMap;"));
    assert!(graph_source.contains("node_indices_by_system_id: HashMap<String, usize>"));
    assert!(graph_source.contains("conflict_edge_indices_by_node: Vec<Vec<usize>>"));
    assert!(graph_source.contains("conflict_node_adjacency: Vec<Vec<usize>>"));
    assert!(graph_source.contains("let graph_inputs = schedule_conflict_graph_inputs(&nodes);"));
    assert!(
        graph_source.contains("node_indices_by_system_id: graph_inputs.node_indices_by_system_id")
    );
    assert!(graph_source
        .contains("let node_indices_by_stage = node_indices_by_stage(&nodes, &graph_inputs);"));
    assert!(
        graph_source.contains("same_stage_non_barrier_conflict_pair_upper_bound(&graph_inputs)")
    );
    assert!(graph_source.contains("let mut edges = Vec::with_capacity(edge_upper_bound);"));
    assert!(graph_source.contains("let mut next_stage_positions = [0_usize; SystemStage::COUNT];"));
    assert!(graph_source.contains(
        "let mut batches = Vec::<ScheduleParallelBatch>::with_capacity(self.nodes.len())"
    ));
    assert!(graph_source.contains("if let Some(batch) = batches.last_mut()"));
    assert!(graph_source.contains("let can_extend_last_batch = batch.stage == node.stage"));
    assert!(graph_source.contains(
        "if can_extend_last_batch {\n                    batch.push_system(node.system_id.clone(), node_index);\n                    continue;\n                }"
    ));
    assert!(graph_source.contains("if nodes.len() <= 1"));
    assert!(graph_source
        .contains("let conflict_edge_indices_by_node = empty_node_index_lists(nodes.len());"));
    assert!(
        graph_source.contains("let conflict_node_adjacency = empty_node_index_lists(nodes.len());")
    );
    assert!(graph_source.contains("edges: Vec::new(),"));
    assert!(
        graph_source.contains("fn empty_node_index_lists(node_count: usize) -> Vec<Vec<usize>>")
    );
    assert!(graph_source.contains("let mut lists = Vec::with_capacity(node_count);"));
    assert!(graph_source.contains("lists.resize_with(node_count, Vec::new);"));
    assert!(graph_source.contains("for left_index in 0..nodes.len()"));
    assert!(
        graph_source.contains("let same_stage_node_indices = &node_indices_by_stage[stage_index];")
    );
    assert!(graph_source.contains("let left_stage_index = next_stage_positions[stage_index];"));
    assert!(graph_source.contains("next_stage_positions[stage_index] += 1;"));
    assert!(graph_source
        .contains("for right_index in &same_stage_node_indices[(left_stage_index + 1)..]"));
    assert!(
        graph_source
            .matches("empty_node_index_lists(nodes.len())")
            .count()
            >= 4,
        "trivial and multi-node graph paths should share the empty node-index list helper"
    );
    assert!(graph_source.contains("&mut conflict_edge_indices_by_node"));
    assert!(graph_source.contains("&mut conflict_node_adjacency"));
    assert!(graph_source.contains("conflict_edge_indices_by_node: &mut [Vec<usize>]"));
    assert!(graph_source.contains("conflict_node_adjacency: &mut [Vec<usize>]"));
    assert!(graph_source.contains("Some(&self.graph.edges[edge_index])"));
    assert!(graph_source.contains("self.node_indices_by_system_id"));
    assert!(graph_source.contains("self.conflict_edge_indices_by_node[*node_index]"));
    assert!(graph_source.contains("match self.node_indices_by_system_id.get(system_id)"));
    assert!(graph_source.contains(
        "Some(node_index) => self.conflict_edge_indices_by_node[*node_index].as_slice()"
    ));
    assert!(graph_source.contains("None => &[],"));
    assert!(graph_source.contains("ScheduleConflictEdges::new(self, edge_indices)"));
    assert!(graph_source.contains("struct ScheduleConflictEdges<'graph>"));
    assert!(graph_source.contains("impl<'graph> Iterator for ScheduleConflictEdges<'graph>"));
    assert!(graph_source.contains("let edge_index = self.edge_indices[self.next_index];"));
    assert!(!graph_source
        .contains(".map(|node_index| self.conflict_edge_indices_by_node[*node_index].as_slice())"));
    assert!(!graph_source.contains("edge_indices.iter().map(|index| &self.edges[*index])"));
    assert!(!graph_source.contains("&edges[*index]"));
    assert!(graph_source.contains("self.conflict_node_adjacency"));
    assert!(graph_source
        .contains("let Some(left_index) = self.node_indices_by_system_id.get(left_system_id)"));
    assert!(graph_source
        .contains("let Some(right_index) = self.node_indices_by_system_id.get(right_system_id)"));
    assert!(graph_source.contains("return false;"));
    assert!(graph_source.contains("self.node_indices_conflict(*left_index, *right_index)"));
    assert!(graph_source.contains("struct ScheduleConflictGraphInputs"));
    assert!(graph_source.contains("node_indices_by_system_id: HashMap<String, usize>"));
    assert!(graph_source.contains("node_counts_by_stage: [usize; SystemStage::COUNT]"));
    assert!(graph_source.contains("non_barrier_node_counts_by_stage: [usize; SystemStage::COUNT]"));
    assert!(graph_source.contains(
        "fn schedule_conflict_graph_inputs(nodes: &[ScheduleConflictNode]) -> ScheduleConflictGraphInputs"
    ));
    assert!(graph_source
        .contains("let mut node_indices_by_system_id = HashMap::with_capacity(nodes.len())"));
    assert!(
        graph_source.contains("node_indices_by_system_id.insert(node.system_id.clone(), index);")
    );
    assert!(graph_source.contains("let mut node_counts_by_stage = [0_usize; SystemStage::COUNT];"));
    assert!(graph_source
        .contains("let mut non_barrier_node_counts_by_stage = [0_usize; SystemStage::COUNT];"));
    assert!(graph_source.contains("let stage_index = node.stage.rank();"));
    assert!(graph_source.contains("node_counts_by_stage[stage_index] += 1;"));
    assert!(graph_source.contains("if !node.is_barrier()"));
    assert!(graph_source.contains("non_barrier_node_counts_by_stage[stage_index] += 1;"));
    assert!(graph_source.contains(
        "fn node_indices_by_stage(\n    nodes: &[ScheduleConflictNode],\n    graph_inputs: &ScheduleConflictGraphInputs,"
    ));
    assert!(
        graph_source.contains("Vec::with_capacity(graph_inputs.node_counts_by_stage[stage_index])")
    );
    assert!(graph_source.contains("node_indices_by_stage[node.stage.rank()].push(index);"));
    assert!(graph_source
        .contains("fn same_stage_non_barrier_conflict_pair_upper_bound(\n    graph_inputs: &ScheduleConflictGraphInputs,"));
    assert!(graph_source.contains("let mut edge_upper_bound = 0;"));
    assert!(
        graph_source.contains("for node_count in graph_inputs.non_barrier_node_counts_by_stage")
    );
    assert!(graph_source.contains("edge_upper_bound += conflict_pair_count(node_count);"));
    assert!(graph_source.contains("edge_upper_bound\n}"));
    assert!(graph_source.contains("fn conflict_pair_count(node_count: usize) -> usize"));
    assert!(graph_source.contains("node_count.saturating_mul(node_count.saturating_sub(1)) / 2"));
    let batch_struct = graph_source
        .split("pub struct ScheduleParallelBatch {")
        .nth(1)
        .and_then(|text| text.split("enum ScheduleParallelBatchSystems").next())
        .expect("read ScheduleParallelBatch declaration");
    assert!(batch_struct.contains("systems: ScheduleParallelBatchSystems"));
    assert!(!batch_struct.contains("system_ids: Vec<String>"));
    assert!(!batch_struct.contains("node_indices: Vec<usize>"));
    assert!(graph_source.contains("enum ScheduleParallelBatchSystems"));
    assert!(
        graph_source.contains("Single {\n        system_id: String,\n        node_index: usize,")
    );
    assert!(graph_source
        .contains("Pair {\n        system_ids: [String; 2],\n        node_indices: [usize; 2],"));
    assert!(graph_source
        .contains("Triple {\n        system_ids: [String; 3],\n        node_indices: [usize; 3],"));
    assert!(graph_source.contains(
        "Multiple {\n        system_ids: Vec<String>,\n        node_indices: Vec<usize>,"
    ));
    assert!(graph_source.contains("ScheduleParallelBatch::single("));
    assert!(
        graph_source.contains("fn push_system(&mut self, system_id: String, node_index: usize)")
    );
    assert!(graph_source.contains("batch.push_system(node.system_id.clone(), node_index);"));
    assert!(graph_source.contains("ScheduleParallelBatchSystems::Pair {"));
    assert!(graph_source.contains("system_ids: [std::mem::take(first_system_id), system_id]"));
    assert!(graph_source.contains("node_indices: [*first_node_index, node_index]"));
    assert!(graph_source.contains("ScheduleParallelBatchSystems::Triple {"));
    assert!(graph_source
        .contains("system_ids: [\n                    std::mem::take(&mut system_ids[0]),"));
    assert!(graph_source.contains("node_indices: [node_indices[0], node_indices[1], node_index]"));
    assert!(graph_source.contains("let mut promoted_system_ids = Vec::with_capacity(4);"));
    assert!(graph_source.contains("promoted_system_ids.push(std::mem::take(&mut system_ids[0]));"));
    assert!(graph_source.contains("promoted_system_ids.push(std::mem::take(&mut system_ids[1]));"));
    assert!(graph_source.contains("promoted_system_ids.push(std::mem::take(&mut system_ids[2]));"));
    assert!(graph_source.contains("let mut promoted_node_indices = Vec::with_capacity(4);"));
    assert!(graph_source.contains("promoted_node_indices.extend_from_slice(node_indices);"));
    assert!(graph_source.contains("fn node_indices(&self) -> &[usize]"));
    assert!(graph_source.contains("std::slice::from_ref(node_index)"));
    assert!(graph_source.contains("std::slice::from_ref(system_id)"));
    assert!(graph_source.contains(
        "ScheduleParallelBatchSystems::Pair { node_indices, .. } => node_indices.as_slice()"
    ));
    assert!(graph_source.contains(
        "ScheduleParallelBatchSystems::Triple { node_indices, .. } => node_indices.as_slice()"
    ));
    assert!(graph_source.contains(
        "ScheduleParallelBatchSystems::Pair { system_ids, .. } => system_ids.as_slice()"
    ));
    assert!(graph_source.contains(
        "ScheduleParallelBatchSystems::Triple { system_ids, .. } => system_ids.as_slice()"
    ));
    assert!(graph_source.contains(".node_indices"));
    assert!(graph_source.contains("node_indices_conflict("));
    assert!(graph_source.contains("node_indices_conflict_with_any("));
    assert!(
        graph_source.contains("let Some(neighbors) = self.conflict_node_adjacency.get(left_index)")
    );
    assert!(graph_source
        .contains("let Some(neighbors) = self.conflict_node_adjacency.get(right_index)"));
    assert!(graph_source.contains("neighbors.binary_search(&right_index).is_ok()"));
    assert!(graph_source.contains("sorted_slices_intersect(node_indices, neighbors)"));
    assert!(graph_source
        .contains("fn sorted_slices_intersect(left: &[usize], right: &[usize]) -> bool"));
    assert!(graph_source.contains("while left_index < left.len() && right_index < right.len()"));
    assert!(graph_source.contains("let left_value = left[left_index];"));
    assert!(graph_source.contains("let right_value = right[right_index];"));
    assert!(graph_source.contains("conflict_edge_indices_by_node[left_index].push(edge_index);"));
    assert!(graph_source.contains("conflict_edge_indices_by_node[right_index].push(edge_index);"));
    assert!(graph_source.contains("conflict_node_adjacency[left_index].push(right_index);"));
    assert!(graph_source.contains("conflict_node_adjacency[right_index].push(left_index);"));
    assert!(graph_source.contains("if !left.access.conflicts_with(&right.access)"));
    assert!(!graph_source.contains("use std::collections::{HashMap, HashSet};"));
    assert!(
        !graph_source.contains("vec![Vec::<usize>::new(); nodes.len()]"),
        "outer adjacency lists must use the shared exact-capacity helper"
    );
    assert!(!graph_source.contains("conflict_adjacency"));
    assert!(!graph_source.contains("conflict_edge_indices: HashMap<String, Vec<usize>>"));
    assert!(!graph_source.contains("HashMap::<String, Vec<usize>>::new()"));
    assert!(!graph_source.contains("HashSet<String>"));
    assert!(!graph_source.contains("let mut edges = Vec::new();"));
    assert!(!graph_source.contains("let edges = &self.edges;"));
    assert!(
        !graph_source.contains(".get(system_id)\n            .into_iter()\n            .flat_map")
    );
    assert!(!graph_source.contains("let mut batches = Vec::<ScheduleParallelBatch>::new();"));
    assert!(!graph_source.contains(".zip(self.node_indices_by_system_id.get(right_system_id))"));
    assert!(!graph_source.contains(".is_some_and(|(left_index, right_index)|"));
    assert!(!graph_source
        .contains(".is_some_and(|neighbors| neighbors.binary_search(&right_index).is_ok())"));
    assert!(!graph_source
        .contains(".is_some_and(|neighbors| sorted_slices_intersect(node_indices, neighbors))"));
    assert!(!graph_source.contains("batches.last().is_some_and"));
    assert!(!graph_source.contains("last batch must exist after is_some_and"));
    assert!(!graph_source.contains("system_ids: vec![node.system_id.clone()]"));
    assert!(!graph_source.contains("node_indices: vec![node_index]"));
    assert!(!graph_source.contains("std::mem::replace("));
    assert!(!graph_source.contains("system_ids: vec![first_system_id, system_id]"));
    assert!(!graph_source.contains("node_indices: vec![first_node_index, node_index]"));
    assert!(!graph_source.contains("let mut promoted_system_ids = Vec::with_capacity(3);"));
    assert!(!graph_source.contains("let mut promoted_node_indices = Vec::with_capacity(3);"));
    assert!(!graph_source.contains("fn node_indices_by_system_id(nodes: &[ScheduleConflictNode])"));
    assert!(!graph_source.contains("fn node_indices_by_system_id(nodes: &[ScheduleConflictNode]) -> HashMap<String, usize> {\n    nodes"));
    assert!(!graph_source.contains(".entry(left.system_id.clone())"));
    assert!(!graph_source.contains(".entry(right.system_id.clone())"));
    assert!(!graph_source.contains("for right_index in (left_index + 1)..nodes.len()"));
    assert!(!graph_source.contains("if left.stage != right.stage"));
    assert!(!graph_source
        .contains("let node_indices_by_system_id = node_indices_by_system_id(&nodes);"));
    assert!(
        !graph_source.contains("let (node_indices_by_stage, non_barrier_node_counts_by_stage) =")
    );
    assert!(!graph_source.contains(
        "same_stage_non_barrier_conflict_pair_upper_bound(&nodes, &node_indices_by_stage)"
    ));
    assert!(!graph_source.contains(
        "same_stage_non_barrier_conflict_pair_upper_bound(&non_barrier_node_counts_by_stage)"
    ));
    assert!(!graph_source.contains(".copied()\n        .map(conflict_pair_count)"));
    assert!(!graph_source.contains("filter(|node_index| !nodes[**node_index].is_barrier())"));
    assert!(!graph_source.contains("while let (Some(left_value), Some(right_value)) ="));
    assert!(!graph_source.contains(".binary_search(&left_index)"));
    assert!(!graph_source.contains("stage bucket must include every source node index"));
    assert!(!graph_source.contains("self.edges.iter().any(|edge|"));
    assert!(!graph_source.contains("self.edges.iter().filter(move |edge|"));
    assert!(!graph_source.contains("batch.node_indices.iter().all(|system_node_index|"));
    assert!(!graph_source.contains("!self.node_indices_conflict(*system_node_index, node_index)"));
    assert!(!graph_source.contains("conflict_node_adjacency: Vec<HashSet<usize>>"));
    assert!(!graph_source.contains("neighbors.contains(&right_index)"));
    assert!(!graph_source.contains(".system_ids\n                        .iter()"));
    assert!(!graph_source.contains("!self.systems_conflict(system_id, node.system_id())"));

    let access_source = include_str!("../../ecs/system/system_param_access.rs");
    assert!(access_source.contains("pub fn conflicts_with(&self, other: &Self) -> bool"));
    assert!(access_source.contains("resource_access_conflicts("));
    assert!(access_source.contains("type_access_conflicts("));
    assert!(access_source.contains("if let Err(index) = ids.binary_search(&resource_id)"));
    assert!(access_source.contains("if let Err(index) = ids.binary_search(&type_id)"));
    assert!(access_source.contains("ids.insert(index, resource_id);"));
    assert!(access_source.contains("ids.insert(index, type_id);"));
    assert!(access_source.contains("ids.binary_search(&type_id).is_ok()"));
    assert!(access_source.contains("query_access.writes().binary_search(component_id).is_err()"));
    assert!(access_source.contains("insert_type_id(&mut self.event_reads, type_id);\n        insert_type_id(&mut self.event_writes, type_id);"));
    assert!(access_source.contains("insert_type_id(&mut self.message_reads, type_id);\n        insert_type_id(&mut self.message_writes, type_id);"));
    assert!(access_source.contains("access_slices_intersect(left_writes, right_reads)"));
    assert!(access_source
        .contains("read_only_access_intersects(left_reads, left_writes, right_writes)"));
    assert!(access_source
        .contains("let mut conflicts = Vec::with_capacity(system_param_conflict_upper_bound("));
    assert!(access_source.contains("fn system_param_conflict_upper_bound("));
    assert!(access_source.contains("fn access_conflict_upper_bound<T>("));
    assert!(access_source.contains("read_only_access_count(left_reads, left_writes)"));
    assert!(access_source.contains("fn push_access_intersections<T>("));
    assert!(access_source.contains("fn push_read_only_access_intersections<T>("));
    assert!(access_source.contains("fn read_access_is_written<T>("));
    assert!(access_source.contains("fn push_conflict("));
    assert!(access_source.contains("conflicts.push(conflict);"));
    let access_boolean_intersect = access_source
        .split("fn access_slices_intersect<T>(")
        .nth(1)
        .and_then(|text| text.split("fn read_only_access_intersects<T>").next())
        .expect("read system-param access boolean intersection helper");
    assert!(access_boolean_intersect
        .contains("while left_index < left.len() && right_index < right.len()"));
    assert!(access_boolean_intersect.contains("let left_value = &left[left_index];"));
    assert!(access_boolean_intersect.contains("let right_value = &right[right_index];"));
    assert!(!access_boolean_intersect.contains("while let (Some(left_value), Some(right_value)) ="));
    let access_read_only_intersect = access_source
        .split("fn read_only_access_intersects<T>(")
        .nth(1)
        .and_then(|text| text.split("fn push_resource_conflicts").next())
        .expect("read system-param access read-only intersection helper");
    assert!(access_read_only_intersect
        .contains("while read_index < reads.len() && right_index < right.len()"));
    assert!(access_read_only_intersect.contains("let read_value = reads[read_index];"));
    assert!(access_read_only_intersect.contains("let right_value = right[right_index];"));
    assert!(access_read_only_intersect
        .contains("read_access_is_written(read_value, writes, &mut write_index)"));
    assert!(
        !access_read_only_intersect.contains("while let (Some(read_value), Some(right_value)) =")
    );
    let access_diagnostic_intersect = access_source
        .split("fn push_access_intersections<T>(")
        .nth(1)
        .and_then(|text| {
            text.split("fn push_read_only_access_intersections<T>")
                .next()
        })
        .expect("read system-param access diagnostic intersection helper");
    assert!(access_diagnostic_intersect
        .contains("while left_index < left.len() && right_index < right.len()"));
    assert!(access_diagnostic_intersect.contains("let left_value = left[left_index];"));
    assert!(access_diagnostic_intersect.contains("let right_value = right[right_index];"));
    assert!(access_diagnostic_intersect
        .contains("push_conflict(conflicts, conflict_kind(left_value));"));
    assert!(
        !access_diagnostic_intersect.contains("while let (Some(left_value), Some(right_value)) =")
    );
    assert!(!access_diagnostic_intersect.contains("conflict_kind(*left_value)"));
    let access_read_only_diagnostic = access_source
        .split("fn push_read_only_access_intersections<T>(")
        .nth(1)
        .and_then(|text| text.split("fn read_access_is_written<T>").next())
        .expect("read system-param read-only diagnostic helper");
    assert!(access_read_only_diagnostic
        .contains("while read_index < reads.len() && right_index < right.len()"));
    assert!(access_read_only_diagnostic.contains("let read_value = reads[read_index];"));
    assert!(access_read_only_diagnostic.contains("let right_value = right[right_index];"));
    assert!(access_read_only_diagnostic
        .contains("read_access_is_written(read_value, writes, &mut write_index)"));
    assert!(access_read_only_diagnostic
        .contains("push_conflict(conflicts, conflict_kind(read_value));"));
    assert!(
        !access_read_only_diagnostic.contains("while let (Some(read_value), Some(right_value)) =")
    );
    assert!(!access_read_only_diagnostic.contains("conflict_kind(*read_value)"));
    let access_write_probe = access_source
        .split("fn read_access_is_written<T>(")
        .nth(1)
        .and_then(|text| text.split("fn push_conflict(").next())
        .expect("read system-param write probe helper");
    assert!(access_write_probe.contains("while *write_index < writes.len()"));
    assert!(access_write_probe.contains("let write_value = writes[*write_index];"));
    assert!(access_write_probe.contains("if write_value < access_id"));
    assert!(access_write_probe.contains("if write_value == access_id"));
    assert!(!access_write_probe.contains("while let Some(write_value) = writes.get(*write_index)"));
    assert!(!access_source.contains("!self.conflict_kinds_with(other).is_empty()"));
    assert!(!access_source.contains("let mut conflicts = Vec::new();"));
    assert!(!access_source.contains("fn insert_conflict("));
    assert!(!access_source.contains("conflicts.contains(&conflict)"));
    assert!(!access_source.contains("query_access.writes().contains(component_id)"));
    assert!(!access_source.contains("ids.contains(&type_id)"));
    assert!(!access_source.contains("ids.sort_unstable();"));

    let query_access_source = include_str!("../../ecs/query/query_access.rs");
    assert!(query_access_source.contains(
        "fn sorted_component_slices_intersect(left: &[ComponentId], right: &[ComponentId]) -> bool"
    ));
    let sorted_component_intersect = query_access_source
        .split("fn sorted_component_slices_intersect(left: &[ComponentId], right: &[ComponentId]) -> bool")
        .nth(1)
        .expect("read sorted component intersection helper");
    assert!(sorted_component_intersect
        .contains("while left_index < left.len() && right_index < right.len()"));
    assert!(sorted_component_intersect.contains("let left_value = left[left_index];"));
    assert!(sorted_component_intersect.contains("let right_value = right[right_index];"));
    assert!(sorted_component_intersect.contains("if left_value == right_value"));
    assert!(sorted_component_intersect.contains("left_index += 1;"));
    assert!(sorted_component_intersect.contains("right_index += 1;"));
    assert!(query_access_source
        .contains("sorted_component_slices_intersect(&self.writes, &other.reads)"));
    assert!(query_access_source
        .contains("sorted_component_slices_intersect(&self.reads, &other.writes)"));
    assert!(!query_access_source
        .contains("sorted_component_slices_intersect(&self.writes, &other.writes)"));
    assert!(!query_access_source
        .contains("left.iter()\n        .any(|component_id| contains_id(right, *component_id))"));
    assert!(query_access_source.contains(
        "fn push_sorted_component_intersections(\n    conflicts: &mut Vec<ComponentId>,"
    ));
    let sorted_component_diagnostic = query_access_source
        .split("fn push_sorted_component_intersections(")
        .nth(1)
        .and_then(|text| {
            text.split("fn push_read_only_component_intersections")
                .next()
        })
        .expect("read sorted component diagnostic helper");
    assert!(sorted_component_diagnostic
        .contains("while left_index < left.len() && right_index < right.len()"));
    assert!(sorted_component_diagnostic.contains("let left_value = left[left_index];"));
    assert!(sorted_component_diagnostic.contains("let right_value = right[right_index];"));
    assert!(
        sorted_component_diagnostic.contains("insert_sorted_component_id(conflicts, left_value);")
    );
    assert!(
        !sorted_component_diagnostic.contains("while let (Some(left_value), Some(right_value)) =")
    );
    assert!(query_access_source.contains(
        "let mut conflicts = Vec::with_capacity(component_conflict_upper_bound(self, other));"
    ));
    assert!(query_access_source
        .contains("fn component_conflict_upper_bound(left: &QueryAccess, right: &QueryAccess)"));
    assert!(query_access_source.contains("left.writes.len().min(right.reads.len())"));
    assert!(query_access_source.contains("read_only_component_count(left).min(right.writes.len())"));
    assert!(query_access_source.contains("fn read_only_component_count(access: &QueryAccess)"));
    assert!(query_access_source.contains("insert_sorted_component_id(conflicts, left_value);"));
    assert_eq!(
        query_access_source
            .matches("push_sorted_component_intersections(&mut conflicts")
            .count(),
        1,
        "detailed QueryAccess diagnostics must not repeat a write/write pass already covered by write-implies-read"
    );
    assert!(query_access_source.contains("push_read_only_component_intersections("));
    let read_only_component_diagnostic = query_access_source
        .split("fn push_read_only_component_intersections(")
        .nth(1)
        .and_then(|text| text.split("fn read_component_is_written").next())
        .expect("read read-only component diagnostic helper");
    assert!(read_only_component_diagnostic
        .contains("while read_index < reads.len() && right_index < right.len()"));
    assert!(read_only_component_diagnostic.contains("let read_value = reads[read_index];"));
    assert!(read_only_component_diagnostic.contains("let right_value = right[right_index];"));
    assert!(read_only_component_diagnostic
        .contains("if read_component_is_written(read_value, writes, &mut write_index)"));
    assert!(read_only_component_diagnostic
        .contains("insert_sorted_component_id(conflicts, read_value);"));
    assert!(!read_only_component_diagnostic
        .contains("while let (Some(read_value), Some(right_value)) ="));
    assert!(query_access_source.contains("read_component_is_written("));
    let written_component_probe = query_access_source
        .split("fn read_component_is_written(")
        .nth(1)
        .and_then(|text| text.split("fn insert_id").next())
        .expect("read written component probe helper");
    assert!(written_component_probe.contains("while *write_index < writes.len()"));
    assert!(written_component_probe.contains("let write_value = writes[*write_index];"));
    assert!(written_component_probe.contains("if write_value < component_id"));
    assert!(written_component_probe.contains("if write_value == component_id"));
    assert!(
        !written_component_probe.contains("while let Some(write_value) = writes.get(*write_index)")
    );
    assert!(!query_access_source.contains(
        "push_sorted_component_intersections(&mut conflicts, &self.reads, &other.writes)"
    ));
    assert!(!query_access_source.contains(
        "push_sorted_component_intersections(&mut conflicts, &self.writes, &other.writes)"
    ));
    assert!(!query_access_source.contains("fn push_intersections("));
    assert!(!query_access_source.contains("let mut conflicts = Vec::new();"));
    assert!(
        !sorted_component_intersect.contains("while let (Some(left_value), Some(right_value)) =")
    );

    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].stage(), SystemStage::Update);
    assert_eq!(
        batches[0]
            .system_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["read.health", "read.counter"]
    );
    assert_eq!(
        batches[1]
            .system_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["write.health"]
    );
}

#[test]
fn schedule_conflict_graph_keeps_parallel_batches_inside_stage_boundaries() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "update.a",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "post-update.b",
            SystemStage::PostUpdate,
            SystemParamAccess::default(),
        ),
    ]);

    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].stage(), SystemStage::Update);
    assert_eq!(batches[1].stage(), SystemStage::PostUpdate);
}
