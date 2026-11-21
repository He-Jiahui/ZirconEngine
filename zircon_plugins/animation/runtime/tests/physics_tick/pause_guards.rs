//! Real D11 world/asset controls for retaining a paused transition.
use super::{
    drain_animation_clip_events, register_single_clip_graph, runtime_asset_manager,
    runtime_with_physics_animation_scene_asset, single_hand_translation_clip,
    state_machine_interruption::{assert_hand_translation, spawn_state_machine_player, uri},
    subscribe_animation_clip_events, timed_transition_state_machine, two_bone_skeleton,
    AnimationEventTrackAsset, AnimationParameterValue, AnimationStateMachineMarker,
    AnimationStateMachinePlayerComponent, AssetUri, BTreeMap, ResourceHandle, ResourceId,
    ResourceKind, ResourceRecord,
};
use crate::runtime_helpers::ScopedTestRuntime;
use zircon_plugin_animation_runtime::AnimationClipEvent;
use zircon_runtime::core::framework::animation::{
    AnimationEventRecord, AnimationStateMachineAsset,
};
use zircon_runtime::core::resource::ResourceState;
use zircon_runtime::scene::components::PointLight;
use zircon_runtime::scene::ecs::{RemovedComponentsParam, SystemState};
use zircon_runtime::scene::{
    AnimationStateTransitionRuntime, EntityId, EventSubscription, LevelSystem,
};

const STEP: f32 = 0.125;
const POINT: f32 = 0.1875;
const TAG: &str = "run_mid";

struct Fixture {
    level: LevelSystem,
    runtime: ScopedTestRuntime,
    entity: EntityId,
    epoch: u64,
    machine: AssetUri,
    run_graph: AssetUri,
    revised_idle: AssetUri,
    clip_events: EventSubscription<AnimationClipEvent>,
    neutral_events: EventSubscription<AnimationEventRecord>,
}

#[derive(Debug)]
struct Frame {
    playing: bool,
    state: Option<String>,
    time: f32,
    transition: Option<AnimationStateTransitionRuntime>,
    counts: (usize, usize),
}

impl Fixture {
    fn new() -> Self {
        let runtime = runtime_with_physics_animation_scene_asset();
        let assets = runtime_asset_manager(&runtime.handle());
        let skeleton = uri("pause-guard.skeleton");
        let skeleton_id = ResourceId::from_locator(&skeleton);
        let resources = assets.resource_manager();
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
        let register_graph = |label: &str, hand_x: f32, has_point: bool| {
            let clip = uri(&format!("pause-guard-{label}.clip"));
            let graph = uri(&format!("pause-guard-{label}.graph"));
            let mut payload = single_hand_translation_clip(&skeleton, hand_x);
            if has_point {
                payload.event_tracks.push(AnimationEventTrackAsset {
                    target_id: Some("Root/Hand".into()),
                    event: TAG.into(),
                    time_seconds: POINT,
                    payload: Some("guard".into()),
                });
            }
            resources
                .register_ready(
                    ResourceRecord::new(
                        ResourceId::from_locator(&clip),
                        ResourceKind::AnimationClip,
                        clip.clone(),
                    ),
                    payload,
                )
                .unwrap();
            register_single_clip_graph(&assets, &graph, &clip);
            graph
        };
        let idle = register_graph("idle", 0.0, false);
        let run_graph = register_graph("run", 10.0, true);
        let revised_idle = register_graph("revised-idle", 6.0, false);
        let machine = uri("pause-guard.machine");
        let machine_id = ResourceId::from_locator(&machine);
        let mut model = timed_transition_state_machine(&idle, &run_graph);
        model.transitions[0].duration_seconds = 0.25;
        resources
            .register_ready(
                ResourceRecord::new(
                    machine_id,
                    ResourceKind::AnimationStateMachine,
                    machine.clone(),
                )
                .with_source_hash("pause-guard-machine-v1"),
                model,
            )
            .unwrap();
        drop(assets);
        let level = runtime.create_default_level().unwrap();
        let entity = spawn_state_machine_player(
            &level,
            skeleton_id,
            machine_id,
            BTreeMap::from([("advance".to_string(), AnimationParameterValue::Bool(true))]),
        );
        let epoch = level.capture_world_replacement_epoch();
        let clip_events = subscribe_animation_clip_events(&level);
        let neutral_events = level.with_world_mut(|world| {
            let mut subscription =
                world.register_dormant_event_subscription::<AnimationEventRecord>();
            assert!(world.connect_event_subscription(&mut subscription));
            subscription
        });
        Self {
            level,
            runtime,
            entity,
            epoch,
            machine,
            run_graph,
            revised_idle,
            clip_events,
            neutral_events,
        }
    }

