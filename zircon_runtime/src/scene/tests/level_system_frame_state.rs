#[cfg(feature = "animation")]
use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(feature = "animation")]
use crate::core::framework::animation::{
    AnimationClipEvent, AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission,
    AnimationClipEventSampler, AnimationClipEventSamplingBatch, AnimationClipEventSamplingCursor,
    AnimationClipEventSamplingRange, AnimationClipEventSamplingRequest, AnimationEventRecord,
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
#[cfg(feature = "physics-contracts")]
use crate::core::framework::physics::{SimulatedPoseFeed, SkeletalPoseTarget, SkeletalPoseTargets};
use crate::core::math::Transform;
#[cfg(feature = "animation")]
use crate::core::math::Vec3;
#[cfg(feature = "animation")]
use crate::core::resource::ResourceId;
use crate::scene::components::Name;
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::{DefaultLevelManager, World};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration, WorldFact};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RetiredWorldEvent(u32);

#[test]
fn replacing_a_level_publishes_the_new_epoch_and_invalidates_existing_watches() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let token = level.watch_world(WatchRegistration::new(WatchKey::WorldStructure));
    let previous_epoch = level.capture_world_replacement_epoch();

    level.replace_world_and_reset_runtime_state(World::empty());

    let current_epoch = level.capture_world_replacement_epoch();
    assert_eq!(current_epoch, previous_epoch.checked_add(1).unwrap());
    assert_eq!(
        level.drain_world_invalidations(),
        vec![zircon_runtime_interface::world_sync::InvalidationBatch {
            generation: level.with_world(World::world_generation),
            dirty: vec![token],
            facts: vec![WorldFact::WorldReplaced {
                replacement_epoch: current_epoch,
            }],
        }]
    );
}

#[cfg(feature = "animation")]
fn clip_event_range(entity: u64) -> AnimationClipEventSamplingRange {
    AnimationClipEventSamplingRange {
        entity,
        clip_id: ResourceId::from_stable_label("animation.clip.capacity"),
        from_time_seconds: 0.0,
        to_time_seconds: 1.0,
        looping: false,
    }
}

#[cfg(feature = "animation")]
fn pose_output() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: Some("Locomotion".to_string()),
        bones: vec![AnimationPoseBone {
            name: "Root".to_string(),
            local_transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        }],
    }
}

#[cfg(feature = "animation")]
struct UnavailableClipEventSampler;

#[cfg(feature = "animation")]
impl AnimationClipEventSampler for UnavailableClipEventSampler {
    fn sample_clip_events(
        &self,
        _request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        None
    }
}

#[cfg(feature = "animation")]
struct CompleteClipEventSampler;

#[cfg(feature = "animation")]
impl AnimationClipEventSampler for CompleteClipEventSampler {
    fn sample_clip_events(
        &self,
        _request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        Some(AnimationClipEventSamplingBatch::default())
    }
}

#[test]
#[cfg(feature = "animation")]
fn level_clip_event_queue_retries_an_unavailable_contract_sample() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity: 17,
                clip_id: ResourceId::from_stable_label("animation.clip.pending"),
                from_time_seconds: 0.0,
                to_time_seconds: 1.0,
                looping: false,
            }]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );

    assert!(
        level
            .drain_animation_clip_events(replacement_epoch, &UnavailableClipEventSampler)
            .expect("current replacement epoch drains the queue")
            .is_empty()
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(1)
    );
    assert_eq!(
        level.animation_clip_event_drain_metrics(),
        (1, 1, false, 0, 1, 0)
    );
}

#[cfg(feature = "animation")]
struct PartialThenCompleteClipEventSampler;

#[cfg(feature = "animation")]
impl AnimationClipEventSampler for PartialThenCompleteClipEventSampler {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        if request.entity == 17 {
            return Some(AnimationClipEventSamplingBatch {
                next_cursor: Some(AnimationClipEventSamplingCursor::at_range_start(
                    request.cursor.playback_time_seconds + 1.0,
                )),
                playback_span_seconds: 1.0,
                ..AnimationClipEventSamplingBatch::default()
            });
        }

