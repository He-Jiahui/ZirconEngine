use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, MutexGuard};

use crate::core::framework::animation::{
    AnimationClipEvent, AnimationClipEventQueueAdmission, AnimationClipEventSampler,
    AnimationClipEventSamplingLimits, AnimationClipEventSamplingRange,
    AnimationClipEventSamplingRequest, AnimationPoseSnapshot,
};
use crate::core::math::Real;
use crate::scene::EntityId;

use super::frame_state::{
    AnimationClipEventDrainMetrics, AnimationPlaybackStateSnapshot, AnimationRuntimeState,
    LevelFrameStateSnapshot,
};
use super::{AnimationStateTransitionRuntime, LevelSystem};

#[cfg(test)]
#[path = "tests/drain_output_capacity.rs"]
mod drain_output_capacity_tests;

const ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES: usize = 32;
const ANIMATION_CLIP_EVENT_RETAINED_DRAIN_WINDOWS: usize = 8;
const ANIMATION_CLIP_EVENT_MAX_PENDING_SAMPLES: usize =
    ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES * ANIMATION_CLIP_EVENT_RETAINED_DRAIN_WINDOWS;

impl LevelSystem {
    fn lock_animation_state_if_replacement_epoch(
        &self,
        replacement_epoch: u64,
    ) -> Option<MutexGuard<'_, AnimationRuntimeState>> {
        let state = self.lock_animation_state();
        (self.world_replacement_epoch.load(Ordering::Acquire) == replacement_epoch).then_some(state)
    }

    /// Combines explicit host demand, committed evaluator demand, and pending events.
    pub fn animation_requires_continuous_frame(&self) -> bool {
        let state = self.lock_animation_state();
        state.animation_requires_continuous_frame
            || (state.evaluated_animation_requires_continuous_frame
                && state.evaluated_animation_replacement_epoch
                    == self.world_replacement_epoch.load(Ordering::Acquire))
            || state.animation_event_backlog_requires_continuous_frame
    }

    /// Sets the explicit host channel independently of frame-scoped evaluators.
    pub fn record_animation_requires_continuous_frame(&self, requires_continuous_frame: bool) {
        self.lock_animation_state()
            .animation_requires_continuous_frame = requires_continuous_frame;
    }

    /// Captures the current tick's identity before an evaluator starts work.
    pub fn capture_animation_evaluation_frame(&self) -> Option<(u64, u64)> {
        let state = self.lock_animation_state();
        let (epoch, frame, _) = state.pending_animation_evaluation_frame?;
        (epoch == self.world_replacement_epoch.load(Ordering::Acquire)).then_some((epoch, frame))
    }

    /// Adds one producer's result; only the entire successful tick can publish it.
    pub fn stage_animation_evaluation_frame(
        &self,
        replacement_epoch: u64,
        frame: u64,
        requires_continuous_frame: bool,
    ) -> bool {
        let Some(mut state) = self.lock_animation_state_if_replacement_epoch(replacement_epoch)
        else {
            return false;
        };
        let Some(pending) = state.pending_animation_evaluation_frame.as_mut() else {
            return false;
        };
        if (pending.0, pending.1) != (replacement_epoch, frame) {
            return false;
        }
        pending.2 |= requires_continuous_frame;
        true
    }

    pub(crate) fn begin_animation_evaluation_frame(&self) -> AnimationEvaluationFrame<'_> {
        let identity = {
            let mut state = self.lock_animation_state();
            match state.animation_evaluation_frame_sequence.checked_add(1) {
                Some(frame) => {
                    let epoch = self.world_replacement_epoch.load(Ordering::Acquire);
                    state.animation_evaluation_frame_sequence = frame;
                    state.pending_animation_evaluation_frame = Some((epoch, frame, false));
                    Some((epoch, frame))
                }
                None => {
                    // Exhaustion retires this channel instead of recycling a producer token.
                    state.pending_animation_evaluation_frame = None;
                    state.evaluated_animation_requires_continuous_frame = false;
                    None
                }
            }
        };
        AnimationEvaluationFrame {
            level: self,
            identity,
        }
    }

    pub(crate) fn animation_playback_snapshot(&self) -> Arc<AnimationPlaybackStateSnapshot> {
        Arc::clone(&self.lock_animation_state().playback_state)
    }

    pub fn animation_playback_times(
        &self,
        replacement_epoch: u64,
    ) -> Option<(
        Arc<BTreeMap<EntityId, Real>>,
        Arc<BTreeMap<EntityId, Real>>,
        Arc<BTreeMap<EntityId, AnimationStateTransitionRuntime>>,
    )> {
        let state = self.lock_animation_state_if_replacement_epoch(replacement_epoch)?;
        let snapshot = &state.playback_state;
        Some((
            Arc::clone(snapshot.animation_graph_times()),
            Arc::clone(snapshot.animation_state_machine_times()),
            Arc::clone(snapshot.animation_state_machine_transitions()),
        ))
    }

    /// Admits the largest complete prefix of producer batches for the current replacement epoch.
    ///
    /// Ranges within one producer batch remain atomic. Capacity-deferred batches retry from
    /// unchanged playback state. A batch larger than the queue's absolute capacity is rejected
    /// terminally so it cannot permanently block later producers.
    pub fn enqueue_animation_clip_event_range_batches(
        &self,
        replacement_epoch: u64,
        batches: Vec<Vec<AnimationClipEventSamplingRange>>,
    ) -> AnimationClipEventQueueAdmission {
        let Some(mut state) = self.lock_animation_state_if_replacement_epoch(replacement_epoch)
        else {
            return AnimationClipEventQueueAdmission::RetiredEpoch;
        };
        let (batch_admissions, admitted_range_count, deferred_range_count, rejected_range_count) =
            state.enqueue_clip_event_sample_batches(
                ANIMATION_CLIP_EVENT_MAX_PENDING_SAMPLES,
                batches,
            );
        AnimationClipEventQueueAdmission::Current {
            batch_admissions,
            admitted_range_count,
            deferred_range_count,
            rejected_range_count,
        }
    }

    pub fn drain_animation_clip_events(
        &self,
        replacement_epoch: u64,
        sampler: &dyn AnimationClipEventSampler,
    ) -> Option<Vec<AnimationClipEvent>> {
        let limits = AnimationClipEventSamplingLimits::default();
        let mut metrics = AnimationClipEventDrainMetrics::default();
        let pending_sample_count = {
            let mut state = self.lock_animation_state_if_replacement_epoch(replacement_epoch)?;
            let (pending_sample_count, overflowed_sample_count) =
                state.begin_clip_event_drain(ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES);
            metrics.overflowed_sample_count = overflowed_sample_count;
            pending_sample_count
        };
        let mut events = if pending_sample_count == 0 {
            Vec::new()
        } else {
            Vec::with_capacity(limits.max_events)
        };
        let mut emitted_event_bytes: usize = 0;

        for _ in 0..pending_sample_count {
            let Some(mut pending) = self
                .lock_animation_state_if_replacement_epoch(replacement_epoch)?
                .take_clip_event_sample()
            else {
                break;
            };
            let Some(remaining_event_bytes) =
                limits.max_event_bytes.checked_sub(emitted_event_bytes)
            else {
                self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            };
            if remaining_event_bytes == 0 {
                self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            }
            let remaining_events = limits.max_events.saturating_sub(events.len());
            if remaining_events == 0 {
                self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            }
            let Some(batch) = sampler.sample_clip_events(AnimationClipEventSamplingRequest {
                entity: pending.entity,
                clip_id: pending.clip_id,
                from_time_seconds: pending.from_time_seconds,
                to_time_seconds: pending.to_time_seconds,
                looping: pending.looping,
                cursor: pending.cursor.clone(),
                limits: AnimationClipEventSamplingLimits {
                    max_events: remaining_events,
                    max_event_bytes: remaining_event_bytes,
                    max_playback_span_seconds: limits.max_playback_span_seconds,
                },
            }) else {
                self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                    .requeue_clip_event_sample_back(pending);
                metrics.unavailable_asset_count = metrics.unavailable_asset_count.saturating_add(1);
                continue;
            };
            emitted_event_bytes = emitted_event_bytes.saturating_add(batch.emitted_event_bytes);
            metrics.oversized_event_count = metrics
                .oversized_event_count
                .saturating_add(batch.oversized_event_count);
            metrics.budget_exhausted |= batch.budget_exhausted;
            events.extend(batch.events);
            if let Some(cursor) = batch.next_cursor {
                pending.cursor = cursor;
                self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                    .requeue_clip_event_sample_front(pending);
                // Finish a range's resumable portion before visiting later playback ranges.
                // Otherwise a direction change can overtake events still pending in this range.
                break;
            }
        }

        let mut state = self.lock_animation_state_if_replacement_epoch(replacement_epoch)?;
        let (deferred_range_count, oldest_pending_age_drain_windows) = state.clip_event_backlog();
        metrics.deferred_range_count = deferred_range_count;
        metrics.oldest_pending_age_drain_windows = oldest_pending_age_drain_windows;
        state.record_clip_event_drain(metrics);
        Some(events)
    }

    pub fn animation_clip_event_backlog_len(&self, replacement_epoch: u64) -> Option<usize> {
        Some(
            self.lock_animation_state_if_replacement_epoch(replacement_epoch)?
                .clip_event_backlog()
                .0,
        )
    }

    pub(crate) fn animation_clip_event_drain_metrics(
        &self,
    ) -> (usize, u64, bool, usize, usize, usize) {
        let metrics = self.lock_animation_state().last_clip_event_drain();
        (
            metrics.deferred_range_count,
            metrics.oldest_pending_age_drain_windows,
            metrics.budget_exhausted,
            metrics.oversized_event_count,
            metrics.unavailable_asset_count,
            metrics.overflowed_sample_count,
        )
    }

    /// Publishes an immutable animation pose snapshot for the current replacement epoch.
    ///
    /// Ordinary component mutations do not retire the producer. The payload is stamped with the
    /// World mutation generation observed while the World lane remains locked.
    pub fn record_animation_pose_snapshot(
        &self,
        replacement_epoch: u64,
        animation_poses: AnimationPoseSnapshot,
    ) -> bool {
        let published = self.frame_state_snapshot();
        let published_matches_snapshot = Arc::ptr_eq(published.animation_poses(), &animation_poses);
        let world = self.lock_world();
        if self.world_replacement_epoch.load(Ordering::Acquire) != replacement_epoch {
            return false;
        }
        let world_generation = world.world_generation();
        let mut current = self.lock_frame_state();
        if published_matches_snapshot
            && published.world_generation() == world_generation
            && Arc::ptr_eq(&published, &current)
        {
            return true;
        }
        if current.world_generation() == world_generation
            && Arc::ptr_eq(current.animation_poses(), &animation_poses)
        {
            return true;
        }

        Self::publish_animation_frame(&mut current, world_generation, animation_poses);
        true
    }

    pub fn record_animation_playback_times(
        &self,
        replacement_epoch: u64,
        animation_graph_times: BTreeMap<EntityId, Real>,
        animation_state_machine_times: BTreeMap<EntityId, Real>,
        animation_state_machine_transitions: BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    ) -> bool {
        let Some(mut animation_state) =
            self.lock_animation_state_if_replacement_epoch(replacement_epoch)
        else {
            return false;
        };
        let published = &animation_state.playback_state;
        if published.animation_graph_times().as_ref() == &animation_graph_times
            && published.animation_state_machine_times().as_ref() == &animation_state_machine_times
            && published.animation_state_machine_transitions().as_ref()
                == &animation_state_machine_transitions
        {
            return true;
        }

        animation_state.playback_state = Arc::new(published.with_values(
            animation_graph_times,
            animation_state_machine_times,
            animation_state_machine_transitions,
        ));
        true
    }

    pub(super) fn publish_animation_frame(
        frame_state: &mut Arc<LevelFrameStateSnapshot>,
        world_generation: u64,
        animation_poses: AnimationPoseSnapshot,
    ) {
        *frame_state =
            Arc::new(frame_state.with_animation_poses(world_generation, animation_poses));
    }
}

pub(crate) struct AnimationEvaluationFrame<'a> {
    level: &'a LevelSystem,
    identity: Option<(u64, u64)>,
}

impl AnimationEvaluationFrame<'_> {
    pub(crate) fn commit(mut self) {
        self.finish(true);
    }

    fn finish(&mut self, succeeded: bool) {
        let Some(identity) = self.identity.take() else {
            return;
        };
        let mut state = self.level.lock_animation_state();
        let Some((epoch, frame, requires_continuous_frame)) =
            state.pending_animation_evaluation_frame
        else {
            return;
        };
        if identity != (epoch, frame) {
            return;
        }
        state.pending_animation_evaluation_frame = None;
        state.evaluated_animation_replacement_epoch = epoch;
        state.evaluated_animation_requires_continuous_frame = succeeded
            && epoch == self.level.world_replacement_epoch.load(Ordering::Acquire)
            && requires_continuous_frame;
    }
}

impl Drop for AnimationEvaluationFrame<'_> {
    fn drop(&mut self) {
        // The caller has released callback and World guards before this cleanup.
        self.finish(false);
    }
}
