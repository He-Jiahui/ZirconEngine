use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::framework::animation::{
    AnimationParameterContentFingerprint, AnimationParameterSet, AnimationPoseMap,
    AnimationPoseOutput, AnimationPoseSnapshot,
};
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
use super::sequences::CachedCompiledSequence;
use super::state_machine_cache::{CachedCompiledStateMachine, StateMachineInstanceCache};

const GRAPH_EVALUATION_FRAME_CACHE_LIMIT: usize = 256;

pub(super) type GraphEvaluationCacheKey = (
    zircon_runtime::asset::AssetId,
    zircon_runtime::asset::AssetId,
    AnimationParameterContentFingerprint,
);

#[derive(Debug)]
pub(super) struct CachedGraphEvaluation {
    pub(super) parameters: AnimationParameterSet,
    pub(super) evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
}

#[derive(Debug, Default)]
pub(super) struct StateMachineRuntimeJournal {
    previous_by_instance: BTreeMap<MachineInstanceKey, StateMachineRuntimePrevious>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PresentationPoseChange {
    Full,
    Partial { changed_entities: Box<[EntityId]> },
}

#[derive(Clone, Debug)]
pub(super) struct PresentationPoseUpdate {
    pub(super) snapshot: AnimationPoseSnapshot,
    pub(super) change: PresentationPoseChange,
}

#[derive(Debug)]
struct StateMachineRuntimePrevious {
    previous_interrupted_transition_source: Option<InterruptedTransitionSource>,
    previous_nested_machine_state: Option<String>,
    previous_nested_machine_transition: Option<AnimationStateTransitionRuntime>,
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
    prepared_replacement_epoch: Option<u64>,
    evaluation_state_active: bool,
    clip_evaluator: AnimationClipEvaluator,
    direct_clip_worker_evaluators: Vec<AnimationClipEvaluator>,
    direct_clip_worker_stats: DirectClipWorkerStats,
    projection: Mutex<AnimationEvaluationProjection>,
    pose_target_bindings: PoseTargetBindings,
    presentation_poses: AnimationPoseSnapshot,
    pub(super) graph_evaluation_cache: BTreeMap<GraphEvaluationCacheKey, CachedGraphEvaluation>,
    pub(super) graph_evaluation_count: u64,
    pub(super) sequence_cache: BTreeMap<zircon_runtime::asset::AssetId, CachedCompiledSequence>,
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
    // Warm-start hints affect cost only, so semantic event rollback must not deep-copy them.
    pub(super) state_machine_instance_cache: StateMachineInstanceCache,
    pub(super) interrupted_transition_sources:
        BTreeMap<MachineInstanceKey, InterruptedTransitionSource>,
    pub(super) nested_machine_states: BTreeMap<MachineInstanceKey, String>,
    pub(super) nested_machine_transitions:
        BTreeMap<MachineInstanceKey, AnimationStateTransitionRuntime>,
    state_machine_runtime_journal: Option<StateMachineRuntimeJournal>,
    clip_event_admission_cursor: Option<EntityId>,
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
        self.projection_guard().stats()
    }

    pub(super) fn take_projection(&mut self) -> AnimationEvaluationProjection {
        std::mem::take(&mut *self.projection_guard())
    }

    pub(super) fn restore_projection(&mut self, projection: AnimationEvaluationProjection) {
        *self.projection_guard() = projection;
    }

    pub(super) fn commit_projection_revision_stage(
        &mut self,
        stage: super::parameter_apply::AnimationProjectionRevisionStage,
        deferred_entities: &BTreeSet<EntityId>,
    ) {
        self.projection_guard()
            .commit_revision_stage(stage, deferred_entities);
    }

    fn projection_guard(&self) -> MutexGuard<'_, AnimationEvaluationProjection> {
        self.projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn graph_evaluation_count(&self) -> u64 {
        self.graph_evaluation_count
    }

    pub(super) fn clip_event_admission_cursor(&self) -> Option<EntityId> {
        self.clip_event_admission_cursor
    }

    pub(super) fn begin_state_machine_runtime_transaction(
        &mut self,
        active_entities: &BTreeSet<EntityId>,
    ) {
        self.state_machine_instance_cache
            .retain_entities(active_entities);
        self.retain_interrupted_transition_sources(active_entities);
        self.retain_nested_machine_instances(active_entities);
        debug_assert!(self.state_machine_runtime_journal.is_none());
        self.state_machine_runtime_journal = Some(StateMachineRuntimeJournal::default());
    }

    pub(super) fn finish_state_machine_runtime_transaction(
        &mut self,
    ) -> StateMachineRuntimeJournal {
        self.state_machine_runtime_journal
            .take()
            .unwrap_or_default()
    }

