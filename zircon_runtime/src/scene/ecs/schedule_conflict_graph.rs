use super::{SystemParamAccess, SystemParamConflictKind, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleConflictGraph {
    nodes: Vec<ScheduleConflictNode>,
    edges: Vec<ScheduleConflictEdge>,
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

        for left_index in 0..nodes.len() {
            for right_index in (left_index + 1)..nodes.len() {
                let left = &nodes[left_index];
                let right = &nodes[right_index];
                if left.stage != right.stage {
                    continue;
                }
                if left.is_barrier() || right.is_barrier() {
                    continue;
                }

                let conflicts = left.access.conflict_kinds_with(&right.access);
                if conflicts.is_empty() {
                    continue;
                }

                edges.push(ScheduleConflictEdge {
                    left_system_id: left.system_id.clone(),
                    right_system_id: right.system_id.clone(),
                    stage: left.stage,
                    conflicts,
                });
            }
        }

        Self { nodes, edges }
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
        self.edges.iter().any(|edge| {
            (edge.left_system_id.as_str() == left_system_id
                && edge.right_system_id.as_str() == right_system_id)
                || (edge.left_system_id.as_str() == right_system_id
                    && edge.right_system_id.as_str() == left_system_id)
        })
    }

    pub fn conflicts_for<'graph>(
        &'graph self,
        system_id: &'graph str,
    ) -> impl Iterator<Item = &'graph ScheduleConflictEdge> + 'graph {
        self.edges.iter().filter(move |edge| {
            edge.left_system_id.as_str() == system_id || edge.right_system_id.as_str() == system_id
        })
    }

    pub fn conservative_parallel_batches(&self) -> Vec<ScheduleParallelBatch> {
        let mut batches = Vec::<ScheduleParallelBatch>::new();

        for node in &self.nodes {
            if node.is_barrier() {
                // Barriers are ordering boundaries, not data-access systems. They occupy
                // their own batch so future parallel runners never overlap sync work with
                // producer or consumer systems.
                batches.push(ScheduleParallelBatch {
                    stage: node.stage,
                    system_ids: vec![node.system_id.clone()],
                    has_barrier: true,
                });
                continue;
            }

            if batches.last().is_some_and(|batch| {
                batch.stage == node.stage
                    && !batch.has_barrier
                    && batch
                        .system_ids
                        .iter()
                        .all(|system_id| !self.systems_conflict(system_id, node.system_id()))
            }) {
                batches
                    .last_mut()
                    .expect("last batch must exist after is_some_and")
                    .system_ids
                    .push(node.system_id.clone());
            } else {
                batches.push(ScheduleParallelBatch {
                    stage: node.stage,
                    system_ids: vec![node.system_id.clone()],
                    has_barrier: false,
                });
            }
        }

        batches
    }
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
