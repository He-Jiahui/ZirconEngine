use std::cmp::Ordering;
use std::collections::HashMap;

use super::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, SceneSystemDescriptor, SceneSystemRegistry,
    SceneSystemThreadAffinity, ScheduleConflictGraph, ScheduleError, ScheduledSceneStep,
    SystemOrderingConstraint, SystemRef, SystemSetId, SystemStage,
};

/// One tick's schedule snapshot, grouped by stage to avoid repeated stage scans.
#[derive(Clone, Debug)]
pub(crate) struct SceneScheduleStagePlan {
    stages: Vec<SystemStage>,
    internal_systems_by_stage: [Vec<SceneSystemDescriptor>; SystemStage::COUNT],
    native_steps_by_stage: [Vec<ScheduledSceneStep>; SystemStage::COUNT],
    native_conflict_graphs_by_stage: [ScheduleConflictGraph; SystemStage::COUNT],
}

impl SceneScheduleStagePlan {
    pub(crate) fn from_registry(
        stages: &[SystemStage],
        registry: &SceneSystemRegistry,
    ) -> Result<Self, ScheduleError> {
        let systems = registry.systems();
        let native_systems = registry.native_systems();
        let runtime_systems = registry.runtime_systems();
        let mut stage_order = Vec::with_capacity(stages.len());
        for stage in stages.iter().copied() {
            stage_order.push(stage);
        }

        let internal_system_counts = internal_system_counts_by_stage(systems);
        let mut internal_systems_by_stage =
            internal_system_groups_with_capacity(&internal_system_counts);
        let native_step_counts = native_step_counts_by_stage(native_systems, runtime_systems);
        let mut native_steps_by_stage = native_step_groups_with_capacity(&native_step_counts);
        let native_conflict_graphs_by_stage =
            SystemStage::ORDER.map(|stage| registry.native_system_conflict_graph_for_stage(stage));
        let all_nodes = PlanNodes::new(systems, native_systems, runtime_systems);
        for stage in stages.iter().copied() {
            let stage_nodes = all_nodes.stage_nodes(stage);
            let sorted = topological_stage_order(stage, &stage_nodes, &all_nodes)?;
            for (plan_order, node_index) in sorted.into_iter().enumerate() {
                let plan_order = plan_order as i32;
                match stage_nodes[node_index] {
                    PlanNodeRef::Internal(system) => {
                        let mut system = system.clone();
                        system.order = plan_order;
                        internal_systems_by_stage[stage.rank()].push(system);
                    }
                    PlanNodeRef::Native(system) => {
                        let worker_safe = system.thread_affinity()
                            == SceneSystemThreadAffinity::WorkerSafe
                            && system.supports_worldless_execution()
                            && system.constraints().is_empty();
                        native_steps_by_stage[stage.rank()].push(ScheduledSceneStep::native(
                            system.id(),
                            system.stage(),
                            plan_order,
                            worker_safe,
                            system.access().has_conservative_world_access(),
                        ));
                        if system.has_deferred_commands() {
                            native_steps_by_stage[stage.rank()].push(
                                ScheduledSceneStep::apply_deferred_after(
                                    system.id(),
                                    system.stage(),
                                    plan_order,
                                ),
                            );
                        }
                    }
                    PlanNodeRef::Runtime(system) => {
                        native_steps_by_stage[stage.rank()].push(ScheduledSceneStep::runtime(
                            system.id(),
                            system.stage(),
                            plan_order,
                        ));
                    }
                }
            }
        }

        Ok(Self {
            stages: stage_order,
            internal_systems_by_stage,
            native_steps_by_stage,
            native_conflict_graphs_by_stage,
        })
    }

    pub(crate) fn stages(&self) -> &[SystemStage] {
        &self.stages
    }

    pub(crate) fn internal_systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> &[SceneSystemDescriptor] {
        &self.internal_systems_by_stage[stage.rank()]
    }

    pub(crate) fn native_steps_for_stage(&self, stage: SystemStage) -> &[ScheduledSceneStep] {
        &self.native_steps_by_stage[stage.rank()]
    }

    pub(crate) fn native_conflict_graph_for_stage(
        &self,
        stage: SystemStage,
    ) -> &ScheduleConflictGraph {
        &self.native_conflict_graphs_by_stage[stage.rank()]
    }
}

