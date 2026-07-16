use zircon_runtime::animation::sample_clip_events;
use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::scene::LevelSystem;

use super::requests::PendingClipEventSample;

pub(super) fn publish_clip_events(
    asset_manager: &ProjectAssetManager,
    level: &LevelSystem,
    pending_samples: Vec<PendingClipEventSample>,
) {
    let events = pending_samples
        .into_iter()
        .filter_map(|pending| {
            let clip = asset_manager
                .load_animation_clip_asset(pending.clip_id)
                .ok()?;
            Some(sample_clip_events(
                &clip,
                pending.entity,
                pending.from_time_seconds,
                pending.to_time_seconds,
                pending.looping,
            ))
        })
        .flatten()
        .collect::<Vec<_>>();
    publish_events(
        level,
        events
            .iter()
            .map(animation_event_record)
            .collect::<Vec<_>>(),
    );
    publish_events(level, events);
}

fn animation_event_record(
    event: &zircon_runtime::animation::AnimationClipEvent,
) -> AnimationEventRecord {
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

pub(super) fn publish_events<E>(level: &LevelSystem, events: Vec<E>)
where
    E: zircon_runtime::scene::ecs::Event,
{
    if events.is_empty() {
        return;
    }
    level.with_world_mut(|world| publish_events_to_world(world, events));
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
    use zircon_runtime::animation::AnimationClipEvent;

    use super::animation_event_record;

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
}
