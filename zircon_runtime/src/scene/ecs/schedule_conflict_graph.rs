use std::collections::HashMap;

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
    system_ids: Vec<String>,
    node_indices: Vec<usize>,
    has_barrier: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleConflictNodeKind {
    System,
    Barrier,
}

impl ScheduleConflictGraph {
    pub fn from_nodes(nodes: impl IntoIterator<Item = ScheduleConflictNode>) -> Self {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        let mut edges = Vec::new();
        let node_indices_by_system_id = node_indices_by_system_id(&nodes);
        let mut conflict_edge_indices_by_node = vec![Vec::<usize>::new(); nodes.len()];
        let mut conflict_node_adjacency = vec![Vec::<usize>::new(); nodes.len()];
        let node_indices_by_stage = node_indices_by_stage(&nodes);
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
            node_indices_by_system_id,
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
        self.node_indices_by_system_id
            .get(left_system_id)
            .zip(self.node_indices_by_system_id.get(right_system_id))
            .is_some_and(|(left_index, right_index)| {
                self.node_indices_conflict(*left_index, *right_index)
            })
    }

    pub fn conflicts_for<'graph>(
        &'graph self,
        system_id: &'graph str,
    ) -> impl Iterator<Item = &'graph ScheduleConflictEdge> + 'graph {
        let edges = &self.edges;
        self.node_indices_by_system_id
            .get(system_id)
            .into_iter()
            .flat_map(move |node_index| {
                self.conflict_edge_indices_by_node[*node_index]
                    .iter()
                    .map(move |index| &edges[*index])
            })
    }

    pub fn conservative_parallel_batches(&self) -> Vec<ScheduleParallelBatch> {
        let mut batches = Vec::<ScheduleParallelBatch>::with_capacity(self.nodes.len());

        for (node_index, node) in self.nodes.iter().enumerate() {
            if node.is_barrier() {
                // Barriers are ordering boundaries, not data-access systems. They occupy
                // their own batch so future parallel runners never overlap sync work with
                // producer or consumer systems.
                batches.push(ScheduleParallelBatch {
                    stage: node.stage,
                    system_ids: vec![node.system_id.clone()],
                    node_indices: vec![node_index],
                    has_barrier: true,
                });
                continue;
            }

            if batches.last().is_some_and(|batch| {
                batch.stage == node.stage
                    && !batch.has_barrier
                    && !self.node_indices_conflict_with_any(&batch.node_indices, node_index)
            }) {
                let batch = batches
                    .last_mut()
                    .expect("last batch must exist after is_some_and");
                batch.system_ids.push(node.system_id.clone());
                batch.node_indices.push(node_index);
            } else {
                batches.push(ScheduleParallelBatch {
                    stage: node.stage,
                    system_ids: vec![node.system_id.clone()],
                    node_indices: vec![node_index],
                    has_barrier: false,
                });
            }
        }

        batches
    }

    fn node_indices_conflict(&self, left_index: usize, right_index: usize) -> bool {
        self.conflict_node_adjacency
            .get(left_index)
            .is_some_and(|neighbors| neighbors.binary_search(&right_index).is_ok())
    }

    fn node_indices_conflict_with_any(&self, node_indices: &[usize], right_index: usize) -> bool {
        self.conflict_node_adjacency
            .get(right_index)
            .is_some_and(|neighbors| sorted_slices_intersect(node_indices, neighbors))
    }
}

fn node_indices_by_system_id(nodes: &[ScheduleConflictNode]) -> HashMap<String, usize> {
    let mut node_indices_by_system_id = HashMap::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        node_indices_by_system_id.insert(node.system_id.clone(), index);
    }
    node_indices_by_system_id
}

fn node_indices_by_stage(nodes: &[ScheduleConflictNode]) -> [Vec<usize>; SystemStage::COUNT] {
    let mut node_counts_by_stage = [0_usize; SystemStage::COUNT];
    for node in nodes {
        node_counts_by_stage[node.stage.rank()] += 1;
    }

    let mut node_indices_by_stage =
        std::array::from_fn(|stage_index| Vec::with_capacity(node_counts_by_stage[stage_index]));
    for (index, node) in nodes.iter().enumerate() {
        node_indices_by_stage[node.stage.rank()].push(index);
    }
    node_indices_by_stage
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
    while let (Some(left_value), Some(right_value)) = (left.get(left_index), right.get(right_index))
    {
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
    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn system_ids(&self) -> &[String] {
        &self.system_ids
    }

    pub fn has_barrier(&self) -> bool {
        self.has_barrier
    }
}