fn internal_system_counts_by_stage(
    systems: &[SceneSystemDescriptor],
) -> [usize; SystemStage::COUNT] {
    let mut counts = [0_usize; SystemStage::COUNT];
    for system in systems {
        counts[system.stage.rank()] += 1;
    }
    counts
}

fn internal_system_groups_with_capacity(
    internal_system_counts: &[usize; SystemStage::COUNT],
) -> [Vec<SceneSystemDescriptor>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(internal_system_counts[stage_index]))
}

fn native_step_counts_by_stage(
    systems: &[BoxedSceneSystem],
    runtime_systems: &[BoxedRuntimeSceneSystem],
) -> [usize; SystemStage::COUNT] {
    let mut counts = [0_usize; SystemStage::COUNT];
    for system in systems {
        let step_count = if system.has_deferred_commands() { 2 } else { 1 };
        counts[system.stage().rank()] += step_count;
    }
    for system in runtime_systems {
        counts[system.stage().rank()] += 1;
    }
    counts
}

fn native_step_groups_with_capacity(
    native_step_counts: &[usize; SystemStage::COUNT],
) -> [Vec<ScheduledSceneStep>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(native_step_counts[stage_index]))
}

#[derive(Clone, Copy)]
enum PlanNodeRef<'a> {
    Internal(&'a SceneSystemDescriptor),
    Native(&'a BoxedSceneSystem),
    Runtime(&'a BoxedRuntimeSceneSystem),
}

impl<'a> PlanNodeRef<'a> {
    fn id(self) -> &'a str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Native(system) => system.id(),
            Self::Runtime(system) => system.id(),
        }
    }

    fn stage(self) -> SystemStage {
        match self {
            Self::Internal(system) => system.stage,
            Self::Native(system) => system.stage(),
            Self::Runtime(system) => system.stage(),
        }
    }

    fn order(self) -> i32 {
        match self {
            Self::Internal(system) => system.order,
            Self::Native(system) => system.order(),
            Self::Runtime(system) => system.order(),
        }
    }

    fn sets(self) -> &'a [SystemSetId] {
        match self {
            Self::Internal(system) => &system.sets,
            Self::Native(system) => system.sets(),
            Self::Runtime(system) => system.sets(),
        }
    }

    fn constraints(self) -> &'a [SystemOrderingConstraint] {
        match self {
            Self::Internal(system) => &system.constraints,
            Self::Native(system) => system.constraints(),
            Self::Runtime(system) => system.constraints(),
        }
    }
}

struct PlanNodes<'a> {
    systems: &'a [SceneSystemDescriptor],
    native_systems: &'a [BoxedSceneSystem],
    runtime_systems: &'a [BoxedRuntimeSceneSystem],
    stages_by_id: HashMap<&'a str, SystemStage>,
}

impl<'a> PlanNodes<'a> {
    fn new(
        systems: &'a [SceneSystemDescriptor],
        native_systems: &'a [BoxedSceneSystem],
        runtime_systems: &'a [BoxedRuntimeSceneSystem],
    ) -> Self {
        let mut stages_by_id =
            HashMap::with_capacity(systems.len() + native_systems.len() + runtime_systems.len());
        for system in systems {
            stages_by_id.insert(system.id.as_str(), system.stage);
        }
        for system in native_systems {
            stages_by_id.insert(system.id(), system.stage());
        }
        for system in runtime_systems {
            stages_by_id.insert(system.id(), system.stage());
        }
        Self {
            systems,
            native_systems,
            runtime_systems,
            stages_by_id,
        }
    }

    fn stage_nodes(&self, stage: SystemStage) -> Vec<PlanNodeRef<'a>> {
        let mut nodes = Vec::new();
        for system in self.systems {
            if system.stage == stage {
                nodes.push(PlanNodeRef::Internal(system));
            }
        }
        for system in self.native_systems {
            if system.stage() == stage {
                nodes.push(PlanNodeRef::Native(system));
            }
        }
        for system in self.runtime_systems {
            if system.stage() == stage {
                nodes.push(PlanNodeRef::Runtime(system));
            }
        }
        nodes
    }
}

