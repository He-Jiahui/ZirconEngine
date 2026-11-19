//! Public LevelSystem controls for sequence publication and clip admission.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::animation::{
    AnimationChannelKeyAsset, AnimationChannelValueAsset, AnimationClipEventBatchAdmission,
    AnimationClipEventQueueAdmission, AnimationClipEventSamplingRange, AnimationEventTrackAsset,
    AnimationInterpolationAsset,
};
use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
use zircon_runtime::core::resource::{ResourceActiveVersionKey, ResourceManager, ResourceState};
use zircon_runtime::scene::components::AnimationPlayerComponent;
use zircon_runtime::scene::ecs::{ComponentTicks, LifecycleEventKind};

use super::{
    animation_assets::single_hand_translation_clip, edit_mixed_translation, publish_mixed_ready,
    sequence_asset_for_entity, AnimationSequenceAsset, AnimationSequenceMarker,
    AnimationSequencePlayerComponent, AssetUri, EntityId, LocalTransform, ReadySequence, Real,
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, SequenceFrames, Vec3, FRAME,
    PAUSED_TIME, SENTINEL,
};

const SECOND_TIME: Real = 0.75;

fn translation_sequence(target: &str, values: [Vec3; 2]) -> AnimationSequenceAsset {
    let mut sequence = sequence_asset_for_entity(target);
    let binding = &mut sequence.bindings[0];
    binding.target_id = None;
    let channel = &mut binding.tracks[0].channel;
    channel.interpolation = AnimationInterpolationAsset::Linear;
    channel.keys = [
        (0.0, Vec3::ZERO),
        (PAUSED_TIME, values[0]),
        (SECOND_TIME, values[1]),
    ]
    .into_iter()
    .map(|(time_seconds, value)| AnimationChannelKeyAsset {
        time_seconds,
        value: AnimationChannelValueAsset::Vec3(value.to_array()),
        in_tangent: None,
        out_tangent: None,
    })
    .collect();
    sequence
}

fn authored_record(uri: &AssetUri, payload: &AnimationSequenceAsset) -> ResourceRecord {
    let mut digest = DefaultHasher::new();
    digest.write(&payload.to_bytes().unwrap());
    ResourceRecord::new(
        ResourceId::from_locator(uri),
        ResourceKind::AnimationSequence,
        uri.clone(),
    )
    .with_source_hash(format!("{:016x}", digest.finish()))
}

fn active_sequence(
    frames: &SequenceFrames,
    id: ResourceId,
    expected: &AnimationSequenceAsset,
) -> (ResourceActiveVersionKey, u64) {
    let assets = frames.assets();
    let resources = assets.resource_manager();
    let key = resources
        .projection_snapshot()
        .active()
        .key(id)
        .expect("active key");
    let snapshot = resources
        .snapshot_expected::<AnimationSequenceMarker, AnimationSequenceAsset>(
            ResourceHandle::new(id),
            &key,
        )
        .expect("paired active key and typed sequence");
    assert_eq!(snapshot.record().state, ResourceState::Ready);
    assert_eq!(snapshot.resource().as_ref(), expected);
    assert_eq!(&assets.load_animation_sequence_asset(id).unwrap(), expected);
    (key, snapshot.revision())
}

fn watch_translation(
    frames: &SequenceFrames,
    target: EntityId,
    publication: Option<(ResourceManager, ResourceRecord, AnimationSequenceAsset)>,
) -> Arc<Mutex<Vec<Vec3>>> {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&trace);
    frames.level.with_world_mut(|world| {
        world.observe_component_lifecycle::<LocalTransform>(
            LifecycleEventKind::Replace,
            // SAFETY: reads the replaced component and writes only an external trace
            // and the resource authority. No World mutations or new lifecycle producers
            // are reachable from this callback, so it adds nothing to a NodeRecord.
            unsafe {
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, event| {
                    if event.entity() != target {
                        return;
                    }
                    let mut writes = captured.lock().unwrap();
                    writes.push(
                        world
                            .get::<LocalTransform>(target)
                            .unwrap()
                            .transform
                            .translation,
                    );
                    let first = writes.len() == 1;
                    drop(writes);
                    if first {
                        if let Some((resources, record, payload)) = &publication {
                            resources
                                .register_ready(record.clone(), payload.clone())
                                .expect("publish real P2 during the first target write");
                        }
                    }
                })
            },
        );
    });
    trace
}

