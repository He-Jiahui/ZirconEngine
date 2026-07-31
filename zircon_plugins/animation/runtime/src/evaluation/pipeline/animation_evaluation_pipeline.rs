use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::scene::ecs::Resource;
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId};

use crate::{
    AnimationClipEvaluator, AnimationClipEvaluatorStats, AnimationEvaluationDiagnostic,
    SkeletonTargetTable,
};

use super::direct_clip_worker::DirectClipWorkerStats;
use super::graph_cache::CachedCompiledGraph;
use super::graph_timing_cache::CachedGraphTiming;
use super::interrupted_transition_source::InterruptedTransitionSource;
use super::machine_instance_key::MachineInstanceKey;
use super::parameter_apply::AnimationEvaluationProjection;
use super::pose_target_binding::PoseTargetBindings;
use super::state_machine_cache::CachedCompiledStateMachine;

const GRAPH_EVALUATION_FRAME_CACHE_LIMIT: usize = 256;

#[derive(Debug)]
pub(super) struct CachedGraphEvaluation {
    pub(super) graph_id: zircon_runtime::asset::AssetId,
    pub(super) skeleton_id: zircon_runtime::asset::AssetId,
    pub(super) parameters: zircon_runtime::core::framework::animation::AnimationParameterMap,
    pub(super) evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
}

/// Cumulative work accepted by the typed animation projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationEvaluationProjectionStats {
    pub skeleton_candidate_count: u64,
    pub clip_player_candidate_count: u64,
    pub sequence_player_candidate_count: u64,
    pub graph_player_candidate_count: u64,
    pub state_machine_player_candidate_count: u64,
    pub clip_pose_request_count: u64,
    pub sequence_request_count: u64,
    pub graph_pose_request_count: u64,
    pub state_machine_pose_request_count: u64,
}

