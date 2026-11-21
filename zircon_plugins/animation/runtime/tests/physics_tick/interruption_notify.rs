// A state keeps the notifications from its old fade window, not a second scan
// when the same playback becomes the source of a new fade.
use std::collections::BTreeMap;

use zircon_runtime::asset::{AssetReference, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationClipEvent, AnimationEventRecord, AnimationEventTrackAsset, AnimationParameterValue,
    AnimationStateMachineLayerAsset, AnimationStateMachineLayerBlendModeAsset,
    AnimationTransitionInterruptionPolicyAsset,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};
use zircon_runtime::scene::{EntityId, EventSubscription, LevelSystem};

use super::state_machine_interruption::{assert_hand_translation, spawn_state_machine_player, uri};
use super::{
    drain_animation_clip_events, interruptible_transition_state_machine,
    register_single_clip_graph, runtime_asset_manager, runtime_with_physics_animation_scene_asset,
    single_hand_translation_clip, single_state_machine, subscribe_animation_clip_events,
    two_bone_skeleton,
};

#[test]
fn root_next_source_window_is_sampled_once() {
    run_notify_case(Fade::FromRun, false);
}

#[test]
fn root_current_source_window_is_sampled_once() {
    run_notify_case(Fade::FromIdle, false);
}

#[test]
fn root_return_target_restarts_without_rescanning_source() {
    run_notify_case(Fade::BackToIdle, false);
}

#[test]
fn continued_root_keeps_each_source_occurrence() {
    run_notify_case(Fade::Continue, false);
}

#[test]
fn layer_next_source_window_is_sampled_once() {
    run_notify_case(Fade::FromRun, true);
}

#[derive(Clone, Copy)]
enum Fade {
    Continue,
    FromRun,
    FromIdle,
    BackToIdle,
}

type Notify = (&'static str, Real);

const IDLE_ENTRY: Notify = ("idle_entry", 0.125);
const RUN_ENTRY: Notify = ("run_entry", 0.125);
const SPRINT_ENTRY: Notify = ("sprint_entry", 0.125);
const IDLE_WINDOW: Notify = ("idle_window", 0.375);
const RUN_WINDOW: Notify = ("run_window", 0.375);
const SHARED: Notify = ("shared", 0.5);
const IDLE_NEXT: Notify = ("idle_next", 0.625);
const RUN_NEXT: Notify = ("run_next", 0.625);

fn run_notify_case(fade: Fade, layered: bool) {
    let runtime = runtime_with_physics_animation_scene_asset();
    let assets = runtime_asset_manager(&runtime.handle());
    let (skeleton, machine) = register_fixture(&assets, fade, layered);
    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton,
        machine,
        BTreeMap::from([
            ("start".to_string(), AnimationParameterValue::Bool(true)),
            (
                "interrupt".to_string(),
                AnimationParameterValue::Bool(false),
            ),
        ]),
    );
    let mut streams = NotifyStreams::subscribe(&level);

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    if matches!(fade, Fade::BackToIdle) {
        // Let the root and Run clocks diverge using an ordinary exit-time gate.
        let (_, times, transitions) = level
            .animation_playback_times(level.capture_world_replacement_epoch())
            .unwrap();
        assert_eq!(times[&entity], 0.25);
        assert!(!transitions.contains_key(&entity));
        streams.record_frame(&level, entity, "before-exit", &[IDLE_ENTRY]);

        runtime.tick_level_seconds(&level, 0.25).unwrap();
        assert_hand_translation(&level, entity, 0.0);
        assert_transition(&level, entity, 0.5, ("Idle", "Run", 0.5, 0.0, 0.0));
        streams.record_frame(&level, entity, "exit-gate", &[IDLE_WINDOW, SHARED]);
    } else {
        assert_hand_translation(&level, entity, 2.5);
        if !layered {
            assert_transition(&level, entity, 0.25, ("Idle", "Run", 0.25, 0.25, 0.25));
        }
        streams.record_frame(&level, entity, "initial", &[IDLE_ENTRY, RUN_ENTRY]);
    }

    if !matches!(fade, Fade::Continue) {
        level.with_world_mut(|world| {
            let mut player = world
                .animation_state_machine_player(entity)
                .unwrap()
                .clone();
            player
                .parameters
                .insert("start".to_string(), AnimationParameterValue::Bool(false));
            player
                .parameters
                .insert("interrupt".to_string(), AnimationParameterValue::Bool(true));
            assert!(world
                .set_animation_state_machine_player(entity, Some(player))
                .unwrap());
        });
    }

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    let (playback, transition, hand_x, notifications): (_, _, _, &[Notify]) = match fade {
        Fade::Continue => (
            0.5,
            ("Idle", "Run", 0.5, 0.5, 0.5),
            5.0,
            &[IDLE_WINDOW, RUN_WINDOW, SHARED, SHARED],
        ),
        Fade::FromRun => (
            0.5,
            ("Run", "Sprint", 0.5, 0.0, 0.0),
            5.0,
            &[IDLE_WINDOW, RUN_WINDOW, SHARED, SHARED],
        ),
        Fade::FromIdle => (
            0.5,
            ("Idle", "Sprint", 0.5, 0.0, 0.0),
            5.0,
            &[IDLE_WINDOW, RUN_WINDOW, SHARED, SHARED],
        ),
        // Run has only reached 0.25; a range starting at root time 0.5 would
        // wrongly travel backward through RUN_WINDOW before it is crossed.
        Fade::BackToIdle => (
            0.75,
            ("Run", "Idle", 0.25, 0.0, 0.0),
            2.5,
            &[IDLE_NEXT, RUN_ENTRY],
        ),
    };
    assert_hand_translation(&level, entity, hand_x);
    if !layered {
        assert_transition(&level, entity, playback, transition);
    }
    // Identical records from Idle and Run are two legal occurrences. Compare
    // both streams together so a baseline failure reports both observations.
    streams.record_frame(&level, entity, "decision", notifications);

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    let (transition, hand_x, notifications): (_, _, &[Notify]) = match fade {
        Fade::Continue => (
            ("Idle", "Run", 0.75, 0.75, 0.75),
            7.5,
            &[IDLE_NEXT, RUN_NEXT],
        ),
        Fade::FromRun => (
            ("Run", "Sprint", 0.75, 0.25, 0.25),
            8.75,
            &[RUN_NEXT, SPRINT_ENTRY],
        ),
        Fade::FromIdle => (
            ("Idle", "Sprint", 0.75, 0.25, 0.25),
            8.75,
            &[IDLE_NEXT, SPRINT_ENTRY],
        ),
        // Idle starts a new target playback from zero; the same IDLE_ENTRY
        // from the first frame is now a legitimate new occurrence.
        Fade::BackToIdle => (
            ("Run", "Idle", 0.5, 0.25, 0.25),
            1.875,
            &[IDLE_ENTRY, RUN_WINDOW, SHARED],
        ),
    };
    assert_hand_translation(&level, entity, hand_x);
    if !layered {
        assert_transition(&level, entity, playback + 0.25, transition);
    }
    streams.record_frame(&level, entity, "next", notifications);
    // Delay only the event assertions until all genuine frames have been observed.
    streams.assert_frames();
}