        Some(AnimationClipEventSamplingBatch {
            events: vec![AnimationClipEvent {
                entity: request.entity,
                target_id: None,
                event: "ready".to_string(),
                payload: None,
                clip_time_seconds: 0.25,
                playback_time_seconds: 0.25,
            }],
            emitted_event_bytes: "ready".len(),
            playback_span_seconds: 0.25,
            ..AnimationClipEventSamplingBatch::default()
        })
    }
}

#[test]
#[cfg(feature = "animation")]
fn level_clip_event_queue_rotates_a_partial_sample_behind_ready_work() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity: 17,
                clip_id: ResourceId::from_stable_label("animation.clip.long"),
                from_time_seconds: 0.0,
                to_time_seconds: 4.0,
                looping: false,
            }]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity: 18,
                clip_id: ResourceId::from_stable_label("animation.clip.ready"),
                from_time_seconds: 0.0,
                to_time_seconds: 1.0,
                looping: false,
            }]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );

    let events = level
        .drain_animation_clip_events(replacement_epoch, &PartialThenCompleteClipEventSampler)
        .expect("current replacement epoch drains the queue");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, 18);
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(1)
    );
    assert_eq!(
        level.animation_clip_event_drain_metrics(),
        (1, 1, false, 0, 0, 0)
    );
}

#[test]
#[cfg(feature = "animation")]
fn level_clip_event_queue_bounds_growth_and_ages_tail_by_drain_window() {
    const EXPECTED_PENDING_CAPACITY: usize = 256;

    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    for entity in 0..EXPECTED_PENDING_CAPACITY as u64 {
        assert_eq!(
            level.enqueue_animation_clip_event_range_batches(
                replacement_epoch,
                vec![vec![clip_event_range(entity)]],
            ),
            AnimationClipEventQueueAdmission::Current {
                batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
                admitted_range_count: 1,
                deferred_range_count: 0,
                rejected_range_count: 0,
            },
        );
    }
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![clip_event_range(EXPECTED_PENDING_CAPACITY as u64)]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Deferred],
            admitted_range_count: 0,
            deferred_range_count: 1,
            rejected_range_count: 0,
        },
    );

    assert!(
        level
            .drain_animation_clip_events(replacement_epoch, &UnavailableClipEventSampler)
            .expect("current replacement epoch drains the bounded queue")
            .is_empty()
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(EXPECTED_PENDING_CAPACITY)
    );
    assert_eq!(
        level.animation_clip_event_drain_metrics(),
        (EXPECTED_PENDING_CAPACITY, 1, false, 0, 32, 1)
    );

    assert!(
        level
            .drain_animation_clip_events(replacement_epoch, &UnavailableClipEventSampler)
            .expect("current replacement epoch drains the next bounded batch")
            .is_empty()
    );
    assert_eq!(
        level.animation_clip_event_drain_metrics(),
        (EXPECTED_PENDING_CAPACITY, 2, false, 0, 32, 0)
    );
}

#[test]
#[cfg(feature = "animation")]
fn oversized_clip_event_owner_is_rejected_without_blocking_the_next_owner() {
    const EXPECTED_PENDING_CAPACITY: usize = 256;

    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let oversized = (0..=EXPECTED_PENDING_CAPACITY)
        .map(|_| clip_event_range(17))
        .collect::<Vec<_>>();

    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![oversized, vec![clip_event_range(18)]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![
                AnimationClipEventBatchAdmission::RejectedOversized {
                    range_count: EXPECTED_PENDING_CAPACITY + 1,
                    capacity: EXPECTED_PENDING_CAPACITY,
                },
                AnimationClipEventBatchAdmission::Admitted,
            ],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: EXPECTED_PENDING_CAPACITY + 1,
        },
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(1)
    );
}

#[test]
#[cfg(feature = "animation")]
fn level_clip_event_queue_keeps_each_producer_batch_atomic() {
    const RETAINED_SAMPLES: usize = 255;

    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            (0..RETAINED_SAMPLES as u64)
                .map(|entity| vec![clip_event_range(entity)])
                .collect(),
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted; RETAINED_SAMPLES],
            admitted_range_count: RETAINED_SAMPLES,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![clip_event_range(300), clip_event_range(301)]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Deferred],
            admitted_range_count: 0,
            deferred_range_count: 2,
            rejected_range_count: 0,
        },
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(RETAINED_SAMPLES),
    );
}

