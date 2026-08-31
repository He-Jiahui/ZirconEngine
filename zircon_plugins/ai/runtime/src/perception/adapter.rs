use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiHearingStimulusEvent, AiPerceptionSense, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::framework::sound::SoundGameplayEmission;
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::scene::ecs::Resource;
use zircon_runtime::scene::World;

pub const AI_HEARING_ANIMATION_EVENT_NAME: &str = "ai.hearing";
pub const AI_HEARING_PENDING_EVENT_CAPACITY: usize = 1_024;
pub const AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS: Real = 5.0;
pub const AI_HEARING_INGEST_EVENT_LIMIT: usize = 1_024;

type Receiver = (EntityId, Vec3, Real, Real);

#[derive(Clone, Debug)]
struct PendingHearingEvent {
    event: AiHearingStimulusEvent,
    receiver_ids: Arc<[EntityId]>,
    next_receiver: usize,
}

#[derive(Debug, Default)]
pub struct HearingStimulusAdapter {
    pending: VecDeque<PendingHearingEvent>,
    receiver_index: HashMap<EntityId, Receiver>,
    dropped_events: u64,
    expired_events: u64,
}

impl Resource for HearingStimulusAdapter {}

impl HearingStimulusAdapter {
    pub fn pending_event_count(&self) -> usize {
        self.pending.len()
    }

    pub fn dropped_event_count(&self) -> u64 {
        self.dropped_events
    }

    pub fn expired_event_count(&self) -> u64 {
        self.expired_events
    }

