use super::{
    InternalSceneSystem, SceneSystemClockDomain, SceneSystemDescriptor, SceneSystemPauseBehavior,
    SceneSystemTickPolicy, ScheduledSceneStep, ScheduledSceneStepRef, SystemStage,
};

const RECEIPT_VERSION: u16 = 1;
const BLAKE3_V1_ALGORITHM_ID: u16 = 1;
const TOPOLOGY_DOMAIN: &[u8] = b"zircon.schedule.topology.v1";
const TICK_POLICY_DOMAIN: &[u8] = b"zircon.schedule.tick-policy.v1";
const RECEIPT_DOMAIN: &[u8] = b"zircon.schedule.receipt.v1";
const STAGE_SECTION_TAG: u8 = 0xa1;
const EDGE_SECTION_TAG: u8 = 0xa2;

/// BLAKE3 digest emitted by a compiled scene schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleBuildDigest([u8; blake3::OUT_LEN]);

impl ScheduleBuildDigest {
    pub const fn as_bytes(self) -> [u8; blake3::OUT_LEN] {
        self.0
    }
}

/// Immutable identity of one successfully compiled scene schedule.
///
/// It describes the execution graph, rather than registration order. Replay
/// and BuildSet owners can persist these digests to reject an incompatible
/// system graph before simulation begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleBuildReceipt {
    version: u16,
    digest_algorithm: u16,
    digest: ScheduleBuildDigest,
    topology_digest: ScheduleBuildDigest,
    tick_policy_digest: ScheduleBuildDigest,
    system_count: u64,
    execution_step_count: u64,
    edge_count: u64,
}

impl ScheduleBuildReceipt {
    pub const VERSION: u16 = RECEIPT_VERSION;
    pub const BLAKE3_V1_ALGORITHM_ID: u16 = BLAKE3_V1_ALGORITHM_ID;

    pub const fn version(self) -> u16 {
        self.version
    }

    pub const fn digest_algorithm(self) -> u16 {
        self.digest_algorithm
    }

    pub const fn digest(self) -> ScheduleBuildDigest {
        self.digest
    }

    pub const fn topology_digest(self) -> ScheduleBuildDigest {
        self.topology_digest
    }

    pub const fn tick_policy_digest(self) -> ScheduleBuildDigest {
        self.tick_policy_digest
    }

    pub const fn system_count(self) -> u64 {
        self.system_count
    }

    pub const fn execution_step_count(self) -> u64 {
        self.execution_step_count
    }

    pub const fn edge_count(self) -> u64 {
        self.edge_count
    }