fn register_fixture(
    assets: &ProjectAssetManager,
    fade: Fade,
    layered: bool,
) -> (ResourceId, ResourceId) {
    let skeleton_uri = uri("interrupt-notify.skeleton");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let rm = assets.resource_manager();
    rm.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    let graph_uris = [
        (
            "idle",
            0.0,
            &[IDLE_ENTRY, IDLE_WINDOW, SHARED, IDLE_NEXT][..],
        ),
        ("run", 10.0, &[RUN_ENTRY, RUN_WINDOW, SHARED, RUN_NEXT][..]),
        ("sprint", 20.0, &[SPRINT_ENTRY][..]),
    ]
    .map(|(name, x, tracks)| {
        let clip_uri = uri(&format!("interrupt-notify-{name}.clip"));
        let graph_uri = uri(&format!("interrupt-notify-{name}.graph"));
        let mut clip = single_hand_translation_clip(&skeleton_uri, x);
        clip.event_tracks = tracks
            .iter()
            .map(|&(event, time_seconds)| AnimationEventTrackAsset {
                target_id: Some("Root/Hand".to_string()),
                event: event.to_string(),
                time_seconds,
                payload: Some("0.75".to_string()),
            })
            .collect();
        rm.register_ready(
            ResourceRecord::new(
                ResourceId::from_locator(&clip_uri),
                ResourceKind::AnimationClip,
                clip_uri.clone(),
            ),
            clip,
        );
        register_single_clip_graph(assets, &graph_uri, &clip_uri);
        graph_uri
    });
    let mut machine =
        interruptible_transition_state_machine(&graph_uris[0], &graph_uris[1], &graph_uris[2]);
    if matches!(fade, Fade::FromIdle) {
        machine.transitions[0].interruption =
            AnimationTransitionInterruptionPolicyAsset::CurrentToNext;
        machine.transitions[1].from_state = "Idle".to_string();
    } else if matches!(fade, Fade::BackToIdle) {
        machine.transitions[0].exit_time = Some(0.5);
        machine.transitions[1].to_state = "Idle".to_string();
    }
    let machine_uri = uri("interrupt-notify.machine");
    let machine_id = ResourceId::from_locator(&machine_uri);
    rm.register_ready(
        ResourceRecord::new(
            machine_id,
            ResourceKind::AnimationStateMachine,
            machine_uri.clone(),
        ),
        machine,
    );
    if !layered {
        return (skeleton_id, machine_id);
    }

    // Do not reuse Idle's clip in the wrapper: its independent root events
    // would be indistinguishable from the layer's equally valid events.
    let base_clip = uri("interrupt-notify-base.clip");
    let base_graph = uri("interrupt-notify-base.graph");
    let base_uri = uri("interrupt-notify-base.machine");
    let base_id = ResourceId::from_locator(&base_uri);
    rm.register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&base_clip),
            ResourceKind::AnimationClip,
            base_clip.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 0.0),
    );
    register_single_clip_graph(assets, &base_graph, &base_clip);
    let mut base = single_state_machine(&base_graph);
    base.layers.push(AnimationStateMachineLayerAsset {
        name: "upper".to_string(),
        state_machine: AssetReference::from_locator(machine_uri),
        weight: 1.0,
        blend_mode: AnimationStateMachineLayerBlendModeAsset::Override,
        mask_weights: vec![0.0, 1.0],
    });
    rm.register_ready(
        ResourceRecord::new(base_id, ResourceKind::AnimationStateMachine, base_uri),
        base,
    );
    (skeleton_id, base_id)
}