    fn tick(&mut self, delta: f32) -> Frame {
        assert_eq!(
            self.level.animation_clip_event_backlog_len(self.epoch),
            Some(0)
        );
        self.runtime
            .tick_level_seconds(&self.level, f64::from(delta))
            .unwrap();
        assert_eq!(self.level.capture_world_replacement_epoch(), self.epoch);
        assert_eq!(
            self.level.animation_clip_event_backlog_len(self.epoch),
            Some(0)
        );
        let (_, times, transitions) = self.level.animation_playback_times(self.epoch).unwrap();
        let (playing, state) = self.level.with_world(|world| {
            let p = world.animation_state_machine_player(self.entity).unwrap();
            (p.playing, p.active_state.clone())
        });
        let typed = drain_animation_clip_events(&self.level, &mut self.clip_events);
        let neutral = self.level.with_world_mut(|world| {
            world.update_events::<AnimationEventRecord>();
            world
                .read_event_subscription(&mut self.neutral_events)
                .cloned()
                .collect::<Vec<_>>()
        });
        let frame = Frame {
            playing,
            state,
            time: times[&self.entity],
            transition: transitions.get(&self.entity).cloned(),
            counts: (typed.len(), neutral.len()),
        };
        println!("PAUSE_GUARD_V1 dt={delta:?} {frame:?} typed={typed:?} neutral={neutral:?}");
        for event in typed {
            assert_eq!(
                (
                    event.entity,
                    event.event.as_str(),
                    event.target_id.as_deref(),
                    event.payload.as_deref(),
                    event.clip_time_seconds,
                    event.playback_time_seconds
                ),
                (
                    self.entity,
                    TAG,
                    Some("Root/Hand"),
                    Some("guard"),
                    POINT,
                    POINT
                )
            );
        }
        let expected = AnimationEventRecord::new(self.entity, TAG)
            .with_target_id("Root/Hand")
            .with_payload("guard")
            .at_times(POINT, POINT);
        assert!(neutral.iter().all(|event| event == &expected));
        frame
    }

    fn playing(&self, value: bool, disable_edge: bool) {
        self.level.with_world_mut(|world| {
            let mut p = world
                .animation_state_machine_player(self.entity)
                .unwrap()
                .clone();
            p.playing = value;
            if disable_edge {
                assert!(p
                    .parameters
                    .update_existing("advance", AnimationParameterValue::Bool(false)));
            }
            world
                .set_animation_state_machine_player(self.entity, Some(p))
                .unwrap();
        });
    }

    fn edit_player(&self, real_remove: bool) {
        self.level.with_world_mut(|world| {
            type Removed =
                SystemState<RemovedComponentsParam<AnimationStateMachinePlayerComponent>>;
            let p = world
                .animation_state_machine_player(self.entity)
                .unwrap()
                .clone();
            let old = world
                .component_change_ticks::<AnimationStateMachinePlayerComponent>(self.entity)
                .unwrap()
                .added();
            let mut removed = Removed::new(world).unwrap();
            assert!(!removed.run(world, |mut stream| stream
                .read()
                .any(|id| id == self.entity)));
            assert!(!world.contains_component::<PointLight>(self.entity));
            if real_remove {
                assert!(world
                    .set_animation_state_machine_player(self.entity, None)
                    .unwrap());
                assert!(world.animation_state_machine_player(self.entity).is_none());
                assert!(world
                    .set_animation_state_machine_player(self.entity, Some(p.clone()))
                    .unwrap());
            }
            // Both arms have the same final values and topology. The mixed bundle
            // refreshes Added even though this operation removes no player.
            world
                .commands()
                .insert_bundle(self.entity, (p.clone(), PointLight::default()));
            let result = world.apply_deferred();
            assert!(result.is_success(), "{result:?}");
            assert!(world.contains_component::<PointLight>(self.entity));
            assert_eq!(world.animation_state_machine_player(self.entity), Some(&p));
            let new = world
                .component_change_ticks::<AnimationStateMachinePlayerComponent>(self.entity)
                .unwrap()
                .added();
            assert_ne!(new, old);
            let did_remove = removed.run(world, |mut stream| {
                stream.read().any(|id| id == self.entity)
            });
            assert_eq!(did_remove, real_remove);
            println!("PAUSE_GUARD_EDIT Removed={did_remove} Added={old:?}->{new:?}");
        });
    }

