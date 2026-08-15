use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::animation::ProjectAnimationClipEventSampler;
use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::{
    AnimationClipEvent, AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission,
    AnimationClipEventSamplingRange, AnimationEventRecord,
};
use zircon_runtime::scene::LevelSystem;

use super::requests::PendingClipEventSample;

pub(super) struct ClipEventAdmission {
    pub(super) deferred_entities: BTreeSet<zircon_runtime::scene::EntityId>,
    pub(super) next_cursor: Option<zircon_runtime::scene::EntityId>,
    pub(super) diagnostics: Vec<AnimationEventRecord>,
}

pub(super) fn enqueue_clip_event_samples(
    level: &LevelSystem,
    replacement_epoch: u64,
    cursor: Option<zircon_runtime::scene::EntityId>,
    pending_samples: Vec<PendingClipEventSample>,
) -> Option<ClipEventAdmission> {
    let mut by_entity = BTreeMap::<_, Vec<_>>::new();
    for pending in pending_samples {
        by_entity
            .entry(pending.entity)
            .or_default()
            .push(AnimationClipEventSamplingRange {
                entity: pending.entity,
                clip_id: pending.clip_id,
                from_time_seconds: pending.from_time_seconds,
                to_time_seconds: pending.to_time_seconds,
                looping: pending.looping,
            });
    }
    let mut batches = by_entity.into_iter().collect::<Vec<_>>();
    if let Some(cursor) = cursor {
        let first_after_cursor = batches.partition_point(|(entity, _)| *entity <= cursor);
        batches.rotate_left(first_after_cursor);
    }
    let entities = batches
        .iter()
        .map(|(entity, _)| *entity)
        .collect::<Vec<_>>();
    let admission = level.enqueue_animation_clip_event_range_batches(
        replacement_epoch,
        batches.into_iter().map(|(_, ranges)| ranges).collect(),
    );
    let AnimationClipEventQueueAdmission::Current {
        batch_admissions, ..
    } = admission
    else {
        return None;
    };
    debug_assert_eq!(entities.len(), batch_admissions.len());
    let mut deferred_entities = BTreeSet::new();
    let mut diagnostics = Vec::new();
    let mut next_cursor = cursor;
    for (entity, batch_admission) in entities.into_iter().zip(batch_admissions) {
        match batch_admission {
            AnimationClipEventBatchAdmission::Admitted => next_cursor = Some(entity),
            AnimationClipEventBatchAdmission::Deferred => {
                deferred_entities.insert(entity);
            }
            AnimationClipEventBatchAdmission::RejectedOversized {
                range_count,
                capacity,
            } => diagnostics.push(clip_event_batch_capacity_diagnostic(
                entity,
                range_count,
                capacity,
            )),
        }
    }
    Some(ClipEventAdmission {
        deferred_entities,
        next_cursor,
        diagnostics,
    })
}

fn clip_event_batch_capacity_diagnostic(
    entity: zircon_runtime::scene::EntityId,
    range_count: usize,
    capacity: usize,
) -> AnimationEventRecord {
    AnimationEventRecord::new(entity, "animation.clip_event_batch_capacity_exceeded")
        .with_payload(format!("range_count={range_count};capacity={capacity}"))
}

pub(super) fn publish_clip_events(
    asset_manager: &ProjectAssetManager,
    level: &LevelSystem,
    replacement_epoch: u64,
) -> bool {
    let sampler = ProjectAnimationClipEventSampler::new(asset_manager);
    let Some(events) = level.drain_animation_clip_events(replacement_epoch, &sampler) else {
        return false;
    };
    if !publish_events(
        level,
        replacement_epoch,
        events
            .iter()
            .map(animation_event_record)
            .collect::<Vec<_>>(),
    ) {
        return false;
    }
    publish_events(level, replacement_epoch, events)
}

fn animation_event_record(event: &AnimationClipEvent) -> AnimationEventRecord {
    AnimationEventRecord {
        entity: event.entity,
        clip: None,
        target_id: event.target_id.clone(),
        name: event.event.clone(),
        payload: event.payload.clone(),
        clip_time_seconds: event.clip_time_seconds,
        playback_time_seconds: event.playback_time_seconds,
    }
}

pub(super) fn publish_events<E>(level: &LevelSystem, replacement_epoch: u64, events: Vec<E>) -> bool
where
    E: zircon_runtime::scene::ecs::Event,
{
    level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            publish_events_to_world(world, events)
        })
        .is_some()
}

fn publish_events_to_world<E>(world: &mut zircon_runtime::scene::World, events: Vec<E>)
where
    E: zircon_runtime::scene::ecs::Event,
{
    for event in events {
        world.send_event(event);
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::animation::AnimationClipEvent;

    use super::{animation_event_record, clip_event_batch_capacity_diagnostic};

    #[test]
    fn clip_events_publish_neutral_animation_records() {
        let record = animation_event_record(&AnimationClipEvent {
            entity: 17,
            target_id: Some("left_foot".to_string()),
            event: "ai.hearing".to_string(),
            payload: Some("0.75".to_string()),
            clip_time_seconds: 0.2,
            playback_time_seconds: 1.2,
        });

        assert_eq!(record.entity, 17);
        assert_eq!(record.target_id.as_deref(), Some("left_foot"));
        assert_eq!(record.name, "ai.hearing");
        assert_eq!(record.payload.as_deref(), Some("0.75"));
        assert_eq!(record.clip_time_seconds, 0.2);
        assert_eq!(record.playback_time_seconds, 1.2);
    }

    #[test]
    fn oversized_owner_produces_terminal_capacity_diagnostic() {
        let record = clip_event_batch_capacity_diagnostic(17, 257, 256);

        assert_eq!(record.entity, 17);
        assert_eq!(record.name, "animation.clip_event_batch_capacity_exceeded");
        assert_eq!(
            record.payload.as_deref(),
            Some("range_count=257;capacity=256")
        );
    }
}