fn assert_transition(
    level: &LevelSystem,
    entity: EntityId,
    playback: Real,
    expected: (&str, &str, Real, Real, Real),
) {
    let (_, times, transitions) = level
        .animation_playback_times(level.capture_world_replacement_epoch())
        .unwrap();
    assert_eq!(times[&entity], playback);
    let actual = transitions.get(&entity).expect("live root fade");
    assert_eq!(
        (
            actual.from_state.as_str(),
            actual.to_state.as_str(),
            actual.from_time_seconds,
            actual.to_time_seconds,
            actual.elapsed_seconds
        ),
        expected,
    );
}

struct NotifyStreams {
    typed: EventSubscription<AnimationClipEvent>,
    neutral: EventSubscription<AnimationEventRecord>,
    frames: Vec<NotifyFrame>,
}

struct NotifyFrame {
    name: &'static str,
    typed: Vec<AnimationClipEvent>,
    neutral: Vec<AnimationClipEvent>,
    expected: Vec<AnimationClipEvent>,
}

fn trace_event(event: &AnimationClipEvent) -> serde_json::Value {
    serde_json::json!([
        event.entity,
        event.target_id.as_deref(),
        event.event.as_str(),
        event.payload.as_deref(),
        event.clip_time_seconds,
        event.playback_time_seconds,
    ])
}

impl NotifyStreams {
    fn subscribe(level: &LevelSystem) -> Self {
        let typed = subscribe_animation_clip_events(level);
        let neutral = level.with_world_mut(|world| {
            let mut subscription =
                world.register_dormant_event_subscription::<AnimationEventRecord>();
            assert!(world.connect_event_subscription(&mut subscription));
            subscription
        });
        Self {
            typed,
            neutral,
            frames: Vec::new(),
        }
    }

    fn record_frame(
        &mut self,
        level: &LevelSystem,
        entity: EntityId,
        frame: &'static str,
        events: &[Notify],
    ) {
        let mut typed = drain_animation_clip_events(level, &mut self.typed);
        let mut neutral: Vec<_> = level.with_world_mut(|world| {
            world.update_events::<AnimationEventRecord>();
            world
                .read_event_subscription(&mut self.neutral)
                .map(|event| AnimationClipEvent {
                    entity: event.entity,
                    target_id: event.target_id.clone(),
                    event: event.name.clone(),
                    payload: event.payload.clone(),
                    clip_time_seconds: event.clip_time_seconds,
                    playback_time_seconds: event.playback_time_seconds,
                })
                .collect()
        });
        let mut expected: Vec<_> = events
            .iter()
            .map(|&(event, time)| AnimationClipEvent {
                entity,
                target_id: Some("Root/Hand".to_string()),
                event: event.to_string(),
                payload: Some("0.75".to_string()),
                clip_time_seconds: time,
                playback_time_seconds: time,
            })
            .collect();
        // Ordering independent clip sources is not part of this contract.
        // Sort without deduplication and do not filter out unexpected records.
        for stream in [&mut typed, &mut neutral, &mut expected] {
            stream.sort_by(|a, b| {
                a.event
                    .cmp(&b.event)
                    .then_with(|| a.clip_time_seconds.total_cmp(&b.clip_time_seconds))
            });
        }
        let backlog = level
            .animation_clip_event_backlog_len(level.capture_world_replacement_epoch())
            .expect("current animation event backlog");
        assert_eq!(backlog, 0, "{frame}: ordinary frame must drain its ranges");
        println!(
            "r33_171_frame {}",
            serde_json::json!({
                "frame": frame,
                "entity": entity,
                "backlog": backlog,
                "typed": typed.iter().map(trace_event).collect::<Vec<_>>(),
                "neutral": neutral.iter().map(trace_event).collect::<Vec<_>>(),
            })
        );
        self.frames.push(NotifyFrame {
            name: frame,
            typed,
            neutral,
            expected,
        });
    }

    fn assert_frames(self) {
        for frame in self.frames {
            assert_eq!(
                (&frame.typed, &frame.neutral),
                (&frame.expected, &frame.expected),
                "r33-171-event-cardinality: {} (typed, neutral)",
                frame.name,
            );
        }
    }
}