    pub fn pending_receiver_snapshot_count(&self) -> usize {
        self.pending
            .iter()
            .map(|pending| Arc::as_ptr(&pending.receiver_ids) as *const () as usize)
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub(crate) fn clear_pending(&mut self) {
        self.pending.clear();
    }

    pub(crate) fn record_dropped_events(&mut self, count: u64) {
        self.dropped_events = self.dropped_events.saturating_add(count);
    }

    pub(crate) fn advance_time(&mut self, delta_seconds: Real) {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        for pending in &mut self.pending {
            pending.event.age_seconds += delta_seconds;
        }
        let before = self.pending.len();
        self.pending
            .retain(|pending| pending.event.age_seconds < AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS);
        self.expired_events += before.saturating_sub(self.pending.len()) as u64;
    }

    pub(crate) fn enqueue(
        &mut self,
        events: impl IntoIterator<Item = AiHearingStimulusEvent>,
        receivers: &[Receiver],
    ) {
        let mut receiver_ids = None;
        let mut events = events.into_iter();
        for event in events.by_ref().take(AI_HEARING_INGEST_EVENT_LIMIT) {
            if !event.age_seconds.is_finite()
                || event.age_seconds < 0.0
                || event.age_seconds >= AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS
            {
                self.expired_events += 1;
                continue;
            }
            if self.pending.len() == AI_HEARING_PENDING_EVENT_CAPACITY {
                self.pending.pop_front();
                self.dropped_events += 1;
            }
            let receiver_ids = receiver_ids.get_or_insert_with(|| {
                receivers
                    .iter()
                    .map(|receiver| receiver.0)
                    .collect::<Arc<[_]>>()
            });
            self.pending.push_back(PendingHearingEvent {
                event,
                receiver_ids: Arc::clone(receiver_ids),
                next_receiver: 0,
            });
        }
        if events.next().is_some() {
            self.record_dropped_events(1_u64.saturating_add(events.size_hint().0 as u64));
        }
    }

    pub(crate) fn process_budgeted(
        &mut self,
        receivers: &[Receiver],
        pair_limit: usize,
        mut try_consume: impl FnMut() -> bool,
        mut on_stimulus: impl FnMut(EntityId, AiPerceptionStimulus),
    ) -> HearingAdapterReport {
        let mut report = HearingAdapterReport::default();
        if pair_limit == 0 || self.pending.is_empty() {
            return report;
        }
        self.receiver_index.clear();
        self.receiver_index.extend(
            receivers
                .iter()
                .copied()
                .map(|receiver| (receiver.0, receiver)),
        );
        while report.processed_pairs < pair_limit {
            let Some(mut pending) = self.pending.pop_front() else {
                break;
            };
            while pending.next_receiver < pending.receiver_ids.len()
                && report.processed_pairs < pair_limit
            {
                let receiver_id = pending.receiver_ids[pending.next_receiver];
                pending.next_receiver += 1;
                let Some(receiver) = self.receiver_index.get(&receiver_id).copied() else {
                    continue;
                };
                if !try_consume() {
                    pending.next_receiver -= 1;
                    self.pending.push_front(pending);
                    return report;
                }
                report.processed_pairs += 1;
                if let Some(stimulus) = hearing_stimulus_for_receiver(receiver, &pending.event) {
                    on_stimulus(receiver.0, stimulus);
                    report.refreshed_stimuli += 1;
                }
            }
            if pending.next_receiver < pending.receiver_ids.len() {
                self.pending.push_front(pending);
                break;
            }
        }
        report
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HearingAdapterReport {
    pub processed_pairs: usize,
    pub refreshed_stimuli: usize,
}

pub(crate) fn hearing_event_from_animation(
    world: &World,
    event: &AnimationEventRecord,
) -> Option<AiHearingStimulusEvent> {
    if event.name != AI_HEARING_ANIMATION_EVENT_NAME {
        return None;
    }
    let position = world.world_transform(event.entity)?.translation;
    let strength = event
        .payload
        .as_deref()
        .and_then(|payload| payload.parse::<Real>().ok())
        .filter(|strength| strength.is_finite() && *strength > 0.0)
        .unwrap_or(1.0);
    Some(AiHearingStimulusEvent::animation_event(
        event.entity,
        position,
        strength,
    ))
}

pub(crate) fn hearing_event_from_sound(
    event: &SoundGameplayEmission,
    now_seconds: f64,
) -> Option<AiHearingStimulusEvent> {
    let position = Vec3::from_array(event.position);
    if !position.is_finite() || !event.strength.is_finite() || event.strength <= 0.0 {
        return None;
    }
    let age_seconds = (now_seconds - event.emitted_at_seconds).max(0.0) as Real;
    let mut hearing =
        AiHearingStimulusEvent::sound_playback(event.source, position, event.strength)
            .with_age_seconds(age_seconds);
    hearing.max_range = event.max_range;
    Some(hearing)
}

fn hearing_stimulus_for_receiver(
    receiver: (EntityId, Vec3, Real, Real),
    event: &AiHearingStimulusEvent,
) -> Option<AiPerceptionStimulus> {
    if !event.position.is_finite() || !event.strength.is_finite() || event.strength <= 0.0 {
        return None;
    }
    if !event.age_seconds.is_finite()
        || event.age_seconds < 0.0
        || !receiver.3.is_finite()
        || event.age_seconds >= receiver.3.max(0.0)
    {
        return None;
    }
    let hearing_radius = effective_radius(receiver.2, event.max_range)?;
    let in_range = event.position.distance_squared(receiver.1) <= hearing_radius * hearing_radius;
    in_range.then(|| AiPerceptionStimulus {
        source: event.source,
        sense: AiPerceptionSense::Hearing,
        position: event.position,
        strength: event.strength,
        age_seconds: event.age_seconds,
    })
}

fn effective_radius(receiver_radius: Real, event_radius: Option<Real>) -> Option<Real> {
    if !receiver_radius.is_finite() || receiver_radius <= 0.0 {
        return None;
    }
    match event_radius {
        Some(radius) if radius.is_finite() && radius > 0.0 => Some(receiver_radius.min(radius)),
        Some(_) => None,
        None => Some(receiver_radius),
    }
}

#[cfg(test)]
mod allocation_tests;