    fn publish_v2(&self, keep_run: bool) {
        let assets = runtime_asset_manager(&self.runtime.handle());
        let resources = assets.resource_manager();
        let id = ResourceId::from_locator(&self.machine);
        let snapshot = || {
            resources
                .snapshot::<AnimationStateMachineMarker, AnimationStateMachineAsset>(
                    ResourceHandle::new(id),
                )
                .unwrap()
        };
        let before = snapshot();
        assert_eq!(before.record().state, ResourceState::Ready);
        assert_eq!(
            before
                .states
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["Idle", "Run"]
        );
        // The V2 graph was already Ready. In the compatible arm, no parameter
        // or component change can disguise restarting the disabled edge.
        let mut v2 = timed_transition_state_machine(&self.revised_idle, &self.run_graph);
        v2.transitions[0].duration_seconds = 0.25;
        if !keep_run {
            v2.states.truncate(1);
            v2.transitions.clear();
        }
        resources
            .register_ready(
                ResourceRecord::new(
                    id,
                    ResourceKind::AnimationStateMachine,
                    self.machine.clone(),
                )
                .with_source_hash("pause-guard-machine-v2"),
                v2,
            )
            .unwrap();
        let after = snapshot();
        assert_eq!(after.record().id, before.record().id);
        assert_eq!(
            after.record().primary_locator,
            before.record().primary_locator
        );
        assert_eq!(after.record().state, ResourceState::Ready);
        assert_eq!(after.revision(), before.revision() + 1);
        let names = after
            .states
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            if keep_run {
                vec!["Idle", "Run"]
            } else {
                vec!["Idle"]
            }
        );
        println!(
            "PAUSE_GUARD_READY revision={}->{} states={names:?}",
            before.revision(),
            after.revision()
        );
    }
}

fn phase(frame: &Frame, playing: bool, clock: f32, in_flight: bool, state: &str, points: usize) {
    assert_eq!(frame.playing, playing);
    assert_eq!(frame.time, clock);
    assert_eq!(frame.state.as_deref(), Some(state));
    assert_eq!(frame.counts, (points, points));
    let expected = in_flight.then(|| AnimationStateTransitionRuntime {
        from_state: "Idle".into(),
        to_state: "Run".into(),
        duration_seconds: 0.25,
        elapsed_seconds: STEP,
        from_time_seconds: STEP,
        to_time_seconds: STEP,
    });
    assert_eq!(frame.transition, expected);
}

fn hold(f: &mut Fixture) {
    phase(&f.tick(STEP), true, STEP, true, "Idle", 0);
    f.playing(false, true);
    // Both baseline and a freezing implementation still have live T on this real zero-dt tick.
    phase(&f.tick(0.0), false, STEP, true, "Idle", 0);
    assert_hand_translation(&f.level, f.entity, 5.0);
    let clean = f.tick(STEP); // no setter, graph/asset mutation or replacement on this tick
    assert_eq!(
        (clean.playing, clean.time, clean.counts),
        (false, STEP, (0, 0))
    );
}

#[test]
fn pause_guard_normal_playing_mixed_bundle_does_not_restart_fade() {
    let mut f = Fixture::new();
    phase(&f.tick(STEP), true, STEP, true, "Idle", 0);
    f.edit_player(false);
    phase(&f.tick(STEP), true, 0.25, false, "Run", 1);
    assert_hand_translation(&f.level, f.entity, 10.0);
    phase(&f.tick(STEP), true, 0.375, false, "Run", 0);
}

fn edit_held_player(remove: bool) {
    let mut f = Fixture::new();
    hold(&mut f);
    f.edit_player(remove);
    phase(&f.tick(STEP), false, STEP, !remove, "Idle", 0);
    assert_hand_translation(&f.level, f.entity, if remove { 0.0 } else { 5.0 });
    f.playing(true, false);
    phase(
        &f.tick(STEP),
        true,
        0.25,
        false,
        if remove { "Idle" } else { "Run" },
        usize::from(!remove),
    );
}

#[test]
fn pause_guard_held_mixed_bundle_retains_the_existing_fade() {
    edit_held_player(false);
}

#[test]
fn pause_guard_real_remove_then_add_never_inherits_the_held_fade() {
    edit_held_player(true);
}

fn reload_held_machine(keep_run: bool) {
    let mut f = Fixture::new();
    hold(&mut f);
    f.publish_v2(keep_run);
    phase(&f.tick(STEP), false, STEP, keep_run, "Idle", 0);
    assert_hand_translation(&f.level, f.entity, if keep_run { 8.0 } else { 6.0 });
    f.playing(true, false);
    phase(
        &f.tick(STEP),
        true,
        0.25,
        false,
        if keep_run { "Run" } else { "Idle" },
        usize::from(keep_run),
    );
}

#[test]
fn pause_guard_same_id_ready_both_endpoints_adopts_pose_and_keeps_weight() {
    reload_held_machine(true);
}

#[test]
fn pause_guard_same_id_ready_lost_endpoint_retires_held_fade() {
    reload_held_machine(false);
}
