use zircon_runtime::animation::sample_clip_events;
use zircon_runtime::asset::ProjectAssetManager;
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
    publish_events(level, events);
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
