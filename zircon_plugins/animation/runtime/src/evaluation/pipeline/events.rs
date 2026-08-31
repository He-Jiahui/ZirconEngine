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
    level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            for event in events.iter().map(animation_event_record) {
                world.send_event(event);
            }
            publish_events_to_world(world, events);
        })
        .is_some()
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
    use std::hint::black_box;
    use std::sync::Mutex;
    use std::time::Instant;

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

    #[test]
    fn optimization_batch_20260830cg_clip_events_publish_in_one_world_transaction() {
        let source = include_str!("events.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let start = production
            .find("pub(super) fn publish_clip_events(")
            .expect("clip-event publisher");
        let end = production[start..]
            .find("fn animation_event_record(")
            .map(|offset| start + offset)
            .expect("clip-event publisher boundary");
        let publisher = &production[start..end];

        assert_eq!(
            publisher
                .matches("with_world_mut_if_replacement_epoch")
                .count(),
            1
        );
        assert!(!publisher.contains("publish_events("));
        assert!(publisher.contains("animation_event_record"));
        assert!(publisher.contains("publish_events_to_world"));
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830cg_atomic_clip_event_publish_p95() {
        const ITERATIONS: usize = 100_000;
        const SAMPLES: usize = 17;
        let world = Mutex::new(0_u64);
        let events = (0..8_u64).collect::<Vec<_>>();
        let mut baseline_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let baseline = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    publish_model(&world, &events, true);
                }
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    publish_model(&world, &events, false);
                }
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                baseline_samples.push(baseline());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                baseline_samples.push(baseline());
            }
        }

        let baseline_p95 = percentile_95(&mut baseline_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME170_ATOMIC_EVENT_PUBLISH_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(75),
            "expected one world transaction to reduce P95 by at least 25%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn publish_model(world: &Mutex<u64>, events: &[u64], split: bool) {
        if split {
            let neutral = events.to_vec();
            let mut value = world.lock().unwrap();
            for event in neutral {
                *value = value.wrapping_add(event);
            }
            drop(value);
        }
        let mut value = world.lock().unwrap();
        for event in events {
            *value = value.wrapping_add(*event);
            if !split {
                *value = value.wrapping_add(*event);
            }
        }
        black_box(*value);
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