/// Persistent state shared by every phase of `animation.evaluate`.
#[derive(Debug, Default)]
pub struct AnimationEvaluationPipeline {
    clip_evaluator: AnimationClipEvaluator,
    direct_clip_worker_evaluators: Vec<AnimationClipEvaluator>,
    direct_clip_worker_stats: DirectClipWorkerStats,
    pub(super) projection: AnimationEvaluationProjection,
    pose_target_bindings: PoseTargetBindings,
    presentation_poses: Arc<BTreeMap<EntityId, AnimationPoseOutput>>,
    pub(super) graph_evaluation_cache: Vec<CachedGraphEvaluation>,
    pub(super) graph_evaluation_count: u64,
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
        let mut stats = self.clip_evaluator.stats();
        for evaluator in &self.direct_clip_worker_evaluators {
            accumulate_clip_evaluator_stats(&mut stats, evaluator.stats());
        }
        stats
    }

    pub fn direct_clip_worker_stats(&self) -> DirectClipWorkerStats {
        self.direct_clip_worker_stats
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

    pub fn projection_stats(&self) -> AnimationEvaluationProjectionStats {
        self.projection.stats()
    }

    pub fn graph_evaluation_count(&self) -> u64 {
        self.graph_evaluation_count
    }

    pub(super) fn begin_evaluation_frame(&mut self) {
        self.graph_evaluation_cache.clear();
    }

    pub(super) fn cache_graph_evaluation(
        &mut self,
        graph_id: zircon_runtime::asset::AssetId,
        skeleton_id: zircon_runtime::asset::AssetId,
        parameters: &zircon_runtime::core::framework::animation::AnimationParameterMap,
        evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
    ) {
        if self.graph_evaluation_cache.len() >= GRAPH_EVALUATION_FRAME_CACHE_LIMIT {
            self.graph_evaluation_cache.remove(0);
        }
        self.graph_evaluation_cache.push(CachedGraphEvaluation {
            graph_id,
            skeleton_id,
            parameters: parameters.clone(),
            evaluation,
        });
    }

    pub(super) fn update_presentation_poses(
        &mut self,
        pose_source_entities: &BTreeSet<EntityId>,
        pose_updates: BTreeMap<EntityId, AnimationPoseOutput>,
    ) -> Option<Arc<BTreeMap<EntityId, AnimationPoseOutput>>> {
        let all_sources_updated = pose_source_entities.len() == pose_updates.len()
            && pose_source_entities
                .iter()
                .zip(pose_updates.keys())
                .all(|(source, update)| source == update);
        let next = if all_sources_updated {
            Arc::new(pose_updates)
        } else {
            let mut next = self.presentation_poses.as_ref().clone();
            next.retain(|entity, _| pose_source_entities.contains(entity));
            next.extend(pose_updates);
            Arc::new(next)
        };
        if next.as_ref() == self.presentation_poses.as_ref() {
            return None;
        }
        self.presentation_poses = Arc::clone(&next);
        Some(next)
    }

    pub(super) fn reset_evaluation_state(&mut self) {
        self.projection = AnimationEvaluationProjection::default();
        self.pose_target_bindings.clear();
        self.presentation_poses = Arc::default();
        self.direct_clip_worker_evaluators.clear();
        self.direct_clip_worker_stats = DirectClipWorkerStats::default();
        self.graph_evaluation_cache.clear();
        self.interrupted_transition_sources.clear();
        self.nested_machine_states.clear();
        self.nested_machine_transitions.clear();
    }

    pub(super) fn presentation_poses(&self) -> Arc<BTreeMap<EntityId, AnimationPoseOutput>> {
        Arc::clone(&self.presentation_poses)
    }

    pub(super) fn pose_target_binding_is_current(
        &self,
        root: EntityId,
        world: &zircon_runtime::scene::World,
    ) -> bool {
        self.pose_target_bindings.is_current_for(root, world)
    }

    pub(super) fn cache_pose_target_binding(
        &mut self,
        index: zircon_runtime::scene::world::CompiledDescendantNameIndex,
    ) {
        self.pose_target_bindings.insert(index);
    }

    pub(super) fn resolve_pose_target(&self, root: EntityId, bone_name: &str) -> Option<EntityId> {
        self.pose_target_bindings.resolve(root, bone_name)
    }

    pub(super) fn clip_evaluator_mut(&mut self) -> &mut AnimationClipEvaluator {
        &mut self.clip_evaluator
    }

    pub(super) fn take_direct_clip_worker_evaluator(
        &mut self,
        shard_index: usize,
    ) -> AnimationClipEvaluator {
        if shard_index == 0 {
            return std::mem::take(&mut self.clip_evaluator);
        }
        let worker_index = shard_index - 1;
        if worker_index >= self.direct_clip_worker_evaluators.len() {
            self.direct_clip_worker_evaluators
                .resize_with(worker_index + 1, AnimationClipEvaluator::default);
        }
        std::mem::take(&mut self.direct_clip_worker_evaluators[worker_index])
    }

    pub(super) fn restore_direct_clip_worker_evaluator(
        &mut self,
        shard_index: usize,
        evaluator: AnimationClipEvaluator,
    ) {
        if shard_index == 0 {
            self.clip_evaluator = evaluator;
            return;
        }
        let worker_index = shard_index - 1;
        if worker_index >= self.direct_clip_worker_evaluators.len() {
            self.direct_clip_worker_evaluators
                .resize_with(worker_index, AnimationClipEvaluator::default);
            self.direct_clip_worker_evaluators.push(evaluator);
            return;
        }
        self.direct_clip_worker_evaluators[worker_index] = evaluator;
    }

    pub(super) fn record_direct_clip_worker_batches(
        &mut self,
        batch_lengths: impl IntoIterator<Item = usize>,
        shard_count: usize,
    ) {
        let mut total_instance_count = 0;
        let mut min_shard_len = usize::MAX;
        let mut max_shard_len = 0;
        for batch_len in batch_lengths {
            total_instance_count += batch_len;
            min_shard_len = min_shard_len.min(batch_len);
            max_shard_len = max_shard_len.max(batch_len);
        }
        self.direct_clip_worker_stats.total_batch_count += shard_count as u64;
        self.direct_clip_worker_stats.total_owner_submission_count += shard_count as u64;
        self.direct_clip_worker_stats.total_instance_count += total_instance_count as u64;
        self.direct_clip_worker_stats.last_instance_count = total_instance_count;
        self.direct_clip_worker_stats.last_shard_count = shard_count;
        self.direct_clip_worker_stats.last_owner_submission_count = shard_count;
        self.direct_clip_worker_stats.last_min_shard_len =
            if shard_count == 0 { 0 } else { min_shard_len };
        self.direct_clip_worker_stats.last_max_shard_len = max_shard_len;
    }

    pub(super) fn drain_clip_evaluation_diagnostics(
        &mut self,
    ) -> Vec<AnimationEvaluationDiagnostic> {
        let mut diagnostics = self.clip_evaluator.drain_diagnostics();
        for evaluator in &mut self.direct_clip_worker_evaluators {
            diagnostics.extend(evaluator.drain_diagnostics());
        }
        diagnostics
    }

    pub(crate) fn skeleton_target_table(
        &self,
        skeleton_id: zircon_runtime::asset::AssetId,
    ) -> Option<std::sync::Arc<SkeletonTargetTable>> {
        self.clip_evaluator.target_table(skeleton_id).or_else(|| {
            self.direct_clip_worker_evaluators
                .iter()
                .find_map(|evaluator| evaluator.target_table(skeleton_id))
        })
    }
}

impl Resource for AnimationEvaluationPipeline {}

fn accumulate_clip_evaluator_stats(
    target: &mut AnimationClipEvaluatorStats,
    source: AnimationClipEvaluatorStats,
) {
    target.skeleton_compile_count += source.skeleton_compile_count;
    target.clip_compile_count += source.clip_compile_count;
    target.clip_cache_hit_count += source.clip_cache_hit_count;
    target.pose_pool_miss_count += source.pose_pool_miss_count;
    target.skeleton_eviction_count += source.skeleton_eviction_count;
    target.clip_eviction_count += source.clip_eviction_count;
    target.cached_skeleton_count += source.cached_skeleton_count;
    target.cached_clip_count += source.cached_clip_count;
}