#[test]
#[cfg(feature = "animation")]
fn over_capacity_clip_event_batches_make_segmented_progress() {
    const PENDING_CAPACITY: usize = 256;

    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();

    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            (0..=PENDING_CAPACITY as u64)
                .map(|entity| vec![clip_event_range(entity)])
                .collect(),
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: (0..=PENDING_CAPACITY)
                .map(|index| {
                    if index < PENDING_CAPACITY {
                        AnimationClipEventBatchAdmission::Admitted
                    } else {
                        AnimationClipEventBatchAdmission::Deferred
                    }
                })
                .collect(),
            admitted_range_count: PENDING_CAPACITY,
            deferred_range_count: 1,
            rejected_range_count: 0,
        },
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(PENDING_CAPACITY),
    );
    assert!(level.animation_requires_continuous_frame());

    assert!(
        level
            .drain_animation_clip_events(replacement_epoch, &CompleteClipEventSampler)
            .expect("current replacement epoch drains one bounded window")
            .is_empty()
    );
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![clip_event_range(PENDING_CAPACITY as u64)]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_reuses_sealed_pose_handle_until_world_replacement() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial = level.frame_state_snapshot();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let poses = Arc::new(BTreeMap::from([(17, Arc::new(pose_output()))]));

    assert!(level.record_animation_pose_snapshot(replacement_epoch, Arc::clone(&poses)));
    let published = level.frame_state_snapshot();
    assert_eq!(
        published.animation_generation(),
        initial.animation_generation() + 1
    );
    assert!(Arc::ptr_eq(published.animation_poses(), &poses));

    assert!(level.record_animation_pose_snapshot(replacement_epoch, Arc::clone(&poses)));
    let stable = level.frame_state_snapshot();
    assert!(Arc::ptr_eq(&published, &stable));
    assert!(Arc::ptr_eq(stable.animation_poses(), &poses));

    level.replace_world_and_reset_runtime_state(World::empty());
    let reset = level.frame_state_snapshot();
    assert!(!Arc::ptr_eq(&published, &reset));
    assert_eq!(
        reset.world_generation(),
        level.with_world(World::world_generation)
    );
    assert!(reset.animation_poses().is_empty());
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_rejects_a_pose_payload_from_a_retired_replacement_epoch() {
    let level = DefaultLevelManager::default().create_default_level();
    let retired_epoch = level.capture_world_replacement_epoch();
    let stale_poses = Arc::new(BTreeMap::from([(17, Arc::new(pose_output()))]));

    level.replace_world_and_reset_runtime_state(World::empty());
    assert!(!level.record_animation_pose_snapshot(retired_epoch, stale_poses));
    assert!(level.frame_state_snapshot().animation_poses().is_empty());
}

#[test]
#[cfg(feature = "animation")]
fn ordinary_world_mutation_does_not_retire_animation_publication_epoch() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let before_mutation = level.world_generation();
    level.with_world_mut(|world| {
        world
            .spawn_node(crate::scene::NodeKind::Empty)
            .expect("test scene spawn should succeed");
    });
    let after_mutation = level.world_generation();
    assert!(after_mutation > before_mutation);

    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([(17, Arc::new(pose_output()))])),
    ));
    assert_eq!(
        level.frame_state_snapshot().world_generation(),
        after_mutation
    );
}

#[test]
#[cfg(feature = "animation")]
fn replacement_epoch_rejects_retired_world_and_animation_state_writes() {
    let level = DefaultLevelManager::default().create_default_level();
    let retired_epoch = level.capture_world_replacement_epoch();

    level.replace_world_and_reset_runtime_state(World::empty());
    let current_epoch = level.capture_world_replacement_epoch();
    assert_ne!(current_epoch, retired_epoch);
    assert!(
        level
            .with_world_mut_if_replacement_epoch(retired_epoch, |world| {
                world
                    .spawn_node(crate::scene::NodeKind::Empty)
                    .expect("test scene spawn should succeed")
            })
            .is_none()
    );
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            retired_epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity: 17,
                clip_id: ResourceId::from_stable_label("animation.clip.retired"),
                from_time_seconds: 0.0,
                to_time_seconds: 1.0,
                looping: false,
            }]],
        ),
        AnimationClipEventQueueAdmission::RetiredEpoch,
    );
    assert!(
        level
            .drain_animation_clip_events(retired_epoch, &UnavailableClipEventSampler)
            .is_none()
    );
    assert!(!level.record_animation_playback_times(
        retired_epoch,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    ));
}

