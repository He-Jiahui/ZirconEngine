use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(feature = "animation")]
use std::collections::VecDeque;

#[cfg(feature = "animation")]
use crate::animation::AnimationClipEventSamplingCursor;
#[cfg(feature = "animation")]
use crate::asset::AssetId;
use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::math::Real;
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
        self.last_clip_event_drain = AnimationClipEventDrainMetrics::default();
    }

    pub(super) fn enqueue_clip_event_sample(
        &mut self,
        entity: EntityId,
        clip_id: AssetId,
        from_time_seconds: Real,
        to_time_seconds: Real,
        looping: bool,
    ) {
        self.clip_event_samples
            .push_back(PendingAnimationClipEventSample {
                entity,
                clip_id,
                from_time_seconds,
                to_time_seconds,
                looping,
                cursor: AnimationClipEventSamplingCursor::at_range_start(from_time_seconds),
                age_frames: 0,
            });
    }

    pub(super) fn take_clip_event_sample(&mut self) -> Option<PendingAnimationClipEventSample> {
        self.clip_event_samples.pop_front()
    }

    pub(super) fn requeue_clip_event_sample_front(
        &mut self,
        sample: PendingAnimationClipEventSample,
    ) {
        self.clip_event_samples.push_front(sample);
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
                .map(|sample| sample.age_frames)
                .max()
                .unwrap_or(0),
        )
    }

    pub(super) fn record_clip_event_drain(&mut self, metrics: AnimationClipEventDrainMetrics) {
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
    pub(super) clip_id: AssetId,
    pub(super) from_time_seconds: Real,
    pub(super) to_time_seconds: Real,
    pub(super) looping: bool,
    pub(super) cursor: AnimationClipEventSamplingCursor,
    pub(super) age_frames: u64,
}

#[cfg(feature = "animation")]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AnimationClipEventDrainMetrics {
    pub(crate) deferred_range_count: usize,
    pub(crate) oldest_pending_age_frames: u64,
    pub(crate) budget_exhausted: bool,
    pub(crate) oversized_event_count: usize,
    pub(crate) unavailable_asset_count: usize,
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