fn paused_stamp(
    frames: &SequenceFrames,
    clock: EntityId,
) -> (AnimationSequencePlayerComponent, ComponentTicks) {
    frames.level.with_world(|world| {
        let player = world.animation_sequence_player(clock).unwrap();
        assert!(!player.playing);
        (
            player.clone(),
            world
                .component_change_ticks::<AnimationSequencePlayerComponent>(clock)
                .unwrap(),
        )
    })
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_admission_batch_does_not_mix_publications() {
    let frames = SequenceFrames::open();
    let target = frames.spawn("BatchPublicationTarget");
    let uri = AssetUri::parse("res://animation/batch-publication.sequence.zranim").unwrap();
    let p1_values = [Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)];
    let p2_values = [Vec3::new(0.0, 7.0, 0.0), Vec3::new(0.0, 9.0, 0.0)];
    let p1 = translation_sequence("BatchPublicationTarget", p1_values);
    let p2 = translation_sequence("BatchPublicationTarget", p2_values);
    let p1_record = authored_record(&uri, &p1);
    let p2_record = authored_record(&uri, &p2);
    assert_ne!(p1_record.source_hash, p2_record.source_hash);
    assert_eq!(p1_record.id, p2_record.id);
    let id = p1_record.id;
    let resources = frames.assets().resource_manager();
    resources.register_ready(p1_record, p1.clone()).unwrap();
    let (p1_key, revision) = active_sequence(&frames, id, &p1);
    assert_eq!(revision, 1);
    let seq = ReadySequence { id, revision };
    let clocks = [
        frames.player("BatchClockA", seq, 0.0, false),
        frames.player("BatchClockB", seq, 0.0, false),
    ];
    edit_mixed_translation(&frames, target, SENTINEL);
    assert_eq!(frames.frame("batch_p1_initial", 2), 2);
    assert_eq!(frames.translation(target), Vec3::ZERO);
    assert_eq!(frames.frame("batch_p1_settled", 2), 0);

    for (clock, time) in clocks.into_iter().zip([PAUSED_TIME, SECOND_TIME]) {
        let (mut seek, ticks) = paused_stamp(&frames, clock);
        seek.time_seconds = time;
        frames.level.with_world_mut(|world| {
            assert!(world
                .set_animation_sequence_player(clock, Some(seek))
                .unwrap());
        });
        let (after, after_ticks) = paused_stamp(&frames, clock);
        assert_eq!(after.time_seconds, time);
        assert_eq!(after_ticks.added(), ticks.added());
        assert_ne!(after_ticks.changed(), ticks.changed());
    }
    let sought = clocks.map(|clock| paused_stamp(&frames, clock));
    assert_eq!(
        active_sequence(&frames, id, &p1),
        (p1_key.clone(), revision)
    );
    let trace = watch_translation(&frames, target, Some((resources, p2_record, p2.clone())));
    assert_eq!(frames.frame("batch_publish_during_apply", 2), 2);
    let during = trace.lock().unwrap().clone();
    let (p2_key, p2_revision) = active_sequence(&frames, id, &p2);
    println!("SEQUENCE_ADMISSION_BATCH_V1 p1={p1_key:?}/{revision} p2={p2_key:?}/{p2_revision} during={during:?}");
    assert_ne!(p1_key, p2_key);
    assert_eq!(p2_revision, revision + 1);
    assert_eq!(clocks.map(|clock| paused_stamp(&frames, clock)), sought);
    // The observer publishes only after an actual P1 write. Distinct owner values
    // detect interleaved generations, irrespective of their ECS iteration order.
    assert!((1..=2).contains(&during.len()) && during.iter().all(|v| p1_values.contains(v)));
    assert!(during.len() == 1 || during[0] != during[1]);

    assert_eq!(frames.frame("batch_next_is_p2", 2), 2);
    let all_writes = trace.lock().unwrap().clone();
    let next = &all_writes[during.len()..];
    println!("SEQUENCE_ADMISSION_BATCH_V1 next={next:?}");
    assert_eq!(next.len(), 2, "both owners still owe the actual P2 sample");
    assert!(p2_values.iter().all(|v| next.contains(v)));
    assert_eq!(frames.translation(target), next[1]);
    assert_eq!(frames.frame("batch_p2_quiet", 2), 0);
    assert_eq!(*trace.lock().unwrap(), all_writes);
    assert_eq!(clocks.map(|clock| paused_stamp(&frames, clock)), sought);
    assert!(!frames.level.animation_requires_continuous_frame());
}

