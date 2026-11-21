//! A late-start fade has different root, outgoing, and target clock windows.
use super::{
    drain_animation_clip_events, register_single_clip_graph, runtime_asset_manager,
    runtime_with_physics_animation_scene_asset, single_hand_translation_clip,
    state_machine_interruption::{spawn_state_machine_player, uri},
    subscribe_animation_clip_events, timed_transition_state_machine, two_bone_skeleton,
    AnimationEventTrackAsset, AnimationParameterValue, BTreeMap, Real, ResourceId, ResourceKind,
    ResourceRecord,
};
use zircon_plugin_animation_runtime::AnimationClipEvent;
use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId};

const STEP: Real = 0.125;
const TARGET: &str = "Root/Hand";
type Point = (&'static str, &'static str, Real);
// The two clips deliberately author the same name and target on different clocks.
const IDLE_MID: Point = ("fade_mid", "idle", 0.6875);
const RUN_MID: Point = ("fade_mid", "run", 0.1875);
const RUN_AT_RESUME_ROOT: Point = ("run_at_resume_root", "run", 0.6875);
const RUN_STEADY: Point = ("run_steady", "run", 0.8125);

#[derive(Debug)]
struct Frame {
    epoch: u64,
    playing: bool,
    state: Option<String>,
    root: Option<Real>,
    transition: Option<AnimationStateTransitionRuntime>,
    backlog_before: Option<usize>,
    backlog_after: Option<usize>,
    typed: Vec<AnimationClipEvent>,
    neutral: Vec<AnimationEventRecord>,
}

#[test]
fn late_start_fade_keeps_each_source_once() {
    // Capture both real schedules before checking any frame behavior. The zero-dt
    // arm is a control, not a substitute for freezing on positive-dt world ticks.
    let schedules = [("positive_pause", STEP), ("zero_step_control", 0.0)];
    let observed = schedules.map(|(arm, paused_delta)| observe(arm, paused_delta));
    for ((arm, paused_delta), (entity, epoch, frames)) in schedules.into_iter().zip(observed) {
        expect_frame(arm, 1, &frames[0], epoch, true, "Idle", 0.25, None);
        expect_frame(arm, 2, &frames[1], epoch, true, "Idle", 0.5, None);
        expect_frame(
            arm,
            3,
            &frames[2],
            epoch,
            true,
            "Idle",
            0.625,
            Some(half_fade()),
        );
        let holds = paused_delta == 0.0 || frames[3].transition.is_some();
        if holds {
            expect_frame(
                arm,
                4,
                &frames[3],
                epoch,
                false,
                "Idle",
                0.625,
                Some(half_fade()),
            );
        } else {
            expect_frame(arm, 4, &frames[3], epoch, false, "Run", 0.625, None);
        }
        expect_frame(arm, 5, &frames[4], epoch, true, "Run", 0.75, None);
        expect_frame(arm, 6, &frames[5], epoch, true, "Run", 0.875, None);

        // Completion owes one point from each local range. The root-window point
        // is legitimate for an already-steady Run, but cannot stand in for either
        // fade point. Steady playback should still use the documented root clock.
        let fade_points = [IDLE_MID, RUN_MID];
        let expected: [&[Point]; 6] = if holds {
            [&[], &[], &[], &[], &fade_points, &[RUN_STEADY]]
        } else {
            [
                &[],
                &[],
                &[],
                &fade_points,
                &[RUN_AT_RESUME_ROOT],
                &[RUN_STEADY],
            ]
        };
        for (index, (frame, points)) in frames.iter().zip(expected).enumerate() {
            expect_points(arm, index + 1, frame, entity, points);
        }
    }
}

