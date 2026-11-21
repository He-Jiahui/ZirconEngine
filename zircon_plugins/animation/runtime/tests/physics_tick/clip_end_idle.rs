//! Tests-only candidate. Install as tests/physics_tick/clip_end_idle.rs and add
//! #[path = "physics_tick/clip_end_idle.rs"] mod clip_end_idle; after the
//! clip_events include in tests/physics_tick.rs. Not compiled or run here.

use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::scene::{EntityId, EventSubscription, LevelSystem};

use super::{
    drain_animation_clip_events, runtime_asset_manager, runtime_with_physics_animation_scene_asset,
    single_hand_translation_clip, subscribe_animation_clip_events, AnimationClipMarker,
    AnimationEventTrackAsset, AnimationPlayerComponent, AssetUri, NodeKind, Real, ResourceHandle,
    ResourceId, ResourceKind, ResourceRecord,
};

#[derive(Clone, Copy)]
enum Playback {
    TerminalLongStep,
    TerminalShortSteps,
    PauseAfterAdmission,
    Looping,
}

#[test]
fn terminal_clip_goes_idle_after_admitted_notify_tail() {
    run(Playback::TerminalLongStep);
}

#[test]
fn short_steps_deliver_both_points_and_go_idle() {
    run(Playback::TerminalShortSteps);
}

#[test]
fn admitted_tail_is_not_discarded_when_player_pauses() {
    run(Playback::PauseAfterAdmission);
}

#[test]
fn looping_clip_keeps_legitimate_post_boundary_notifications() {
    run(Playback::Looping);
}

type Occurrence = (EntityId, String, Real, Real);

#[derive(Debug)]
struct Frame {
    typed: Vec<Occurrence>,
    neutral: Vec<Occurrence>,
    backlog: usize,
    demand: bool,
    time: Real,
    playing: bool,
}

fn run(playback: Playback) {
    let runtime = runtime_with_physics_animation_scene_asset();
    let assets = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = AssetUri::parse("res://animation/clip-end-idle.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/clip-end-idle.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.duration_seconds = 2.0;
    clip.event_tracks = [("early", 0.75), ("late", 1.75)]
        .into_iter()
        .map(|(event, time_seconds)| AnimationEventTrackAsset {
            target_id: None,
            event: event.to_owned(),
            time_seconds,
            payload: None,
        })
        .collect();
    assets.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube).unwrap();
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 8.0,
                    time_seconds: 0.5,
                    weight: 1.0,
                    looping: matches!(playback, Playback::Looping),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    let mut typed = subscribe_animation_clip_events(&level);
    let mut neutral = subscribe_neutral(&level);
    let epoch = level.capture_world_replacement_epoch();
    let mut step = |dt| {
        runtime.tick_level_seconds(&level, dt).unwrap();
        let backlog = level.animation_clip_event_backlog_len(epoch).unwrap();
        let demand = level.animation_requires_continuous_frame();
        let (time, playing) = level.with_world(|world| {
            let player = world.animation_player(entity).unwrap();
            (player.time_seconds, player.playing)
        });
        let typed = drain_animation_clip_events(&level, &mut typed)
            .into_iter()
            .map(|event| {
                (
                    event.entity,
                    event.event,
                    event.clip_time_seconds,
                    event.playback_time_seconds,
                )
            })
            .collect();
        let neutral = level.with_world_mut(|world| {
            world.update_events::<AnimationEventRecord>();
            world
                .read_event_subscription(&mut neutral)
                .map(|event| {
                    (
                        event.entity,
                        event.name.clone(),
                        event.clip_time_seconds,
                        event.playback_time_seconds,
                    )
                })
                .collect()
        });
        Frame {
            typed,
            neutral,
            backlog,
            demand,
            time,
            playing,
        }
    };

    let short = matches!(playback, Playback::TerminalShortSteps);
    let pause = matches!(playback, Playback::PauseAfterAdmission);
    let looping = matches!(playback, Playback::Looping);
    let first = step(if short { 0.125 } else { 0.1875 });
    if pause {
        level.with_world_mut(|world| {
            let mut player = world.animation_player(entity).unwrap().clone();
            player.playing = false;
            world.set_animation_player(entity, Some(player)).unwrap();
        });
    }
    let second = step(if short { 0.0625 } else { 0.125 });
    let third = step(0.125);

    // Verify real notifications and clocks before judging whether the Level may become idle.
    let early = (entity, "early".to_string(), 0.75, 0.75);
    let late = (entity, "late".to_string(), 1.75, 1.75);
    assert_occurrences(&first, &[early]);
    assert_occurrences(&second, &[late]);
    if looping {
        assert_occurrences(&third, &[(entity, "early".to_string(), 0.75, 2.75)]);
    } else {
        assert_occurrences(&third, &[]);
    }
    assert_eq!(first.time, if short { 1.5 } else { 2.0 });
    assert_eq!(second.time, if short || pause { 2.0 } else { 3.0 });
    assert_eq!(third.time, if pause { 2.0 } else if short { 3.0 } else { 4.0 });
    assert_eq!((first.playing, second.playing, third.playing), (true, !pause, !pause));
    assert_eq!((first.backlog, first.demand), (if short { 0 } else { 1 }, true), "{first:?}");

    let expected_tail = if looping { (1, true) } else { (0, false) };
    assert_eq!(
        [(second.backlog, second.demand), (third.backlog, third.demand)],
        [expected_tail; 2],
        "old queue must empty after real work ends: second={second:?}, third={third:?}"
    );
}

fn subscribe_neutral(level: &LevelSystem) -> EventSubscription<AnimationEventRecord> {
    level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<AnimationEventRecord>();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    })
}

fn assert_occurrences(frame: &Frame, expected: &[Occurrence]) {
    assert_eq!(frame.typed.as_slice(), expected, "typed: {frame:?}");
    assert_eq!(frame.neutral.as_slice(), expected, "neutral: {frame:?}");
}