#[test]
#[cfg(feature = "animation")]
fn transactional_world_replacement_retires_the_same_epoch_contract() {
    let level = DefaultLevelManager::default().create_default_level();
    let retired_epoch = level.capture_world_replacement_epoch();
    let expected_generation = level.world_generation();

    level
        .replace_world_if_generation(expected_generation, World::empty())
        .expect("current transaction generation replaces the World");

    assert_ne!(
        level.capture_world_replacement_epoch(),
        retired_epoch,
        "transactional and direct replacements must retire the same producer token"
    );
    assert!(
        level
            .with_world_mut_if_replacement_epoch(retired_epoch, |_| ())
            .is_none()
    );
}

#[test]
#[cfg(feature = "animation")]
fn transactional_world_replacement_discards_retired_animation_events() {
    let level = DefaultLevelManager::default().create_default_level();
    let mut retained_subscription = level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<RetiredWorldEvent>();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    });
    level.with_world_mut(|world| {
        world.send_event(AnimationClipEvent {
            entity: 17,
            target_id: None,
            event: "retired.clip".to_string(),
            payload: None,
            clip_time_seconds: 0.25,
            playback_time_seconds: 0.25,
        });
        world.send_event(AnimationEventRecord::new(17, "retired.record"));
        world.update_events::<AnimationEventRecord>();
        world.send_event(RetiredWorldEvent(1));
        world.update_events::<RetiredWorldEvent>();
        world.send_event(RetiredWorldEvent(2));
    });
    let expected_generation = level.world_generation();

    level
        .replace_world_if_generation(expected_generation, World::empty())
        .expect("current transaction generation replaces the World");
    level.with_world_mut(|world| {
        world.update_events::<AnimationClipEvent>();
        world.update_events::<AnimationEventRecord>();
        world.update_events::<RetiredWorldEvent>();
        assert!(
            world
                .events::<AnimationClipEvent>()
                .expect("clip event channel remains registered")
                .is_empty()
        );
        assert!(
            world
                .events::<AnimationEventRecord>()
                .expect("animation event channel remains registered")
                .is_empty()
        );
        assert!(
            world
                .events::<RetiredWorldEvent>()
                .expect("unrelated event channel remains registered")
                .is_empty()
        );
        world.send_event(RetiredWorldEvent(3));
        world.update_events::<RetiredWorldEvent>();
        assert_eq!(
            world
                .read_event_subscription(&mut retained_subscription)
                .map(|event| event.0)
                .collect::<Vec<_>>(),
            vec![3],
            "replacement preserves the reader but not retired queued events"
        );
    });
}

#[cfg(feature = "physics-contracts")]
#[test]
fn transactional_world_replacement_clears_retained_pose_resources_without_animation_runtime() {
    let level = DefaultLevelManager::default().create_default_level();
    level.with_world_mut(|world| {
        let retired_target = SkeletalPoseTarget {
            bone_name: "Root".to_string(),
            local_transform: Transform::default(),
            normalized_weight: 1.0,
        };
        let mut targets = SkeletalPoseTargets::default();
        targets.replace(17, Arc::from([retired_target.clone()]));
        world.insert_resource(targets);
        let mut feed = SimulatedPoseFeed::default();
        feed.replace(17, Arc::from([retired_target]));
        world.insert_resource(feed);
    });
    let expected_generation = level.world_generation();

    level
        .replace_world_if_generation(expected_generation, World::empty())
        .expect("current transaction generation replaces the World");

    level.with_world(|world| {
        assert!(
            world
                .resource::<SkeletalPoseTargets>()
                .targets(17)
                .is_none()
        );
        assert!(world.resource::<SimulatedPoseFeed>().targets(17).is_none());
    });
}