fn observe(arm: &str, paused_delta: Real) -> (EntityId, u64, [Frame; 6]) {
    let runtime = runtime_with_physics_animation_scene_asset();
    let assets = runtime_asset_manager(&runtime.handle());
    let resources = assets.resource_manager();
    let skeleton = uri("pause-late.skeleton");
    let skeleton_id = ResourceId::from_locator(&skeleton);
    resources
        .register_ready(
            ResourceRecord::new(
                skeleton_id,
                ResourceKind::AnimationSkeleton,
                skeleton.clone(),
            ),
            two_bone_skeleton(),
        )
        .unwrap();
    let register_graph = |state: &str, x: Real, points: &[Point]| {
        let graph = uri(&format!("pause-late-{state}.graph"));
        let clip = uri(&format!("pause-late-{state}.clip"));
        let mut asset = single_hand_translation_clip(&skeleton, x);
        asset
            .event_tracks
            .extend(
                points
                    .iter()
                    .map(|&(name, payload, time)| AnimationEventTrackAsset {
                        target_id: Some(TARGET.into()),
                        event: name.into(),
                        time_seconds: time,
                        payload: Some(payload.into()),
                    }),
            );
        resources
            .register_ready(
                ResourceRecord::new(
                    ResourceId::from_locator(&clip),
                    ResourceKind::AnimationClip,
                    clip.clone(),
                ),
                asset,
            )
            .unwrap();
        register_single_clip_graph(&assets, &graph, &clip);
        graph
    };
    let idle = register_graph("idle", 0.0, &[IDLE_MID]);
    let run = register_graph("run", 10.0, &[RUN_MID, RUN_AT_RESUME_ROOT, RUN_STEADY]);
    let machine = uri("pause-late.machine");
    let machine_id = ResourceId::from_locator(&machine);
    let mut authored = timed_transition_state_machine(&idle, &run);
    authored.transitions[0].duration_seconds = 0.25;
    resources
        .register_ready(
            ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine),
            authored,
        )
        .unwrap();
    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        machine_id,
        BTreeMap::from([("advance".to_string(), AnimationParameterValue::Bool(false))]),
    );
    let epoch = level.capture_world_replacement_epoch();
    let mut clip_events = subscribe_animation_clip_events(&level);
    let mut neutral_events = level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<AnimationEventRecord>();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    });

    let mut index = 0;
    let frames = [
        // The default World virtual clock caps a single 0.5s input at 0.25s.
        ("warm_1", 0.25, None),
        ("warm_2", 0.25, None),
        ("open_edge", STEP, None),
        ("pause", paused_delta, Some(false)),
        ("resume", STEP, Some(true)),
        ("steady", STEP, None),
    ]
    .map(|(action, delta, playing)| {
        index += 1;
        if action == "open_edge" || playing.is_some() {
            level.with_world_mut(|world| {
                let mut player = world
                    .animation_state_machine_player(entity)
                    .unwrap()
                    .clone();
                if action == "open_edge" {
                    assert!(player
                        .parameters
                        .update_existing("advance", AnimationParameterValue::Bool(true)));
                }
                if let Some(playing) = playing {
                    player.playing = playing;
                }
                assert!(world
                    .set_animation_state_machine_player(entity, Some(player))
                    .unwrap());
            });
        }
        let backlog_before = level.animation_clip_event_backlog_len(epoch);
        runtime
            .tick_level_seconds(&level, f64::from(delta))
            .unwrap();
        let backlog_after = level.animation_clip_event_backlog_len(epoch);
        let (root, transition) = level.animation_playback_times(epoch).map_or(
            (None, None),
            |(_, times, transitions)| {
                (
                    times.get(&entity).copied(),
                    transitions.get(&entity).cloned(),
                )
            },
        );
        let (playing, state) = level.with_world(|world| {
            let player = world.animation_state_machine_player(entity).unwrap();
            (player.playing, player.active_state.clone())
        });
        let typed = drain_animation_clip_events(&level, &mut clip_events);
        let neutral = level.with_world_mut(|world| {
            world.update_events::<AnimationEventRecord>();
            world
                .read_event_subscription(&mut neutral_events)
                .cloned()
                .collect()
        });
        let frame = Frame {
            epoch: level.capture_world_replacement_epoch(),
            playing,
            state,
            root,
            transition,
            backlog_before,
            backlog_after,
            typed,
            neutral,
        };
        println!(
            "PAUSE_LATE_START_V1 arm={arm} frame={index} action={action} dt={delta:?} {frame:?}"
        );
        frame
    });
    (entity, epoch, frames)
}

fn half_fade() -> AnimationStateTransitionRuntime {
    AnimationStateTransitionRuntime {
        from_state: "Idle".into(),
        to_state: "Run".into(),
        duration_seconds: 0.25,
        elapsed_seconds: STEP,
        from_time_seconds: 0.625,
        to_time_seconds: STEP,
    }
}

#[allow(clippy::too_many_arguments)]
fn expect_frame(
    arm: &str,
    number: usize,
    frame: &Frame,
    epoch: u64,
    playing: bool,
    active: &str,
    root: Real,
    transition: Option<AnimationStateTransitionRuntime>,
) {
    let label = format!("{arm} F{number}");
    assert_eq!(frame.epoch, epoch, "{label}");
    assert_eq!(frame.playing, playing, "{label}");
    assert_eq!(frame.state.as_deref(), Some(active), "{label}");
    assert_eq!(frame.root, Some(root), "{label}");
    assert_eq!(frame.transition, transition, "{label}");
    assert_eq!(
        (frame.backlog_before, frame.backlog_after),
        (Some(0), Some(0)),
        "{label}"
    );
}

fn expect_points(arm: &str, number: usize, frame: &Frame, entity: EntityId, points: &[Point]) {
    let label = format!("{arm} F{number} points={points:?}");
    assert_eq!(
        frame.typed.len(),
        points.len(),
        "{label} typed={:?}",
        frame.typed
    );
    assert_eq!(
        frame.neutral.len(),
        points.len(),
        "{label} neutral={:?}",
        frame.neutral
    );
    for &(name, payload, time) in points {
        let typed = AnimationClipEvent {
            entity,
            target_id: Some(TARGET.into()),
            event: name.into(),
            payload: Some(payload.into()),
            clip_time_seconds: time,
            playback_time_seconds: time,
        };
        let neutral = AnimationEventRecord::new(entity, name)
            .with_target_id(TARGET)
            .with_payload(payload)
            .at_times(time, time);
        assert!(frame.typed.contains(&typed), "{label}: missing {typed:?}");
        assert!(
            frame.neutral.contains(&neutral),
            "{label}: missing {neutral:?}"
        );
    }
}