    fn record_state_machine_runtime_before_write(&mut self, instance: &MachineInstanceKey) {
        let should_record = self
            .state_machine_runtime_journal
            .as_ref()
            .is_some_and(|journal| !journal.previous_by_instance.contains_key(instance));
        if !should_record {
            return;
        }
        let previous = StateMachineRuntimePrevious {
            previous_interrupted_transition_source: self
                .interrupted_transition_sources
                .get(instance)
                .cloned(),
            previous_nested_machine_state: self.nested_machine_states.get(instance).cloned(),
            previous_nested_machine_transition: self
                .nested_machine_transitions
                .get(instance)
                .cloned(),
        };
        if let Some(journal) = self.state_machine_runtime_journal.as_mut() {
            journal
                .previous_by_instance
                .insert(instance.clone(), previous);
        }
    }

    pub(super) fn set_nested_machine_state(&mut self, instance: MachineInstanceKey, state: String) {
        self.record_state_machine_runtime_before_write(&instance);
        self.nested_machine_states.insert(instance, state);
    }

    pub(super) fn set_nested_machine_transition(
        &mut self,
        instance: MachineInstanceKey,
        transition: AnimationStateTransitionRuntime,
    ) {
        self.record_state_machine_runtime_before_write(&instance);
        self.nested_machine_transitions.insert(instance, transition);
    }

    pub(super) fn clear_nested_machine_transition(&mut self, instance: &MachineInstanceKey) {
        self.record_state_machine_runtime_before_write(instance);
        self.nested_machine_transitions.remove(instance);
    }

    pub(super) fn record_state_machine_interrupted_transition_source(
        &mut self,
        instance: MachineInstanceKey,
        from_state: &str,
        to_state: &str,
        pose: AnimationPoseOutput,
    ) {
        self.record_state_machine_runtime_before_write(&instance);
        self.record_interrupted_transition_source(instance, from_state, to_state, pose);
    }

    pub(super) fn clear_state_machine_interrupted_transition_source(
        &mut self,
        instance: &MachineInstanceKey,
    ) {
        self.record_state_machine_runtime_before_write(instance);
        self.clear_interrupted_transition_source(instance);
    }

    pub(super) fn set_clip_event_admission_cursor(&mut self, next_cursor: Option<EntityId>) {
        self.clip_event_admission_cursor = next_cursor;
    }

    pub(super) fn finish_clip_event_admission(
        &mut self,
        journal: StateMachineRuntimeJournal,
        deferred_entities: &BTreeSet<EntityId>,
        next_cursor: Option<EntityId>,
    ) {
        self.clip_event_admission_cursor = next_cursor;
        self.restore_deferred_state_machine_entities(journal, deferred_entities);
    }

    fn restore_deferred_state_machine_entities(
        &mut self,
        journal: StateMachineRuntimeJournal,
        deferred_entities: &BTreeSet<EntityId>,
    ) {
        if deferred_entities.is_empty() {
            return;
        }
        for (instance, previous) in journal.previous_by_instance {
            if !deferred_entities.contains(&instance.entity()) {
                continue;
            }
            restore_state_machine_runtime_entry(
                &mut self.interrupted_transition_sources,
                instance.clone(),
                previous.previous_interrupted_transition_source,
            );
            restore_state_machine_runtime_entry(
                &mut self.nested_machine_states,
                instance.clone(),
                previous.previous_nested_machine_state,
            );
            restore_state_machine_runtime_entry(
                &mut self.nested_machine_transitions,
                instance,
                previous.previous_nested_machine_transition,
            );
        }
    }

    pub(super) fn take_sequence_cache(
        &mut self,
    ) -> BTreeMap<zircon_runtime::asset::AssetId, CachedCompiledSequence> {
        std::mem::take(&mut self.sequence_cache)
    }

    pub(super) fn restore_sequence_cache(
        &mut self,
        sequence_cache: BTreeMap<zircon_runtime::asset::AssetId, CachedCompiledSequence>,
    ) {
        self.sequence_cache = sequence_cache;
    }

    pub(super) fn begin_evaluation_frame(&mut self, replacement_epoch: u64) -> bool {
        let reset = self.prepared_replacement_epoch != Some(replacement_epoch);
        if reset {
            self.reset_evaluation_state(true);
            self.prepared_replacement_epoch = Some(replacement_epoch);
        }
        self.evaluation_state_active = true;
        self.graph_evaluation_cache.clear();
        reset
    }

    pub(super) fn ensure_empty_evaluation_state(&mut self, replacement_epoch: u64) -> bool {
        let replacement_changed = self.prepared_replacement_epoch != Some(replacement_epoch);
        let reset = replacement_changed || self.evaluation_state_active;
        if reset {
            self.reset_evaluation_state(replacement_changed);
        }
        self.prepared_replacement_epoch = Some(replacement_epoch);
        self.evaluation_state_active = false;
        reset
    }

