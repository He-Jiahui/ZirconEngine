use std::any::TypeId;
use std::sync::{Arc, Mutex};

use crate::core::JobScheduler;
use crate::scene::ecs::{
    Component, QueryState, ResMutParam, ResParam, Resource, ScheduleConflictGraph,
    ScheduleConflictNode, ScheduleParallelExecutor, ScheduleParallelExecutorError,
    ScheduleParallelTaskRegistry, SystemParamAccess, SystemParamConflictKind, SystemStage,
    SystemState, With, Without,
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
    assert!(
        graph_source.contains("let node_indices_by_system_id = node_indices_by_system_id(&nodes);")
    );
    assert!(graph_source.contains("let node_indices_by_stage = node_indices_by_stage(&nodes);"));
    assert!(graph_source.contains("let mut next_stage_positions = [0_usize; SystemStage::COUNT];"));
    assert!(graph_source.contains(
        "let mut batches = Vec::<ScheduleParallelBatch>::with_capacity(self.nodes.len())"
    ));
    assert!(graph_source.contains("for left_index in 0..nodes.len()"));
    assert!(
        graph_source.contains("let same_stage_node_indices = &node_indices_by_stage[stage_index];")
    );
    assert!(graph_source.contains("let left_stage_index = next_stage_positions[stage_index];"));
    assert!(graph_source.contains("next_stage_positions[stage_index] += 1;"));
    assert!(graph_source
        .contains("for right_index in &same_stage_node_indices[(left_stage_index + 1)..]"));
    assert!(graph_source.contains(
        "let mut conflict_edge_indices_by_node = vec![Vec::<usize>::new(); nodes.len()]"
    ));
    assert!(graph_source.contains("&mut conflict_edge_indices_by_node"));
    assert!(graph_source.contains("&mut conflict_node_adjacency"));
    assert!(graph_source.contains("conflict_edge_indices_by_node: &mut [Vec<usize>]"));
    assert!(graph_source.contains("conflict_node_adjacency: &mut [Vec<usize>]"));
    assert!(graph_source.contains("&edges[*index]"));
    assert!(graph_source.contains("self.node_indices_by_system_id"));
    assert!(graph_source.contains("self.conflict_edge_indices_by_node[*node_index]"));
    assert!(graph_source.contains("self.conflict_node_adjacency"));
    assert!(graph_source.contains(".zip(self.node_indices_by_system_id.get(right_system_id))"));
    assert!(graph_source.contains("self.node_indices_conflict(*left_index, *right_index)"));
    assert!(graph_source.contains(
        "fn node_indices_by_system_id(nodes: &[ScheduleConflictNode]) -> HashMap<String, usize>"
    ));
    assert!(graph_source
        .contains("let mut node_indices_by_system_id = HashMap::with_capacity(nodes.len())"));
    assert!(
        graph_source.contains("node_indices_by_system_id.insert(node.system_id.clone(), index);")
    );
    assert!(graph_source
        .contains("fn node_indices_by_stage(nodes: &[ScheduleConflictNode]) -> [Vec<usize>; SystemStage::COUNT]"));
    assert!(graph_source.contains("let mut node_counts_by_stage = [0_usize; SystemStage::COUNT];"));
    assert!(graph_source.contains("node_counts_by_stage[node.stage.rank()] += 1;"));
    assert!(graph_source.contains("Vec::with_capacity(node_counts_by_stage[stage_index])"));
    assert!(graph_source.contains("node_indices_by_stage[node.stage.rank()].push(index);"));
    assert!(graph_source.contains("node_indices: Vec<usize>"));
    assert!(graph_source.contains(".node_indices"));
    assert!(graph_source.contains("node_indices_conflict("));
    assert!(graph_source.contains("node_indices_conflict_with_any("));
    assert!(graph_source.contains("neighbors.binary_search(&right_index).is_ok()"));
    assert!(graph_source.contains("sorted_slices_intersect(node_indices, neighbors)"));
    assert!(graph_source
        .contains("fn sorted_slices_intersect(left: &[usize], right: &[usize]) -> bool"));
    assert!(graph_source.contains("while let (Some(left_value), Some(right_value)) ="));
    assert!(graph_source.contains("conflict_edge_indices_by_node[left_index].push(edge_index);"));
    assert!(graph_source.contains("conflict_edge_indices_by_node[right_index].push(edge_index);"));
    assert!(graph_source.contains("conflict_node_adjacency[left_index].push(right_index);"));
    assert!(graph_source.contains("conflict_node_adjacency[right_index].push(left_index);"));
    assert!(graph_source.contains("if !left.access.conflicts_with(&right.access)"));
    assert!(!graph_source.contains("use std::collections::{HashMap, HashSet};"));
    assert!(!graph_source.contains("conflict_adjacency"));
    assert!(!graph_source.contains("conflict_edge_indices: HashMap<String, Vec<usize>>"));
    assert!(!graph_source.contains("HashMap::<String, Vec<usize>>::new()"));
    assert!(!graph_source.contains("HashSet<String>"));
    assert!(!graph_source.contains("let mut batches = Vec::<ScheduleParallelBatch>::new();"));
    assert!(!graph_source.contains("fn node_indices_by_system_id(nodes: &[ScheduleConflictNode]) -> HashMap<String, usize> {\n    nodes"));
    assert!(!graph_source.contains(".entry(left.system_id.clone())"));
    assert!(!graph_source.contains(".entry(right.system_id.clone())"));
    assert!(!graph_source.contains("for right_index in (left_index + 1)..nodes.len()"));
    assert!(!graph_source.contains("if left.stage != right.stage"));
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
    assert!(!access_source.contains("!self.conflict_kinds_with(other).is_empty()"));
    assert!(!access_source.contains("query_access.writes().contains(component_id)"));
    assert!(!access_source.contains("ids.contains(&type_id)"));
    assert!(!access_source.contains("ids.sort_unstable();"));

    let query_access_source = include_str!("../../ecs/query/query_access.rs");
    assert!(query_access_source.contains(
        "fn sorted_component_slices_intersect(left: &[ComponentId], right: &[ComponentId]) -> bool"
    ));
    assert!(query_access_source.contains("while let (Some(left_value), Some(right_value)) ="));
    assert!(query_access_source.contains("if left_value == right_value"));
    assert!(query_access_source.contains("left_index += 1;"));
    assert!(query_access_source.contains("right_index += 1;"));
    assert!(query_access_source
        .contains("sorted_component_slices_intersect(&self.writes, &other.reads)"));
    assert!(query_access_source
        .contains("sorted_component_slices_intersect(&self.reads, &other.writes)"));
    assert!(query_access_source
        .contains("sorted_component_slices_intersect(&self.writes, &other.writes)"));
    assert!(!query_access_source
        .contains("left.iter()\n        .any(|component_id| contains_id(right, *component_id))"));
    assert!(query_access_source.contains(
        "fn push_sorted_component_intersections(\n    conflicts: &mut Vec<ComponentId>,"
    ));
    assert!(query_access_source.contains("insert_sorted_component_id(conflicts, *left_value);"));
    assert!(query_access_source.contains("push_sorted_component_intersections("));
    assert!(!query_access_source.contains("fn push_intersections("));

    let executor_source = include_str!("../../ecs/schedule_parallel_executor.rs");
    assert!(executor_source.contains("if let [system_id] = system_ids"));
    assert!(executor_source.contains("let task = registry.task_for_system(system_id)?;"));
    assert!(executor_source.contains("run_task(system_id, task)?;"));
    assert!(executor_source.contains("fn task_for_system<'registry>("));
    assert!(executor_source.contains("fn tasks_for_batch<'registry>("));
    assert!(executor_source.contains("system_ids: &'registry [String]"));
    assert!(executor_source.contains("Vec<&'registry (dyn Fn() -> Result<(), E> + Send + Sync)>"));
    assert!(executor_source.contains("use rayon::iter::{IntoParallelIterator, ParallelIterator};"));
    assert!(
        executor_source.contains("tasks.into_par_iter().map(|task| task()).collect::<Vec<_>>()")
    );
    assert!(executor_source.contains("for (index, result) in results.into_iter().enumerate()"));
    assert!(
        executor_source.contains(".expect(\"task result index must originate from batch order\")")
    );
    assert!(!executor_source.contains("if tasks.len() == 1"));
    assert!(!executor_source.contains("tasks[0]."));
    assert!(!executor_source.contains("use std::sync::Mutex;"));
    assert!(!executor_source.contains("rayon::scope"));
    assert!(!executor_source.contains("empty_task_results"));
    assert!(!executor_source.contains("Vec::with_capacity(tasks.len())"));
    assert!(!executor_source.contains("Mutex::new("));
    assert!(!executor_source.contains("Some(result);"));
    assert!(!executor_source.contains(".push((index, task()))"));
    assert!(!executor_source.contains("results.sort_by_key"));
    assert!(
        !executor_source.contains("system_id.to_string(),\n                                task()")
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

#[test]
fn schedule_parallel_executor_runs_registered_batches_through_job_scheduler() {
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
    let batches = graph.conservative_parallel_batches();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    for system_id in ["read.health", "read.counter", "write.health"] {
        let observed = observed.clone();
        registry.register(system_id, move || {
            observed.lock().unwrap().push(system_id);
            Ok(())
        });
    }
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    executor.run_batches(&batches, &registry).unwrap();

    assert!(executor.scheduler().parallelism() >= 1);
    assert!(registry.contains("write.health"));
    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 3);
    let mut first_batch = observed[..2].to_vec();
    first_batch.sort_unstable();
    assert_eq!(first_batch, vec!["read.counter", "read.health"]);
    assert_eq!(observed[2], "write.health");
}

#[test]
fn schedule_parallel_executor_reports_missing_tasks_before_running_batch() {
    let graph = ScheduleConflictGraph::from_nodes([ScheduleConflictNode::new(
        "missing.task",
        SystemStage::Update,
        SystemParamAccess::default(),
    )]);
    let batches = graph.conservative_parallel_batches();
    let registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::MissingTask {
            system_id: "missing.task".to_string(),
        }
    );
}

#[test]
fn schedule_parallel_executor_reports_task_failure_by_batch_order() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("ok.task", SystemStage::Update, SystemParamAccess::default()),
        ScheduleConflictNode::new(
            "fail.task",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    registry.register("ok.task", || Ok(()));
    registry.register("fail.task", || Err("boom"));
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::TaskFailed {
            system_id: "fail.task".to_string(),
            error: "boom",
        }
    );
}
