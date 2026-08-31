use std::collections::HashMap;
use std::sync::Arc;

use super::{SystemParamAccess, SystemParamConflictKind, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleConflictGraph {
    nodes: Vec<ScheduleConflictNode>,
    edges: Vec<ScheduleConflictEdge>,
    node_indices_by_system_id: HashMap<String, usize>,
    conflict_edge_indices_by_node: Vec<Vec<usize>>,
    conflict_node_adjacency: Vec<Vec<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleConflictNode {
    system_id: String,
    stage: SystemStage,
    access: SystemParamAccess,
    kind: ScheduleConflictNodeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleConflictEdge {
    left_system_id: String,
    right_system_id: String,
    stage: SystemStage,
    conflicts: Vec<SystemParamConflictKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleParallelBatch {
    stage: SystemStage,
    systems: Arc<ScheduleParallelBatchSystems>,
    has_barrier: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ScheduleParallelBatchSystems {
    Single {
        system_id: String,
        node_index: usize,
    },
    Pair {
        system_ids: [String; 2],
        node_indices: [usize; 2],
    },
    Triple {
        system_ids: [String; 3],
        node_indices: [usize; 3],
    },
    Multiple {
        system_ids: Vec<String>,
        node_indices: Vec<usize>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleConflictNodeKind {
    System,
    Barrier,
}

impl ScheduleConflictGraph {
    pub fn from_nodes(nodes: impl IntoIterator<Item = ScheduleConflictNode>) -> Self {
        let node_iter = nodes.into_iter();
        let (lower_bound, _) = node_iter.size_hint();
        let mut nodes = Vec::with_capacity(lower_bound);
        for node in node_iter {
            nodes.push(node);
        }
        Self::from_node_vec(nodes)
    }

    pub(crate) fn from_node_vec(nodes: Vec<ScheduleConflictNode>) -> Self {
        let graph_inputs = schedule_conflict_graph_inputs(&nodes);
        if nodes.len() <= 1 {
            let conflict_edge_indices_by_node = empty_node_index_lists(nodes.len());
            let conflict_node_adjacency = empty_node_index_lists(nodes.len());
            return Self {
                nodes,
                edges: Vec::new(),
                node_indices_by_system_id: graph_inputs.node_indices_by_system_id,
                conflict_edge_indices_by_node,
                conflict_node_adjacency,
            };
        }

        let mut conflict_edge_indices_by_node = empty_node_index_lists(nodes.len());
        let mut conflict_node_adjacency = empty_node_index_lists(nodes.len());
        let node_indices_by_stage = node_indices_by_stage(&nodes, &graph_inputs);
        let edge_upper_bound = same_stage_non_barrier_conflict_pair_upper_bound(&graph_inputs);
        let mut edges = Vec::with_capacity(edge_upper_bound);
        let mut next_stage_positions = [0_usize; SystemStage::COUNT];

        for left_index in 0..nodes.len() {
            let left = &nodes[left_index];
            let stage_index = left.stage.rank();
            let same_stage_node_indices = &node_indices_by_stage[stage_index];
            let left_stage_index = next_stage_positions[stage_index];
            next_stage_positions[stage_index] += 1;
            if left.is_barrier() {
                continue;
            }

            for right_index in &same_stage_node_indices[(left_stage_index + 1)..] {
                let right_index = *right_index;
                let right = &nodes[right_index];
                if right.is_barrier() {
                    continue;
                }

                if !left.access.conflicts_with(&right.access) {
                    continue;
                }

                let conflicts = left.access.conflict_kinds_with(&right.access);
                let edge_index = edges.len();
                record_conflict_pair(
                    &mut conflict_edge_indices_by_node,
                    &mut conflict_node_adjacency,
                    edge_index,
                    left_index,
                    right_index,
                );
                edges.push(ScheduleConflictEdge {
                    left_system_id: left.system_id.clone(),
                    right_system_id: right.system_id.clone(),
                    stage: left.stage,
                    conflicts,
                });
            }
        }

        Self {
            nodes,
            edges,
            node_indices_by_system_id: graph_inputs.node_indices_by_system_id,
            conflict_edge_indices_by_node,
            conflict_node_adjacency,
        }
    }

    pub fn nodes(&self) -> &[ScheduleConflictNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[ScheduleConflictEdge] {
        &self.edges
    }

    pub fn has_conflicts(&self) -> bool {
        !self.edges.is_empty()
    }

    pub fn systems_conflict(&self, left_system_id: &str, right_system_id: &str) -> bool {
        let Some(left_index) = self.node_indices_by_system_id.get(left_system_id) else {
            return false;
        };
        let Some(right_index) = self.node_indices_by_system_id.get(right_system_id) else {
            return false;
        };

        self.node_indices_conflict(*left_index, *right_index)
    }

    pub fn conflicts_for<'graph>(
        &'graph self,
        system_id: &'graph str,
    ) -> impl Iterator<Item = &'graph ScheduleConflictEdge> + 'graph {
        let edge_indices = match self.node_indices_by_system_id.get(system_id) {
            Some(node_index) => self.conflict_edge_indices_by_node[*node_index].as_slice(),
            None => &[],
        };
        ScheduleConflictEdges::new(self, edge_indices)
    }

    pub fn conservative_parallel_batches(&self) -> Vec<ScheduleParallelBatch> {
        let mut batches = Vec::<ScheduleParallelBatch>::with_capacity(self.nodes.len());

        for (node_index, node) in self.nodes.iter().enumerate() {
            if node.is_barrier() {
                // Barriers are ordering boundaries, not data-access systems. They occupy
                // their own batch so future parallel runners never overlap sync work with
                // producer or consumer systems.
                batches.push(ScheduleParallelBatch::single(
                    node.stage,
                    node.system_id.clone(),
                    node_index,
                    true,
                ));
                continue;
            }

            if let Some(batch) = batches.last_mut() {
                let can_extend_last_batch = batch.stage == node.stage
                    && !batch.has_barrier
                    && !self.node_indices_conflict_with_any(batch.node_indices(), node_index);
                if can_extend_last_batch {
                    batch.push_system(node.system_id.clone(), node_index);
                    continue;
                }
            }

            batches.push(ScheduleParallelBatch::single(
                node.stage,
                node.system_id.clone(),
                node_index,
                false,
            ));
        }

        batches
    }

    fn node_indices_conflict(&self, left_index: usize, right_index: usize) -> bool {
        let Some(neighbors) = self.conflict_node_adjacency.get(left_index) else {
            return false;
        };

        neighbors.binary_search(&right_index).is_ok()
    }

    fn node_indices_conflict_with_any(&self, node_indices: &[usize], right_index: usize) -> bool {
        let Some(neighbors) = self.conflict_node_adjacency.get(right_index) else {
            return false;
        };

        sorted_slices_intersect(node_indices, neighbors)
    }
}

struct ScheduleConflictEdges<'graph> {
    graph: &'graph ScheduleConflictGraph,
    edge_indices: &'graph [usize],
    next_index: usize,
}

impl<'graph> ScheduleConflictEdges<'graph> {
    fn new(graph: &'graph ScheduleConflictGraph, edge_indices: &'graph [usize]) -> Self {
        Self {
            graph,
            edge_indices,
            next_index: 0,
        }
    }
}

impl<'graph> Iterator for ScheduleConflictEdges<'graph> {
    type Item = &'graph ScheduleConflictEdge;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.edge_indices.len() {
            return None;
        }
        let edge_index = self.edge_indices[self.next_index];
        self.next_index += 1;
        Some(&self.graph.edges[edge_index])
    }
}

struct ScheduleConflictGraphInputs {
    node_indices_by_system_id: HashMap<String, usize>,
    node_counts_by_stage: [usize; SystemStage::COUNT],
    non_barrier_node_counts_by_stage: [usize; SystemStage::COUNT],
}

fn schedule_conflict_graph_inputs(nodes: &[ScheduleConflictNode]) -> ScheduleConflictGraphInputs {
    let mut node_indices_by_system_id = HashMap::with_capacity(nodes.len());
    let mut node_counts_by_stage = [0_usize; SystemStage::COUNT];
    let mut non_barrier_node_counts_by_stage = [0_usize; SystemStage::COUNT];
    for (index, node) in nodes.iter().enumerate() {
        node_indices_by_system_id.insert(node.system_id.clone(), index);
        let stage_index = node.stage.rank();
        node_counts_by_stage[stage_index] += 1;
        if !node.is_barrier() {
            non_barrier_node_counts_by_stage[stage_index] += 1;
        }
    }
    ScheduleConflictGraphInputs {
        node_indices_by_system_id,
        node_counts_by_stage,
        non_barrier_node_counts_by_stage,
    }
}

fn empty_node_index_lists(node_count: usize) -> Vec<Vec<usize>> {
    let mut lists = Vec::with_capacity(node_count);
    lists.resize_with(node_count, Vec::new);
    lists
}

fn node_indices_by_stage(
    nodes: &[ScheduleConflictNode],
    graph_inputs: &ScheduleConflictGraphInputs,
) -> [Vec<usize>; SystemStage::COUNT] {
    let mut node_indices_by_stage = std::array::from_fn(|stage_index| {
        Vec::with_capacity(graph_inputs.node_counts_by_stage[stage_index])
    });
    for (index, node) in nodes.iter().enumerate() {
        node_indices_by_stage[node.stage.rank()].push(index);
    }
    node_indices_by_stage
}

fn same_stage_non_barrier_conflict_pair_upper_bound(
    graph_inputs: &ScheduleConflictGraphInputs,
) -> usize {
    let mut edge_upper_bound = 0;
    for node_count in graph_inputs.non_barrier_node_counts_by_stage {
        edge_upper_bound += conflict_pair_count(node_count);
    }
    edge_upper_bound
}

fn conflict_pair_count(node_count: usize) -> usize {
    node_count.saturating_mul(node_count.saturating_sub(1)) / 2
}

fn record_conflict_pair(
    conflict_edge_indices_by_node: &mut [Vec<usize>],
    conflict_node_adjacency: &mut [Vec<usize>],
    edge_index: usize,
    left_index: usize,
    right_index: usize,
) {
    conflict_edge_indices_by_node[left_index].push(edge_index);
    conflict_edge_indices_by_node[right_index].push(edge_index);
    // Pair discovery walks left and right node indices in ascending order, so each
    // adjacency list remains sorted for binary_search without per-node HashSets.
    conflict_node_adjacency[left_index].push(right_index);
    conflict_node_adjacency[right_index].push(left_index);
}

fn sorted_slices_intersect(left: &[usize], right: &[usize]) -> bool {
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        let left_value = left[left_index];
        let right_value = right[right_index];
        if left_value == right_value {
            return true;
        }
        if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    false
}

impl ScheduleConflictNode {
    pub fn new(
        system_id: impl Into<String>,
        stage: SystemStage,
        access: SystemParamAccess,
    ) -> Self {
        Self {
            system_id: system_id.into(),
            stage,
            access,
            kind: ScheduleConflictNodeKind::System,
        }
    }

    pub fn barrier(system_id: impl Into<String>, stage: SystemStage) -> Self {
        Self {
            system_id: system_id.into(),
            stage,
            access: SystemParamAccess::default(),
            kind: ScheduleConflictNodeKind::Barrier,
        }
    }

    pub fn system_id(&self) -> &str {
        &self.system_id
    }

    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    pub fn kind(&self) -> ScheduleConflictNodeKind {
        self.kind
    }

    pub fn is_barrier(&self) -> bool {
        self.kind == ScheduleConflictNodeKind::Barrier
    }
}

impl ScheduleConflictEdge {
    pub fn left_system_id(&self) -> &str {
        &self.left_system_id
    }

    pub fn right_system_id(&self) -> &str {
        &self.right_system_id
    }

    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn conflicts(&self) -> &[SystemParamConflictKind] {
        &self.conflicts
    }
}

impl ScheduleParallelBatch {
    fn single(stage: SystemStage, system_id: String, node_index: usize, has_barrier: bool) -> Self {
        Self {
            stage,
            systems: Arc::new(ScheduleParallelBatchSystems::Single {
                system_id,
                node_index,
            }),
            has_barrier,
        }
    }

    fn push_system(&mut self, system_id: String, node_index: usize) {
        let systems = Arc::make_mut(&mut self.systems);
        let promoted = match systems {
            ScheduleParallelBatchSystems::Single {
                system_id: first_system_id,
                node_index: first_node_index,
            } => Some(ScheduleParallelBatchSystems::Pair {
                system_ids: [std::mem::take(first_system_id), system_id],
                node_indices: [*first_node_index, node_index],
            }),
            ScheduleParallelBatchSystems::Pair {
                system_ids,
                node_indices,
            } => Some(ScheduleParallelBatchSystems::Triple {
                system_ids: [
                    std::mem::take(&mut system_ids[0]),
                    std::mem::take(&mut system_ids[1]),
                    system_id,
                ],
                node_indices: [node_indices[0], node_indices[1], node_index],
            }),
            ScheduleParallelBatchSystems::Triple {
                system_ids,
                node_indices,
            } => {
                let mut promoted_system_ids = Vec::with_capacity(4);
                promoted_system_ids.push(std::mem::take(&mut system_ids[0]));
                promoted_system_ids.push(std::mem::take(&mut system_ids[1]));
                promoted_system_ids.push(std::mem::take(&mut system_ids[2]));
                promoted_system_ids.push(system_id);

                let mut promoted_node_indices = Vec::with_capacity(4);
                promoted_node_indices.extend_from_slice(node_indices);
                promoted_node_indices.push(node_index);

                Some(ScheduleParallelBatchSystems::Multiple {
                    system_ids: promoted_system_ids,
                    node_indices: promoted_node_indices,
                })
            }
            ScheduleParallelBatchSystems::Multiple {
                system_ids,
                node_indices,
            } => {
                system_ids.push(system_id);
                node_indices.push(node_index);
                None
            }
        };

        if let Some(promoted) = promoted {
            *systems = promoted;
        }
    }

    fn node_indices(&self) -> &[usize] {
        self.systems.node_indices()
    }

    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn system_ids(&self) -> &[String] {
        self.systems.system_ids()
    }

    pub(super) fn shared_systems(&self) -> Arc<ScheduleParallelBatchSystems> {
        Arc::clone(&self.systems)
    }

    pub fn has_barrier(&self) -> bool {
        self.has_barrier
    }
}

impl ScheduleParallelBatchSystems {
    fn node_indices(&self) -> &[usize] {
        match self {
            ScheduleParallelBatchSystems::Single { node_index, .. } => {
                std::slice::from_ref(node_index)
            }
            ScheduleParallelBatchSystems::Pair { node_indices, .. } => node_indices.as_slice(),
            ScheduleParallelBatchSystems::Triple { node_indices, .. } => node_indices.as_slice(),
            ScheduleParallelBatchSystems::Multiple { node_indices, .. } => node_indices,
        }
    }

    pub(super) fn system_ids(&self) -> &[String] {
        match self {
            ScheduleParallelBatchSystems::Single { system_id, .. } => {
                std::slice::from_ref(system_id)
            }
            ScheduleParallelBatchSystems::Pair { system_ids, .. } => system_ids.as_slice(),
            ScheduleParallelBatchSystems::Triple { system_ids, .. } => system_ids.as_slice(),
            ScheduleParallelBatchSystems::Multiple { system_ids, .. } => system_ids,
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const PERF_SAMPLE_PAIRS: usize = 21;
    const PERF_ITERATIONS_PER_SAMPLE: usize = 5_000;
    const PERF_SYSTEM_COUNT: usize = 64;

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn shared_schedule_batch_systems_avoid_frame_heap_allocations() {
        let mut batch =
            ScheduleParallelBatch::single(SystemStage::Update, "system.000".to_string(), 0, false);
        for index in 1..PERF_SYSTEM_COUNT {
            batch.push_system(format!("system.{index:03}"), index);
        }

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair_index in 0..PERF_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&batch));
                optimized_samples.push(measure_optimized(&batch));
            } else {
                optimized_samples.push(measure_optimized(&batch));
                legacy_samples.push(measure_legacy(&batch));
            }
        }

        let legacy_p50 = percentile_ns(&mut legacy_samples, 50);
        let legacy_p95 = percentile_ns(&mut legacy_samples, 95);
        let optimized_p50 = percentile_ns(&mut optimized_samples, 50);
        let optimized_p95 = percentile_ns(&mut optimized_samples, 95);
        println!(
            "PERF_RESULT runtime60_shared_schedule_batch_systems legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} systems_per_batch={PERF_SYSTEM_COUNT} iterations_per_sample={PERF_ITERATIONS_PER_SAMPLE} samples={PERF_SAMPLE_PAIRS} legacy_heap_allocations_per_batch=65 optimized_heap_allocations_per_batch=0"
        );

        assert!(
            optimized_p95 <= legacy_p95 / 2,
            "shared batch storage should cut P95 execution-frame setup by at least 50%: legacy={legacy_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn measure_legacy(batch: &ScheduleParallelBatch) -> Duration {
        let started = Instant::now();
        for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
            let system_ids = black_box(batch.system_ids()).to_vec();
            black_box(system_ids);
        }
        started.elapsed()
    }

    fn measure_optimized(batch: &ScheduleParallelBatch) -> Duration {
        let started = Instant::now();
        for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
            let systems = black_box(batch).shared_systems();
            black_box(systems.system_ids());
        }
        started.elapsed()
    }

    fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).div_ceil(100);
        samples[rank.saturating_sub(1)].as_nanos()
    }
}