#[test]
fn transactional_world_replacement_preserves_staged_lifecycle_callback_events() {
    let level = DefaultLevelManager::default().create_default_level();
    let mut subscription = level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<RetiredWorldEvent>();
        assert!(world.connect_event_subscription(&mut subscription));
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, |world, _| {
            world.send_event(RetiredWorldEvent(99));
        });
        subscription
    });
    let (mut staged, _) = level.with_world_mut(|world| {
        world
            .clone_for_dynamic_scene_staging(1024 * 1024)
            .expect("bounded World clones into transaction staging")
    });
    staged
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let expected_generation = level.world_generation();

    level
        .replace_world_if_generation(expected_generation, staged)
        .expect("current transaction generation replaces the World");

    level.with_world_mut(|world| {
        world.update_events::<RetiredWorldEvent>();
        assert_eq!(
            world
                .read_event_subscription(&mut subscription)
                .map(|event| event.0)
                .collect::<Vec<_>>(),
            vec![99],
            "replacement must preserve events emitted during staged lifecycle commit"
        );
    });
}

#[test]
#[cfg(feature = "animation")]
fn level_replace_retires_the_sealed_pose_payload_through_the_public_entry_point() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let world_generation = level.world_generation();
    let poses = Arc::new(BTreeMap::from([(17, Arc::new(pose_output()))]));

    assert!(level.record_animation_pose_snapshot(replacement_epoch, poses));
    let published = level.frame_state_snapshot();
    assert!(!published.animation_poses().is_empty());

    level.replace(World::empty());
    let replaced = level.frame_state_snapshot();
    assert!(!Arc::ptr_eq(&published, &replaced));
    assert!(replaced.animation_poses().is_empty());
    assert!(replaced.world_generation() > world_generation);
}

#[test]
fn level_script_binding_query_uses_the_borrowed_key_without_call_site_allocation() {
    let level = DefaultLevelManager::default().create_default_level();
    let entity = 17;

    level.mark_script_binding_started(entity, "player-controller");
    assert!(level.script_binding_started(entity, "player-controller"));
    assert!(!level.script_binding_started(entity, "camera-controller"));

    let source = include_str!("../level_system/frame_state.rs");
    assert!(source.contains("bindings.contains(binding_key)"));
    assert!(!source.contains("binding_key.to_string()"));
}

#[test]
fn level_render_extract_projects_sealed_pose_payload_after_the_world_lane() {
    let source = include_str!("../level_system_render_extract.rs");

    assert!(source.contains("let candidate_entities = frame_state"));
    assert!(source.contains("let (mut extract, skeletons) = self.with_world_mut"));
    assert!(source.contains("skeletons.into_iter()"));
    assert!(source.contains("pose: pose.clone()"));
}

#[test]
#[cfg(feature = "animation")]
fn level_world_replacement_and_pose_publication_share_an_epoch_commit_order() {
    let level_source = include_str!("../level_system.rs");
    let animation_source = include_str!("../level_system/animation_runtime.rs");
    let replacement = level_source
        .split("pub fn replace_world_and_reset_runtime_state")
        .nth(1)
        .and_then(|section| section.split("pub fn with_world").next())
        .expect("read world replacement implementation");
    let publication = animation_source
        .split("pub fn record_animation_pose_snapshot")
        .nth(1)
        .and_then(|section| {
            section
                .split("pub fn record_animation_playback_times")
                .next()
        })
        .expect("read animation publication implementation");

    let replacement_world_lock = replacement
        .find("let mut current = self.lock_world();")
        .expect("replacement holds the World lane");
    let replacement_frame_lock = replacement
        .find("let mut frame_state = self.lock_frame_state();")
        .expect("replacement publishes the retirement frame while holding the World lane");
    assert!(replacement_world_lock < replacement_frame_lock);
    let replacement_epoch_advance = replacement
        .find("self.advance_world_replacement_epoch()")
        .expect("replacement retires the previous producer epoch without allowing wraparound");
    assert!(replacement_epoch_advance < replacement_frame_lock);

    let publication_world_lock = publication
        .find("let world = self.lock_world();")
        .expect("publication validates against the World lane");
    let publication_frame_lock = publication
        .find("let mut current = self.lock_frame_state();")
        .expect("publication commits while the World lane remains held");
    assert!(publication_world_lock < publication_frame_lock);
    assert!(publication.contains("replacement_epoch: u64"));
    assert!(publication.contains("world_replacement_epoch.load(Ordering::Acquire)"));
    assert!(publication.contains("let world_generation = world.world_generation();"));
}
