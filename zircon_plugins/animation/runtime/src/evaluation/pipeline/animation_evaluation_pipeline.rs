use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
use super::sequences::CachedCompiledSequence;
use super::state_machine_cache::CachedCompiledStateMachine;

const GRAPH_EVALUATION_FRAME_CACHE_LIMIT: usize = 256;

#[derive(Debug)]
pub(super) struct CachedGraphEvaluation {
    pub(super) graph_id: zircon_runtime::asset::AssetId,
    pub(super) skeleton_id: zircon_runtime::asset::AssetId,
    pub(super) parameters: zircon_runtime::core::framework::animation::AnimationParameterMap,
    pub(super) evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
}

#[derive(Clone, Debug)]
pub(super) struct StateMachineRuntimeCheckpoint {
    checkpointed_entities: BTreeSet<EntityId>,
    interrupted_transition_sources: BTreeMap<MachineInstanceKey, InterruptedTransitionSource>,
    nested_machine_states: BTreeMap<MachineInstanceKey, String>,
    nested_machine_transitions: BTreeMap<MachineInstanceKey, AnimationStateTransitionRuntime>,
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
    pub(super) projection: AnimationEvaluationProjection,
    pose_target_bindings: PoseTargetBindings,
    presentation_poses: Arc<BTreeMap<EntityId, AnimationPoseOutput>>,
    pub(super) graph_evaluation_cache: VecDeque<CachedGraphEvaluation>,
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
    pub(super) interrupted_transition_sources:
        BTreeMap<MachineInstanceKey, InterruptedTransitionSource>,
    pub(super) nested_machine_states: BTreeMap<MachineInstanceKey, String>,
    pub(super) nested_machine_transitions:
        BTreeMap<MachineInstanceKey, AnimationStateTransitionRuntime>,
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
        self.projection.stats()
    }

    pub fn graph_evaluation_count(&self) -> u64 {
        self.graph_evaluation_count
    }

    pub(super) fn clip_event_admission_cursor(&self) -> Option<EntityId> {
        self.clip_event_admission_cursor
    }

    pub(super) fn state_machine_runtime_checkpoint(
        &self,
        active_entities: &BTreeSet<EntityId>,
    ) -> StateMachineRuntimeCheckpoint {
        StateMachineRuntimeCheckpoint {
            checkpointed_entities: active_entities.clone(),
            interrupted_transition_sources: self
                .interrupted_transition_sources
                .iter()
                .filter(|(instance, _)| active_entities.contains(&instance.entity()))
                .map(|(instance, source)| (instance.clone(), source.clone()))
                .collect(),
            nested_machine_states: self
                .nested_machine_states
                .iter()
                .filter(|(instance, _)| active_entities.contains(&instance.entity()))
                .map(|(instance, state)| (instance.clone(), state.clone()))
                .collect(),
            nested_machine_transitions: self
                .nested_machine_transitions
                .iter()
                .filter(|(instance, _)| active_entities.contains(&instance.entity()))
                .map(|(instance, transition)| (instance.clone(), transition.clone()))
                .collect(),
        }
    }

    pub(super) fn set_clip_event_admission_cursor(&mut self, next_cursor: Option<EntityId>) {
        self.clip_event_admission_cursor = next_cursor;
    }

    pub(super) fn finish_clip_event_admission(
        &mut self,
        checkpoint: StateMachineRuntimeCheckpoint,
        deferred_entities: &BTreeSet<EntityId>,
        next_cursor: Option<EntityId>,
    ) {
        self.clip_event_admission_cursor = next_cursor;
        self.restore_deferred_state_machine_entities(checkpoint, deferred_entities);
    }

    fn restore_deferred_state_machine_entities(
        &mut self,
        checkpoint: StateMachineRuntimeCheckpoint,
        deferred_entities: &BTreeSet<EntityId>,
    ) {
        if deferred_entities.is_empty() {
            return;
        }
        let restore_entities = checkpoint
            .checkpointed_entities
            .intersection(deferred_entities)
            .copied()
            .collect::<BTreeSet<_>>();
        self.interrupted_transition_sources
            .retain(|instance, _| !restore_entities.contains(&instance.entity()));
        self.interrupted_transition_sources.extend(
            checkpoint
                .interrupted_transition_sources
                .into_iter()
                .filter(|(instance, _)| restore_entities.contains(&instance.entity())),
        );
        self.nested_machine_states
            .retain(|instance, _| !restore_entities.contains(&instance.entity()));
        self.nested_machine_states.extend(
            checkpoint
                .nested_machine_states
                .into_iter()
                .filter(|(instance, _)| restore_entities.contains(&instance.entity())),
        );
        self.nested_machine_transitions
            .retain(|instance, _| !restore_entities.contains(&instance.entity()));
        self.nested_machine_transitions.extend(
            checkpoint
                .nested_machine_transitions
                .into_iter()
                .filter(|(instance, _)| restore_entities.contains(&instance.entity())),
        );
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
        parameters: &zircon_runtime::core::framework::animation::AnimationParameterMap,
        evaluation: std::sync::Arc<crate::CompiledAnimationGraphEvaluation>,
    ) {
        if self.graph_evaluation_cache.len() >= GRAPH_EVALUATION_FRAME_CACHE_LIMIT {
            self.graph_evaluation_cache.pop_front();
        }
        self.graph_evaluation_cache
            .push_back(CachedGraphEvaluation {
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

    fn reset_evaluation_state(&mut self, retire_world_bound_caches: bool) {
        self.clip_evaluator.reset_diagnostics();
        for evaluator in &mut self.direct_clip_worker_evaluators {
            evaluator.reset_diagnostics();
        }
        self.projection = AnimationEvaluationProjection::default();
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
        self.clip_event_admission_cursor = None;
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
mod tests {
    use std::collections::{BTreeSet, VecDeque};
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use zircon_runtime::core::framework::animation::{
        AnimationParameterMap, AnimationPoseOutput, AnimationPoseSource,
    };
    use zircon_runtime::core::resource::ResourceId;
    use zircon_runtime::scene::AnimationStateTransitionRuntime;

    use crate::{
        AnimationAssetRevision, AnimationEvaluationError, CompiledAnimationGraphEvaluation,
    };

    use super::{
        AnimationEvaluationPipeline, GRAPH_EVALUATION_FRAME_CACHE_LIMIT, MachineInstanceKey,
    };

    fn pose() -> AnimationPoseOutput {
        AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones: Vec::new(),
        }
    }

    #[test]
    fn replacement_epoch_and_empty_mode_prepare_without_duplicate_resets() {
        let mut pipeline = AnimationEvaluationPipeline::default();
        pipeline.presentation_poses.make_mut().insert(17, pose());

        assert!(pipeline.begin_evaluation_frame(1));
        assert!(pipeline.presentation_poses.is_empty());

        pipeline.presentation_poses.make_mut().insert(18, pose());
        assert!(!pipeline.begin_evaluation_frame(1));
        assert!(pipeline.presentation_poses.contains_key(&18));

        assert!(pipeline.ensure_empty_evaluation_state(1));
        assert!(pipeline.presentation_poses.is_empty());
        assert!(!pipeline.ensure_empty_evaluation_state(1));
        assert!(!pipeline.begin_evaluation_frame(1));

        assert!(pipeline.begin_evaluation_frame(2));
        assert!(pipeline.presentation_poses.is_empty());
    }

    #[test]
    fn graph_evaluation_frame_cache_preserves_fifo_eviction_order() {
        let mut pipeline = AnimationEvaluationPipeline::default();
        let parameters = AnimationParameterMap::default();

        for index in 0..=GRAPH_EVALUATION_FRAME_CACHE_LIMIT {
            let label = format!("animation.graph.{index}");
            let graph_id = ResourceId::from_stable_label(&label);
            pipeline.cache_graph_evaluation(
                graph_id,
                graph_id,
                &parameters,
                Arc::new(CompiledAnimationGraphEvaluation::default()),
            );
        }

        assert_eq!(
            pipeline.graph_evaluation_cache.len(),
            GRAPH_EVALUATION_FRAME_CACHE_LIMIT
        );
        assert_eq!(
            pipeline
                .graph_evaluation_cache
                .iter()
                .next()
                .map(|cached| cached.graph_id),
            Some(ResourceId::from_stable_label("animation.graph.1"))
        );
        assert_eq!(
            pipeline
                .graph_evaluation_cache
                .iter()
                .next_back()
                .map(|cached| cached.graph_id),
            Some(ResourceId::from_stable_label(&format!(
                "animation.graph.{GRAPH_EVALUATION_FRAME_CACHE_LIMIT}"
            )))
        );
    }

    #[test]
    fn graph_evaluation_frame_cache_source_uses_tail_queue_eviction() {
        let source = include_str!("animation_evaluation_pipeline.rs");
        let tests = source.find("#[cfg(test)]").expect("test module boundary");
        let production = &source[..tests];
        let start = source
            .find("    pub(super) fn cache_graph_evaluation(")
            .expect("graph evaluation cache insertion owner");
        let end = source[start..]
            .find("    pub(super) fn update_presentation_poses(")
            .map(|offset| start + offset)
            .expect("graph evaluation cache insertion boundary");
        let insertion = &source[start..end];

        assert!(production.contains("graph_evaluation_cache: VecDeque<CachedGraphEvaluation>"));
        assert!(insertion.contains("graph_evaluation_cache.pop_front()"));
        assert!(insertion.contains(".push_back(CachedGraphEvaluation"));
        assert!(!insertion.contains("remove(0)"));
    }

    #[test]
    #[ignore = "managed animation frame-cache release performance gate"]
    fn graph_evaluation_frame_cache_tail_queue_release_benchmark_evidence() {
        const OPERATIONS: u64 = 100_000;
        const CAPACITY: usize = GRAPH_EVALUATION_FRAME_CACHE_LIMIT;
        const SAMPLE_PAIRS: usize = 21;

        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut tail_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let mut legacy = Vec::with_capacity(CAPACITY);
                let started = Instant::now();
                for value in 0..OPERATIONS {
                    if legacy.len() >= CAPACITY {
                        black_box(legacy.remove(0));
                    }
                    legacy.push(value);
                }
                black_box(&legacy);
                legacy_samples_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_tail = || {
                let mut tail = VecDeque::with_capacity(CAPACITY);
                let started = Instant::now();
                for value in 0..OPERATIONS {
                    if tail.len() >= CAPACITY {
                        black_box(tail.pop_front());
                    }
                    tail.push_back(value);
                }
                black_box(&tail);
                tail_samples_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_tail();
            } else {
                measure_tail();
                measure_legacy();
            }
        }

        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
        let tail_p95_ns = nearest_rank_percentile(&tail_samples_ns, 95);

        let legacy = legacy_samples_ns
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let tail = tail_samples_ns
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let legacy_moves =
            u128::from(OPERATIONS - CAPACITY as u64).saturating_mul((CAPACITY - 1) as u128);
        println!(
            "ANIMATION_GRAPH_FRAME_CACHE_BENCH_V1 operations={OPERATIONS} capacity={CAPACITY} sample_pairs={SAMPLE_PAIRS} legacy_moves={legacy_moves} tail_moves=0 legacy_p95_ns={legacy_p95_ns} tail_p95_ns={tail_p95_ns} legacy_ns={legacy} tail_ns={tail}"
        );
        assert!(
            tail_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "tail P95 {tail_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        assert!((1..=100).contains(&percentile));
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }

    #[test]
    fn replacement_epoch_retires_pending_diagnostics_from_all_evaluators() {
        let skeleton = AnimationAssetRevision::new(
            ResourceId::from_stable_label("animation.skeleton.replacement-diagnostic"),
            1,
        );
        let clip = AnimationAssetRevision::new(
            ResourceId::from_stable_label("animation.clip.replacement-diagnostic"),
            1,
        );
        let mut pipeline = AnimationEvaluationPipeline::default();
        assert!(pipeline.begin_evaluation_frame(1));
        pipeline.clip_evaluator_mut().record_diagnostic(
            17,
            skeleton,
            clip,
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton.id(),
                clip: clip.id(),
            },
        );
        let mut worker = pipeline.take_direct_clip_worker_evaluator(1);
        worker.record_diagnostic(
            18,
            skeleton,
            clip,
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton.id(),
                clip: clip.id(),
            },
        );
        pipeline.restore_direct_clip_worker_evaluator(1, worker);

        assert!(pipeline.begin_evaluation_frame(2));
        assert_eq!(pipeline.direct_clip_worker_evaluators.len(), 1);
        assert!(
            pipeline
                .drain_clip_evaluation_diagnostics_excluding(&BTreeSet::new())
                .is_empty()
        );

        pipeline.clip_evaluator_mut().record_diagnostic(
            19,
            skeleton,
            clip,
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton.id(),
                clip: clip.id(),
            },
        );
        let current = pipeline.drain_clip_evaluation_diagnostics_excluding(&BTreeSet::new());
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].entity, 19);
    }

    #[test]
    fn deferred_event_entities_restore_state_machine_runtime_checkpoint() {
        let machine = ResourceId::from_stable_label("animation.machine.checkpoint");
        let admitted = MachineInstanceKey::root(17, machine);
        let deferred = MachineInstanceKey::root(18, machine);
        let uncheckpointed_deferred = MachineInstanceKey::root(19, machine);
        let mut pipeline = AnimationEvaluationPipeline::default();
        pipeline
            .nested_machine_states
            .insert(admitted.clone(), "OldA".into());
        pipeline
            .nested_machine_states
            .insert(deferred.clone(), "OldB".into());
        pipeline
            .nested_machine_states
            .insert(uncheckpointed_deferred.clone(), "Paused".into());
        pipeline.nested_machine_transitions.insert(
            deferred.clone(),
            AnimationStateTransitionRuntime {
                from_state: "OldB".into(),
                to_state: "OldC".into(),
                duration_seconds: 1.0,
                elapsed_seconds: 0.25,
                from_time_seconds: 0.25,
                to_time_seconds: 0.0,
            },
        );
        pipeline.record_interrupted_transition_source(deferred.clone(), "OldB", "OldC", pose());
        let checkpoint = pipeline.state_machine_runtime_checkpoint(&BTreeSet::from([17, 18]));

        pipeline
            .nested_machine_states
            .insert(admitted.clone(), "NewA".into());
        pipeline
            .nested_machine_states
            .insert(deferred.clone(), "NewB".into());
        pipeline.nested_machine_transitions.remove(&deferred);
        pipeline.clear_interrupted_transition_source(&deferred);
        pipeline.restore_deferred_state_machine_entities(
            checkpoint,
            &BTreeSet::from([deferred.entity(), uncheckpointed_deferred.entity()]),
        );

        assert_eq!(
            pipeline
                .nested_machine_states
                .get(&admitted)
                .map(String::as_str),
            Some("NewA")
        );
        assert_eq!(
            pipeline
                .nested_machine_states
                .get(&deferred)
                .map(String::as_str),
            Some("OldB")
        );
        assert!(pipeline.nested_machine_transitions.contains_key(&deferred));
        assert!(
            pipeline
                .interrupted_transition_source(&deferred, "OldB", "OldC")
                .is_some()
        );
        assert_eq!(
            pipeline
                .nested_machine_states
                .get(&uncheckpointed_deferred)
                .map(String::as_str),
            Some("Paused")
        );
    }
}