    pub(crate) fn from_compiled_plan(
        stages: &[SystemStage],
        internal_systems_by_stage: &[Vec<SceneSystemDescriptor>; SystemStage::COUNT],
        native_steps_by_stage: &[Vec<ScheduledSceneStep>; SystemStage::COUNT],
        resolved_edges: &[ResolvedScheduleEdge],
    ) -> Self {
        let mut topology = blake3::Hasher::new();
        topology.update(TOPOLOGY_DOMAIN);
        topology.update(&RECEIPT_VERSION.to_le_bytes());

        let mut tick_policy = blake3::Hasher::new();
        tick_policy.update(TICK_POLICY_DOMAIN);
        tick_policy.update(&RECEIPT_VERSION.to_le_bytes());

        let mut system_count = 0_u64;
        let mut execution_step_count = 0_u64;
        for stage in stages.iter().copied() {
            let stage_execution_step_count = internal_systems_by_stage[stage.rank()]
                .len()
                .saturating_add(native_steps_by_stage[stage.rank()].len());
            topology.update(&[STAGE_SECTION_TAG, stage.rank() as u8]);
            topology.update(&(stage_execution_step_count as u64).to_le_bytes());
            tick_policy.update(&[STAGE_SECTION_TAG, stage.rank() as u8]);
            tick_policy.update(&(stage_execution_step_count as u64).to_le_bytes());
            for step in ScheduledSceneStep::iter_sorted_for_stage(
                stage,
                &internal_systems_by_stage[stage.rank()],
                &native_steps_by_stage[stage.rank()],
            ) {
                execution_step_count = execution_step_count.saturating_add(1);
                match step {
                    ScheduledSceneStepRef::Internal(system) => {
                        system_count = system_count.saturating_add(1);
                        append_internal_step(&mut topology, system);
                        append_tick_policy(
                            &mut tick_policy,
                            1,
                            system.id.as_str(),
                            system.stage,
                            system.order,
                            system.system().tick_policy(),
                        );
                    }
                    ScheduledSceneStepRef::Native {
                        id,
                        stage,
                        order,
                        tick_policy: policy,
                        worker_safe,
                        conservative_world_writer,
                    } => {
                        system_count = system_count.saturating_add(1);
                        append_native_step(
                            &mut topology,
                            id,
                            stage,
                            order,
                            worker_safe,
                            conservative_world_writer,
                        );
                        append_tick_policy(&mut tick_policy, 2, id, stage, order, policy);
                    }
                    ScheduledSceneStepRef::Runtime {
                        id,
                        stage,
                        order,
                        tick_policy: policy,
                    } => {
                        system_count = system_count.saturating_add(1);
                        append_runtime_step(&mut topology, id, stage, order);
                        append_tick_policy(&mut tick_policy, 3, id, stage, order, policy);
                    }
                    ScheduledSceneStepRef::ApplyDeferred {
                        after_system_id,
                        stage,
                        order,
                        tick_policy: policy,
                    } => {
                        append_apply_deferred_step(&mut topology, after_system_id, stage, order);
                        append_tick_policy(
                            &mut tick_policy,
                            4,
                            after_system_id,
                            stage,
                            order,
                            policy,
                        );
                    }
                }
            }
        }

        let mut edges = resolved_edges.to_vec();
        edges.sort_unstable_by(|left, right| {
            left.stage
                .rank()
                .cmp(&right.stage.rank())
                .then(left.before_system_id.cmp(&right.before_system_id))
                .then(left.after_system_id.cmp(&right.after_system_id))
        });
        topology.update(&[EDGE_SECTION_TAG]);
        topology.update(&(edges.len() as u64).to_le_bytes());
        for edge in &edges {
            topology.update(&[edge.stage.rank() as u8]);
            append_string(&mut topology, edge.before_system_id.as_str());
            append_string(&mut topology, edge.after_system_id.as_str());
        }

        let topology_digest = ScheduleBuildDigest(*topology.finalize().as_bytes());
        let tick_policy_digest = ScheduleBuildDigest(*tick_policy.finalize().as_bytes());
        let mut receipt = blake3::Hasher::new();
        receipt.update(RECEIPT_DOMAIN);
        receipt.update(&RECEIPT_VERSION.to_le_bytes());
        receipt.update(&BLAKE3_V1_ALGORITHM_ID.to_le_bytes());
        receipt.update(&topology_digest.0);
        receipt.update(&tick_policy_digest.0);
        receipt.update(&system_count.to_le_bytes());
        receipt.update(&execution_step_count.to_le_bytes());
        receipt.update(&(edges.len() as u64).to_le_bytes());

        Self {
            version: RECEIPT_VERSION,
            digest_algorithm: BLAKE3_V1_ALGORITHM_ID,
            digest: ScheduleBuildDigest(*receipt.finalize().as_bytes()),
            topology_digest,
            tick_policy_digest,
            system_count,
            execution_step_count,
            edge_count: edges.len() as u64,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedScheduleEdge {
    stage: SystemStage,
    before_system_id: String,
    after_system_id: String,
}

impl ResolvedScheduleEdge {
    pub(crate) fn new(
        stage: SystemStage,
        before_system_id: impl Into<String>,
        after_system_id: impl Into<String>,
    ) -> Self {
        Self {
            stage,
            before_system_id: before_system_id.into(),
            after_system_id: after_system_id.into(),
        }
    }
}

fn append_internal_step(hasher: &mut blake3::Hasher, system: &SceneSystemDescriptor) {
    hasher.update(&[1]);
    append_string(hasher, system.id.as_str());
    hasher.update(&[system.stage.rank() as u8]);
    hasher.update(&system.order.to_le_bytes());
    hasher.update(&[internal_system_code(system.system)]);
}

fn append_native_step(
    hasher: &mut blake3::Hasher,
    id: &str,
    stage: SystemStage,
    order: i32,
    worker_safe: bool,
    conservative_world_writer: bool,
) {
    hasher.update(&[2]);
    append_string(hasher, id);
    hasher.update(&[stage.rank() as u8]);
    hasher.update(&order.to_le_bytes());
    hasher.update(&[u8::from(worker_safe)]);
    hasher.update(&[u8::from(conservative_world_writer)]);
}

fn append_runtime_step(hasher: &mut blake3::Hasher, id: &str, stage: SystemStage, order: i32) {
    hasher.update(&[3]);
    append_string(hasher, id);
    hasher.update(&[stage.rank() as u8]);
    hasher.update(&order.to_le_bytes());
}

fn append_apply_deferred_step(
    hasher: &mut blake3::Hasher,
    after_system_id: &str,
    stage: SystemStage,
    order: i32,
) {
    hasher.update(&[4]);
    append_string(hasher, after_system_id);
    hasher.update(&[stage.rank() as u8]);
    hasher.update(&order.to_le_bytes());
}

fn append_tick_policy(
    hasher: &mut blake3::Hasher,
    step_kind: u8,
    id: &str,
    stage: SystemStage,
    order: i32,
    policy: SceneSystemTickPolicy,
) {
    hasher.update(&[step_kind]);
    append_string(hasher, id);
    hasher.update(&[stage.rank() as u8]);
    hasher.update(&order.to_le_bytes());
    hasher.update(&[clock_domain_code(policy.clock_domain())]);
    hasher.update(&[pause_behavior_code(policy.pause_behavior())]);
}

fn append_string(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn internal_system_code(system: InternalSceneSystem) -> u8 {
    match system {
        InternalSceneSystem::ApplyDeferred => 1,
        InternalSceneSystem::UpdateEvents => 2,
        InternalSceneSystem::HierarchyValidity => 3,
        InternalSceneSystem::ActiveHierarchy => 4,
        InternalSceneSystem::WorldTransform => 5,
        InternalSceneSystem::NodeCache => 6,
        InternalSceneSystem::RenderExtractPrepare => 7,
    }
}

fn clock_domain_code(domain: SceneSystemClockDomain) -> u8 {
    match domain {
        SceneSystemClockDomain::Virtual => 1,
        SceneSystemClockDomain::MonotonicReal => 2,
        SceneSystemClockDomain::Fixed => 3,
    }
}

fn pause_behavior_code(behavior: SceneSystemPauseBehavior) -> u8 {
    match behavior {
        SceneSystemPauseBehavior::SkipWhenVirtualPaused => 1,
        SceneSystemPauseBehavior::RunWhenVirtualPaused => 2,
    }
}