fn topological_stage_order(
    stage: SystemStage,
    nodes: &[PlanNodeRef<'_>],
    all_nodes: &PlanNodes<'_>,
) -> Result<Vec<usize>, ScheduleError> {
    let mut same_stage_by_id = HashMap::with_capacity(nodes.len());
    for (index, node) in nodes.iter().copied().enumerate() {
        same_stage_by_id.insert(node.id(), index);
    }

    let mut outgoing_edges = vec![Vec::<usize>::new(); nodes.len()];
    let mut incoming_counts = vec![0_usize; nodes.len()];
    for (index, node) in nodes.iter().copied().enumerate() {
        for constraint in node.constraints() {
            for (from, to) in constraint_edges(index, node, constraint, nodes, all_nodes, stage)? {
                if from == to || outgoing_edges[from].contains(&to) {
                    continue;
                }
                outgoing_edges[from].push(to);
                incoming_counts[to] += 1;
            }
        }
    }

    let mut scheduled = vec![false; nodes.len()];
    let mut order = Vec::with_capacity(nodes.len());
    for _ in 0..nodes.len() {
        let Some(next) = next_ready_node(nodes, &incoming_counts, &scheduled) else {
            return Err(ScheduleError::OrderingCycle {
                stage,
                chain: cycle_chain(nodes, &scheduled),
            });
        };
        scheduled[next] = true;
        order.push(next);
        for target in &outgoing_edges[next] {
            incoming_counts[*target] = incoming_counts[*target].saturating_sub(1);
        }
    }

    debug_assert_eq!(same_stage_by_id.len(), nodes.len());
    Ok(order)
}

fn constraint_edges(
    node_index: usize,
    node: PlanNodeRef<'_>,
    constraint: &SystemOrderingConstraint,
    nodes: &[PlanNodeRef<'_>],
    all_nodes: &PlanNodes<'_>,
    stage: SystemStage,
) -> Result<Vec<(usize, usize)>, ScheduleError> {
    let targets = match constraint {
        SystemOrderingConstraint::Before(reference)
        | SystemOrderingConstraint::After(reference) => {
            resolve_reference(node, reference, nodes, all_nodes, stage)?
        }
    };

    let mut edges = Vec::with_capacity(targets.len());
    for target in targets {
        match constraint {
            SystemOrderingConstraint::Before(_) => edges.push((node_index, target)),
            SystemOrderingConstraint::After(_) => edges.push((target, node_index)),
        }
    }
    Ok(edges)
}

fn resolve_reference(
    node: PlanNodeRef<'_>,
    reference: &SystemRef,
    nodes: &[PlanNodeRef<'_>],
    all_nodes: &PlanNodes<'_>,
    stage: SystemStage,
) -> Result<Vec<usize>, ScheduleError> {
    match reference {
        SystemRef::System(target_id) => {
            let Some(target_stage) = all_nodes.stages_by_id.get(target_id.as_str()).copied() else {
                return Ok(Vec::new());
            };
            if target_stage != stage {
                return Err(ScheduleError::CrossStageConstraint {
                    system_id: node.id().to_string(),
                    target_id: target_id.clone(),
                    stage,
                    target_stage,
                });
            }
            Ok(nodes
                .iter()
                .position(|candidate| candidate.id() == target_id)
                .into_iter()
                .collect())
        }
        SystemRef::Set(set) => {
            let mut targets = Vec::new();
            for (index, candidate) in nodes.iter().copied().enumerate() {
                if candidate.sets().contains(set) {
                    targets.push(index);
                }
            }
            Ok(targets)
        }
    }
}

fn next_ready_node(
    nodes: &[PlanNodeRef<'_>],
    incoming_counts: &[usize],
    scheduled: &[bool],
) -> Option<usize> {
    let mut ready = None;
    for index in 0..nodes.len() {
        if scheduled[index] || incoming_counts[index] != 0 {
            continue;
        }
        if ready
            .map(|ready_index| compare_plan_nodes(nodes[index], nodes[ready_index]).is_lt())
            .unwrap_or(true)
        {
            ready = Some(index);
        }
    }
    ready
}

fn compare_plan_nodes(left: PlanNodeRef<'_>, right: PlanNodeRef<'_>) -> Ordering {
    left.order()
        .cmp(&right.order())
        .then(left.id().cmp(right.id()))
}

fn cycle_chain(nodes: &[PlanNodeRef<'_>], scheduled: &[bool]) -> String {
    let mut ids = Vec::new();
    for (index, node) in nodes.iter().copied().enumerate() {
        if !scheduled[index] {
            ids.push(node.id().to_string());
        }
    }
    ids.sort();
    ids.join(" -> ")
}