    pub(super) fn cache_graph_evaluation(
        &mut self,
        graph_id: zircon_runtime::asset::AssetId,
        skeleton_id: zircon_runtime::asset::AssetId,
        parameters: &AnimationParameterSet,
        evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
    ) {
        let cache_key = (graph_id, skeleton_id, parameters.content_fingerprint());
        if self.graph_evaluation_cache.len() >= GRAPH_EVALUATION_FRAME_CACHE_LIMIT {
            return;
        }
        self.graph_evaluation_cache
            .entry(cache_key)
            .or_insert_with(|| CachedGraphEvaluation {
                parameters: parameters.clone(),
                evaluation,
            });
    }

    pub(super) fn update_presentation_poses(
        &mut self,
        pose_source_entities: &BTreeSet<EntityId>,
        pose_updates: BTreeMap<EntityId, AnimationPoseOutput>,
    ) -> Option<PresentationPoseUpdate> {
        let all_sources_updated = pose_source_entities.len() == pose_updates.len()
            && pose_source_entities
                .iter()
                .zip(pose_updates.keys())
                .all(|(source, update)| source == update);
        if all_sources_updated {
            if animation_pose_updates_match(self.presentation_poses.as_ref(), &pose_updates) {
                return None;
            }
            let next = Arc::new(seal_pose_updates(pose_updates));
            self.presentation_poses = Arc::clone(&next);
            return Some(PresentationPoseUpdate {
                snapshot: next,
                change: PresentationPoseChange::Full,
            });
        }

        let removed_entities = self
            .presentation_poses
            .keys()
            .filter(|entity| !pose_source_entities.contains(entity))
            .copied()
            .collect::<Vec<_>>();
        let changed_updates = pose_updates
            .into_iter()
            .filter(|(entity, pose)| {
                self.presentation_poses
                    .get(entity)
                    .is_none_or(|current| current.as_ref() != pose)
            })
            .collect::<BTreeMap<_, _>>();
        if removed_entities.is_empty() && changed_updates.is_empty() {
            return None;
        }

        let changed_entities = removed_entities
            .iter()
            .copied()
            .chain(changed_updates.keys().copied())
            .collect::<Box<[_]>>();
        let mut next = self.presentation_poses.as_ref().clone();
        for entity in removed_entities {
            next.remove(&entity);
        }
        next.extend(
            changed_updates
                .into_iter()
                .map(|(entity, pose)| (entity, Arc::new(pose))),
        );
        let next = Arc::new(next);
        self.presentation_poses = Arc::clone(&next);
        Some(PresentationPoseUpdate {
            snapshot: next,
            change: PresentationPoseChange::Partial { changed_entities },
        })
    }

    fn reset_evaluation_state(&mut self, retire_world_bound_caches: bool) {
        self.clip_evaluator.reset_diagnostics();
        for evaluator in &mut self.direct_clip_worker_evaluators {
            evaluator.reset_diagnostics();
        }
        *self.projection_guard() = AnimationEvaluationProjection::default();
        self.pose_target_bindings.clear();
        self.presentation_poses = Arc::default();
        self.direct_clip_worker_stats = DirectClipWorkerStats::default();
        if retire_world_bound_caches {
            self.sequence_cache.clear();
        }
        self.graph_evaluation_cache.clear();
        self.interrupted_transition_sources.clear();
        self.nested_machine_states.clear();
        self.nested_machine_transitions.clear();
        self.state_machine_runtime_journal = None;
        self.state_machine_instance_cache.clear();
        self.clip_event_admission_cursor = None;
    }

    pub(super) fn presentation_poses(&self) -> AnimationPoseSnapshot {
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

    pub(super) fn drain_clip_evaluation_diagnostics_excluding(
        &mut self,
        deferred_entities: &BTreeSet<EntityId>,
    ) -> Vec<AnimationEvaluationDiagnostic> {
        let mut diagnostics = self
            .clip_evaluator
            .drain_diagnostics_excluding(deferred_entities);
        for evaluator in &mut self.direct_clip_worker_evaluators {
            diagnostics.extend(evaluator.drain_diagnostics_excluding(deferred_entities));
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

fn animation_pose_updates_match(
    current: &AnimationPoseMap,
    updates: &BTreeMap<EntityId, AnimationPoseOutput>,
) -> bool {
    current.len() == updates.len()
        && current.iter().zip(updates).all(
            |((current_entity, current_pose), (update_entity, update_pose))| {
                current_entity == update_entity && current_pose.as_ref() == update_pose
            },
        )
}

fn seal_pose_updates(updates: BTreeMap<EntityId, AnimationPoseOutput>) -> AnimationPoseMap {
    updates
        .into_iter()
        .map(|(entity, pose)| (entity, Arc::new(pose)))
        .collect()
}

fn restore_state_machine_runtime_entry<Value>(
    values: &mut BTreeMap<MachineInstanceKey, Value>,
    instance: MachineInstanceKey,
    previous: Option<Value>,
) {
    if let Some(previous) = previous {
        values.insert(instance, previous);
    } else {
        values.remove(&instance);
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

#[cfg(test)]
#[path = "tests/animation_evaluation_pipeline.rs"]
mod tests;
