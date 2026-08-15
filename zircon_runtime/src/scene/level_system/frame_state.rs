use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(feature = "animation")]
use std::collections::VecDeque;

use crate::core::framework::animation::AnimationPoseOutput;
#[cfg(feature = "animation")]
use crate::core::framework::animation::{
    AnimationClipEventBatchAdmission, AnimationClipEventSamplingCursor,
    AnimationClipEventSamplingRange,
};
use crate::core::math::Real;
#[cfg(feature = "animation")]
use crate::core::resource::ResourceId;
use crate::scene::EntityId;

#[cfg(feature = "animation")]
use super::AnimationStateTransitionRuntime;

#[cfg(feature = "animation")]
#[derive(Clone, Debug)]
pub(super) struct AnimationRuntimeState {
    pub(super) animation_requires_continuous_frame: bool,
    pub(super) animation_event_backlog_requires_continuous_frame: bool,
    pub(super) playback_state: Arc<AnimationPlaybackStateSnapshot>,
    clip_event_samples: VecDeque<PendingAnimationClipEventSample>,
    overflowed_clip_event_sample_count: usize,
    last_clip_event_drain: AnimationClipEventDrainMetrics,
}

#[cfg(feature = "animation")]
impl Default for AnimationRuntimeState {
    fn default() -> Self {
        Self {
            animation_requires_continuous_frame: false,
            animation_event_backlog_requires_continuous_frame: false,
            playback_state: Arc::new(AnimationPlaybackStateSnapshot::default()),
            clip_event_samples: VecDeque::new(),
            overflowed_clip_event_sample_count: 0,
            last_clip_event_drain: AnimationClipEventDrainMetrics::default(),
        }
    }
}

#[cfg(feature = "animation")]
impl AnimationRuntimeState {
    pub(super) fn reset_after_world_replacement(&mut self) {
        self.animation_requires_continuous_frame = false;
        self.animation_event_backlog_requires_continuous_frame = false;
        self.playback_state = Arc::new(self.playback_state.cleared());
        self.clip_event_samples.clear();
        self.overflowed_clip_event_sample_count = 0;
        self.last_clip_event_drain = AnimationClipEventDrainMetrics::default();
    }

    pub(super) fn enqueue_clip_event_sample_batches(
        &mut self,
        max_pending_samples: usize,
        batches: Vec<Vec<AnimationClipEventSamplingRange>>,
    ) -> (Vec<AnimationClipEventBatchAdmission>, usize, usize, usize) {
        let mut remaining_capacity =
            max_pending_samples.saturating_sub(self.clip_event_samples.len());
        let mut batch_admissions = Vec::with_capacity(batches.len());
        let mut admitted_range_count = 0;
        let mut deferred_range_count = 0usize;
        let mut rejected_range_count = 0usize;
        let mut defer_remaining = false;
        for batch in batches {
            let range_count = batch.len();
            if range_count > max_pending_samples {
                rejected_range_count = rejected_range_count.saturating_add(range_count);
                batch_admissions.push(AnimationClipEventBatchAdmission::RejectedOversized {
                    range_count,
                    capacity: max_pending_samples,
                });
                continue;
            }
            if defer_remaining || range_count > remaining_capacity {
                defer_remaining = true;
                deferred_range_count = deferred_range_count.saturating_add(range_count);
                batch_admissions.push(AnimationClipEventBatchAdmission::Deferred);
                continue;
            }
            remaining_capacity -= range_count;
            admitted_range_count += range_count;
            batch_admissions.push(AnimationClipEventBatchAdmission::Admitted);
            for range in batch {
                self.clip_event_samples
                    .push_back(PendingAnimationClipEventSample {
                        entity: range.entity,
                        clip_id: range.clip_id,
                        from_time_seconds: range.from_time_seconds,
                        to_time_seconds: range.to_time_seconds,
                        looping: range.looping,
                        cursor: AnimationClipEventSamplingCursor::at_range_start(
                            range.from_time_seconds,
                        ),
                        age_drain_windows: 0,
                    });
            }
        }
        self.overflowed_clip_event_sample_count = self
            .overflowed_clip_event_sample_count
            .saturating_add(deferred_range_count)
            .saturating_add(rejected_range_count);
        self.animation_event_backlog_requires_continuous_frame =
            !self.clip_event_samples.is_empty() || deferred_range_count > 0;
        (
            batch_admissions,
            admitted_range_count,
            deferred_range_count,
            rejected_range_count,
        )
    }

    pub(super) fn begin_clip_event_drain(&mut self, max_samples: usize) -> (usize, usize) {
        let pending_sample_count = self.clip_event_samples.len().min(max_samples);
        for sample in &mut self.clip_event_samples {
            sample.age_drain_windows = sample.age_drain_windows.saturating_add(1);
        }
        (
            pending_sample_count,
            std::mem::take(&mut self.overflowed_clip_event_sample_count),
        )
    }

    pub(super) fn take_clip_event_sample(&mut self) -> Option<PendingAnimationClipEventSample> {
        self.clip_event_samples.pop_front()
    }

    pub(super) fn requeue_clip_event_sample_back(
        &mut self,
        sample: PendingAnimationClipEventSample,
    ) {
        self.clip_event_samples.push_back(sample);
    }

    pub(super) fn clip_event_backlog(&self) -> (usize, u64) {
        (
            self.clip_event_samples.len(),
            self.clip_event_samples
                .iter()
                .map(|sample| sample.age_drain_windows)
                .max()
                .unwrap_or(0),
        )
    }