#[test]
#[ignore = "requires an isolated host ZIRCON_CONFIG_PATH under the process temp directory"]
fn sequence_tick_admission_deferred_owner_retains_its_late_sample() {
    const CAPACITY: usize = 256;
    const DRAIN_WINDOW: usize = 32;
    const CLIP_EVENT: &str = "sequence_admission_retried_range";
    let frames = SequenceFrames::open();
    let handle = frames.core.handle();
    assert!(
        super::resolve_manager_service(&handle, super::animation_manager_handle(&handle).unwrap())
            .unwrap()
            .playback_settings()
            .skeletal_clips
    );
    let bound = frames.spawn("AdmissionBound");
    let values = [Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 3.0, 0.0)];
    let mut payload = translation_sequence("AdmissionBound", values);
    payload
        .bindings
        .extend(translation_sequence("AdmissionLate", values).bindings);
    let sequence_uri = AssetUri::parse("res://animation/admission-defer.sequence.zranim").unwrap();
    let seq = publish_mixed_ready(&frames, &sequence_uri, &payload);
    let active = active_sequence(&frames, seq.id, &payload);
    let clocks = [
        frames.player("AdmissionA", seq, PAUSED_TIME, false),
        frames.player("AdmissionB", seq, SECOND_TIME, false),
    ];
    let [_, b] = clocks;
    let paused = clocks.map(|clock| paused_stamp(&frames, clock));
    assert_eq!(frames.at_path("AdmissionLate"), None);
    assert_eq!(frames.frame("late_initial_partial", 2), 2);
    assert!(values.contains(&frames.translation(bound)));
    edit_mixed_translation(&frames, bound, SENTINEL);
    assert_eq!(frames.frame("late_initial_settled", 2), 0);

    let assets = frames.assets();
    let resources = assets.resource_manager();
    let skeleton = AssetUri::parse("res://animation/admission.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/admission-owner.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton, 0.0);
    clip.event_tracks.push(AnimationEventTrackAsset {
        target_id: None,
        event: CLIP_EVENT.to_owned(),
        time_seconds: 0.0625,
        payload: None,
    });
    resources
        .register_ready(
            ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
            clip.clone(),
        )
        .unwrap();
    assert_eq!(assets.load_animation_clip_asset(clip_id).unwrap(), clip);
    let gate = AnimationPlayerComponent {
        clip: ResourceHandle::new(clip_id),
        playback_speed: 1.0,
        time_seconds: 0.0,
        weight: 1.0,
        looping: false,
        playing: true,
    };
    let mut subscription = frames.level.with_world_mut(|world| {
        assert!(world.animation_skeleton(b).is_none()); // Event-only producer.
        assert!(world.animation_player(clocks[0]).is_none());
        assert!(world.set_animation_player(b, Some(gate.clone())).unwrap());
        let mut subscription =
            world.register_dormant_event_subscription::<super::animation::AnimationClipEvent>();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    });
    let clip_player = || {
        frames
            .level
            .with_world(|world| world.animation_player(b).unwrap().clone())
    };
    let clip_ticks = || {
        frames.level.with_world(|world| {
            world
                .component_change_ticks::<AnimationPlayerComponent>(b)
                .unwrap()
        })
    };
    let clip_before = clip_ticks();
    let backlog = || frames.level.animation_clip_event_backlog_len(frames.epoch);
    let filler_uri = AssetUri::parse("res://animation/admission-filler.clip.zranim").unwrap();
    let filler_id = ResourceId::from_locator(&filler_uri);
    assert!(assets.load_animation_clip_asset(filler_id).is_err());
    let contributor = frames.spawn("AdmissionBacklogProducer");
    let filler = AnimationClipEventSamplingRange {
        entity: contributor,
        clip_id: filler_id,
        from_time_seconds: 0.0,
        to_time_seconds: 0.125,
        looping: false,
    };
    let admission = frames
        .level
        .enqueue_animation_clip_event_range_batches(frames.epoch, vec![vec![filler; CAPACITY]]);
    assert!(
        matches!(admission, AnimationClipEventQueueAdmission::Current {
        batch_admissions, admitted_range_count: CAPACITY, deferred_range_count: 0, rejected_range_count: 0,
    } if batch_admissions.as_slice() == [AnimationClipEventBatchAdmission::Admitted])
    );
    assert_eq!(backlog(), Some(CAPACITY));
    let late = frames.spawn("AdmissionLate");
    assert_eq!(frames.at_path("AdmissionLate"), Some(late));
    assert_eq!(frames.translation(late), Vec3::ZERO);
    let trace = watch_translation(&frames, late, None);

    let deferred = frames.frame("late_b_deferred", 2);
    let deferred_trace = trace.lock().unwrap().clone();
    println!(
        "SEQUENCE_ADMISSION_DEFER_V1 blocked={:?} clip_time={} late={deferred_trace:?}",
        backlog(),
        clip_player().time_seconds
    );
    assert_eq!(
        backlog(),
        Some(CAPACITY),
        "unreadable ranges must leave the queue full"
    );
    assert_eq!(
        clip_player(),
        gate,
        "B's real clip range must be capacity-deferred"
    );
    assert_eq!(clip_ticks(), clip_before);
    assert_eq!(clocks.map(|clock| paused_stamp(&frames, clock)), paused);
    let bound_when_deferred = frames.translation(bound);

    // Only resource publication clears the admission obstacle. No seek or binding
    // edit can create a new sequence request for B before the next normal frame.
    resources
        .register_ready(
            ResourceRecord::new(filler_id, ResourceKind::AnimationClip, filler_uri),
            single_hand_translation_clip(&skeleton, 0.0),
        )
        .unwrap();
    let resumed = frames.frame("late_b_admitted", 2);
    let resumed_trace = trace.lock().unwrap().clone();
    println!("SEQUENCE_ADMISSION_DEFER_V1 resumed={:?} clip_time={} late={resumed_trace:?} requests={deferred}/{resumed}", backlog(), clip_player().time_seconds);
    assert_eq!(backlog(), Some(CAPACITY - DRAIN_WINDOW + 1));
    assert_eq!(
        clip_player().time_seconds,
        FRAME.as_secs_f32(),
        "B's range really entered the queue"
    );
    assert_eq!(clip_ticks().added(), clip_before.added());
    assert_ne!(clip_ticks().changed(), clip_before.changed());
    assert_eq!(clocks.map(|clock| paused_stamp(&frames, clock)), paused);
    let bound_when_resumed = frames.translation(bound);
    // Stop just the separate clip clock; its admitted range must reach the real event stream.
    frames
        .level
        .with_world_mut(|world| assert!(world.set_animation_player(b, None).unwrap()));
    let mut quiet = 0;
    let mut received = Vec::new();
    // One extra frame lets the scheduled First-stage event update expose the last drain.
    for _ in 0..=CAPACITY / DRAIN_WINDOW {
        quiet += frames.frame("late_quiet_and_drain", 2);
        frames.level.with_world(|world| {
            received.extend(world.read_event_subscription(&mut subscription).cloned());
        });
    }
    assert_eq!(backlog(), Some(0));
    assert_eq!(received.len(), 1);
    assert_eq!(
        (
            received[0].entity,
            received[0].event.as_str(),
            received[0].clip_time_seconds
        ),
        (b, CLIP_EVENT, 0.0625)
    );
    assert_eq!(clocks.map(|clock| paused_stamp(&frames, clock)), paused);
    assert_eq!(active_sequence(&frames, seq.id, &payload), active);
    assert!(!frames.level.animation_requires_continuous_frame());
    assert_eq!(
        deferred_trace.as_slice(),
        &[values[0]],
        "only admitted A writes at its paused time"
    );
    assert_eq!(
        resumed_trace.as_slice(),
        &values,
        "B was not consumed and writes at its own paused time"
    );
    assert_eq!(*trace.lock().unwrap(), resumed_trace);
    assert_eq!(frames.translation(late), values[1]);
    assert_eq!(
        [
            bound_when_deferred,
            bound_when_resumed,
            frames.translation(bound)
        ],
        [SENTINEL; 3],
        "no full or repeated late reapply"
    );
    assert_eq!([deferred, resumed, quiet], [2, 1, 0]);
}
