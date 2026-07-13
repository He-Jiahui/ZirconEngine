use std::collections::BTreeMap;

use zircon_runtime::scene::ecs::Resource;
use zircon_runtime::scene::AnimationStateTransitionRuntime;

use crate::{AnimationClipEvaluator, AnimationClipEvaluatorStats, SkeletonTargetTable};

use super::graph_cache::CachedCompiledGraph;
use super::graph_timing_cache::CachedGraphTiming;
use super::interrupted_transition_source::InterruptedTransitionSource;
use super::machine_instance_key::MachineInstanceKey;
use super::state_machine_cache::CachedCompiledStateMachine;

/// Persistent state shared by every phase of `animation.evaluate`.
#[derive(Debug, Default)]
pub struct AnimationEvaluationPipeline {
    clip_evaluator: AnimationClipEvaluator,
    pub(super) graph_cache: BTreeMap<
        (
            zircon_runtime::asset::AssetId,
            zircon_runtime::asset::AssetId,
        ),
        CachedCompiledGraph,
    >,
    pub(super) graph_access_sequence: u64,
    pub(super) graph_timing_cache: BTreeMap<
        (
            zircon_runtime::asset::AssetId,
            zircon_runtime::asset::AssetId,
        ),
        CachedGraphTiming,
    >,
    pub(super) graph_timing_access_sequence: u64,
    pub(super) state_machine_cache:
        BTreeMap<zircon_runtime::asset::AssetId, CachedCompiledStateMachine>,
    pub(super) state_machine_access_sequence: u64,
    pub(super) interrupted_transition_sources:
        BTreeMap<MachineInstanceKey, InterruptedTransitionSource>,
    pub(super) nested_machine_states: BTreeMap<MachineInstanceKey, String>,
    pub(super) nested_machine_transitions:
        BTreeMap<MachineInstanceKey, AnimationStateTransitionRuntime>,
}

impl AnimationEvaluationPipeline {
    pub fn clip_evaluator_stats(&self) -> AnimationClipEvaluatorStats {
        self.clip_evaluator.stats()
    }

    pub fn compiled_graph_cache_len(&self) -> usize {
        self.graph_cache.len()
    }

    pub fn compiled_graph_timing_cache_len(&self) -> usize {
        self.graph_timing_cache.len()
    }

    pub fn compiled_state_machine_cache_len(&self) -> usize {
        self.state_machine_cache.len()
    }

    pub(super) fn clip_evaluator_mut(&mut self) -> &mut AnimationClipEvaluator {
        &mut self.clip_evaluator
    }

    pub(crate) fn skeleton_target_table(
        &self,
        skeleton_id: zircon_runtime::asset::AssetId,
    ) -> Option<std::sync::Arc<SkeletonTargetTable>> {
        self.clip_evaluator.target_table(skeleton_id)
    }
}

impl Resource for AnimationEvaluationPipeline {}