    pub(super) fn record_clip_event_drain(&mut self, metrics: AnimationClipEventDrainMetrics) {
        self.animation_event_backlog_requires_continuous_frame =
            !self.clip_event_samples.is_empty();
        self.last_clip_event_drain = metrics;
    }

    pub(super) fn last_clip_event_drain(&self) -> AnimationClipEventDrainMetrics {
        self.last_clip_event_drain
    }
}

#[cfg(feature = "animation")]
#[derive(Clone, Debug)]
pub(super) struct PendingAnimationClipEventSample {
    pub(super) entity: EntityId,
    pub(super) clip_id: ResourceId,
    pub(super) from_time_seconds: Real,
    pub(super) to_time_seconds: Real,
    pub(super) looping: bool,
    pub(super) cursor: AnimationClipEventSamplingCursor,
    pub(super) age_drain_windows: u64,
}

#[cfg(feature = "animation")]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AnimationClipEventDrainMetrics {
    pub(crate) deferred_range_count: usize,
    pub(crate) oldest_pending_age_drain_windows: u64,
    pub(crate) budget_exhausted: bool,
    pub(crate) oversized_event_count: usize,
    pub(crate) unavailable_asset_count: usize,
    pub(crate) overflowed_sample_count: usize,
}

/// Immutable playback state shared by one animation scan generation.
///
/// The scan takes one `Arc` handle before entering the World lane. It can then read prior
/// graph/state-machine values without cloning the full maps or nesting the two domain locks.
#[cfg(feature = "animation")]
#[derive(Clone, Debug, Default)]
pub(crate) struct AnimationPlaybackStateSnapshot {
    generation: u64,
    animation_graph_times: Arc<BTreeMap<EntityId, Real>>,
    animation_state_machine_times: Arc<BTreeMap<EntityId, Real>>,
    animation_state_machine_transitions: Arc<BTreeMap<EntityId, AnimationStateTransitionRuntime>>,
}

#[cfg(feature = "animation")]
impl AnimationPlaybackStateSnapshot {
    fn cleared(&self) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            animation_graph_times: Arc::new(BTreeMap::new()),
            animation_state_machine_times: Arc::new(BTreeMap::new()),
            animation_state_machine_transitions: Arc::new(BTreeMap::new()),
        }
    }

    pub(super) fn with_values(
        &self,
        animation_graph_times: BTreeMap<EntityId, Real>,
        animation_state_machine_times: BTreeMap<EntityId, Real>,
        animation_state_machine_transitions: BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    ) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            animation_graph_times: Arc::new(animation_graph_times),
            animation_state_machine_times: Arc::new(animation_state_machine_times),
            animation_state_machine_transitions: Arc::new(animation_state_machine_transitions),
        }
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn animation_graph_times(&self) -> &Arc<BTreeMap<EntityId, Real>> {
        &self.animation_graph_times
    }

    pub(crate) fn animation_state_machine_times(&self) -> &Arc<BTreeMap<EntityId, Real>> {
        &self.animation_state_machine_times
    }

    pub(crate) fn animation_state_machine_transitions(
        &self,
    ) -> &Arc<BTreeMap<EntityId, AnimationStateTransitionRuntime>> {
        &self.animation_state_machine_transitions
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct ScriptRuntimeState {
    generation: u64,
    started_bindings: BTreeMap<EntityId, BTreeSet<Box<str>>>,
}

impl ScriptRuntimeState {
    pub(super) fn contains(&self, entity: EntityId, binding_key: &str) -> bool {
        self.started_bindings
            .get(&entity)
            .is_some_and(|bindings| bindings.contains(binding_key))
    }

    pub(super) fn insert(&mut self, entity: EntityId, binding_key: String) {
        if self
            .started_bindings
            .entry(entity)
            .or_default()
            .insert(binding_key.into_boxed_str())
        {
            self.generation = self.generation.saturating_add(1);
        }
    }

    pub(super) fn reset_after_world_replacement(&mut self) {
        self.generation = self.generation.saturating_add(1);
        self.started_bindings.clear();
    }

    #[cfg(test)]
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }
}

/// Immutable animation payload published at a LevelSystem frame boundary.
///
/// Render extraction clones this handle before it locks the World, so an animation write never
/// requires the extract path to clone or retain the mutable animation lane.
#[derive(Clone, Debug)]
pub(crate) struct LevelFrameStateSnapshot {
    world_generation: u64,
    animation_generation: u64,
    animation_poses: Arc<BTreeMap<EntityId, AnimationPoseOutput>>,
}

impl LevelFrameStateSnapshot {
    pub(super) fn new(world_generation: u64) -> Self {
        Self {
            world_generation,
            animation_generation: 0,
            animation_poses: Arc::new(BTreeMap::new()),
        }
    }

    pub(crate) fn world_generation(&self) -> u64 {
        self.world_generation
    }

    pub(crate) fn animation_generation(&self) -> u64 {
        self.animation_generation
    }

    pub(crate) fn animation_poses(&self) -> &Arc<BTreeMap<EntityId, AnimationPoseOutput>> {
        &self.animation_poses
    }

    pub(super) fn with_animation_poses(
        &self,
        world_generation: u64,
        animation_poses: Arc<BTreeMap<EntityId, AnimationPoseOutput>>,
    ) -> Self {
        Self {
            world_generation,
            animation_generation: self.animation_generation.saturating_add(1),
            animation_poses,
        }
    }
}
