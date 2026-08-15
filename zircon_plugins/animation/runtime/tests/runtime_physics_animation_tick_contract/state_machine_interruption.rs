use super::*;
use zircon_runtime::scene::AnimationStateTransitionRuntime;

#[test]
fn pose_targets_visible_to_physics_step() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("pose-bridge.skeleton");
    let clip_uri = uri("pose-bridge.clip");
    let graph_uri = uri("pose-bridge.graph");
    let machine_uri = uri("pose-bridge.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    let unused_clip_uri = uri("pose-bridge-unused.clip");
    register_animation_blend_assets(&asset_manager, &skeleton_uri, &clip_uri, &unused_clip_uri);
    register_single_clip_graph(&asset_manager, &graph_uri, &clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        single_state_machine(&graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(&level, skeleton_id, machine_id, BTreeMap::new());
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    level.with_world(|world| {
        let targets =
            world.resource::<zircon_runtime::core::framework::physics::SkeletalPoseTargets>();
        let hand = targets
            .targets(entity)
            .and_then(|targets| targets.iter().find(|target| target.bone_name == "Hand"))
            .expect("animation pose target visible to physics FixedUpdate");
        assert_eq!(hand.normalized_weight, 1.0);
        assert!(hand.local_transform.translation.is_finite());
    });
}

#[test]
fn simulated_pose_blends_under_ragdoll_mask() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("simulated-feed.skeleton");
    let clip_uri = uri("simulated-feed.clip");
    let graph_uri = uri("simulated-feed.graph");
    let machine_uri = uri("simulated-feed.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    let unused_clip_uri = uri("simulated-feed-unused.clip");
    register_animation_blend_assets(&asset_manager, &skeleton_uri, &clip_uri, &unused_clip_uri);
    register_single_clip_graph(&asset_manager, &graph_uri, &clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        single_state_machine(&graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(&level, skeleton_id, machine_id, BTreeMap::new());
    level.with_world_mut(|world| {
        let feed =
            world.resource_mut::<zircon_runtime::core::framework::physics::SimulatedPoseFeed>();
        feed.replace(
            entity,
            std::sync::Arc::from([
                zircon_runtime::core::framework::physics::SkeletalPoseTarget {
                    bone_name: "Hand".to_string(),
                    local_transform: Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
                    normalized_weight: 0.25,
                },
            ]),
        );
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 2.5);
    level.with_world(|world| {
        let published = world
            .resource::<zircon_runtime::core::framework::physics::SkeletalPoseTargets>()
            .targets(entity)
            .and_then(|targets| targets.iter().find(|target| target.bone_name == "Hand"))
            .expect("blended simulated pose should be published as the next physics target");
        assert!(published
            .local_transform
            .translation
            .abs_diff_eq(Vec3::new(2.5, 0.0, 0.0), 1.0e-4));
    });
}

#[test]
fn level_tick_blends_state_machine_transition_until_duration_completes() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("transition.skeleton");
    let idle_clip_uri = uri("transition-idle.clip");
    let run_clip_uri = uri("transition-run.clip");
    let idle_graph_uri = uri("transition-idle.graph");
    let run_graph_uri = uri("transition-run.graph");
    let machine_uri = uri("transition.state_machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        timed_transition_state_machine(&idle_graph_uri, &run_graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        machine_id,
        BTreeMap::from([("advance".to_string(), AnimationParameterValue::Bool(true))]),
    );

    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert_hand_translation(&level, entity, 5.0);
    assert_active_state(&level, entity, "Idle");

    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert_hand_translation(&level, entity, 10.0);
    assert_active_state(&level, entity, "Run");
}

#[test]
fn exit_time_gate_waits_for_normalized_state_progress_without_skipping_crossfade() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("exit-time.skeleton");
    let idle_clip_uri = uri("exit-time-idle.clip");
    let run_clip_uri = uri("exit-time-run.clip");
    let idle_graph_uri = uri("exit-time-idle.graph");
    let run_graph_uri = uri("exit-time-run.graph");
    let machine_uri = uri("exit-time.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    let mut machine = timed_transition_state_machine(&idle_graph_uri, &run_graph_uri);
    machine.transitions[0].duration_seconds = 0.5;
    machine.transitions[0].exit_time = Some(0.75);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        machine,
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        machine_id,
        BTreeMap::from([("advance".to_string(), AnimationParameterValue::Bool(true))]),
    );

    runtime.tick_level_seconds(&level, 0.5).unwrap();
    assert_hand_translation(&level, entity, 0.0);
    assert!(level
        .animation_playback_times(level.capture_world_replacement_epoch())
        .expect("current World exposes playback state")
        .2
        .is_empty());

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    assert_hand_translation(&level, entity, 0.0);
    assert_eq!(
        level
            .animation_playback_times(level.capture_world_replacement_epoch())
            .expect("current World exposes playback state")
            .2[&entity]
            .elapsed_seconds,
        0.0
    );

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    assert_hand_translation(&level, entity, 5.0);

    runtime.tick_level_seconds(&level, 0.25).unwrap();
    assert_hand_translation(&level, entity, 10.0);
    assert_active_state(&level, entity, "Run");
}

#[test]
fn next_state_interruption_preserves_crossfade_pose_continuity() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("interruption.skeleton");
    let idle_clip_uri = uri("interruption-idle.clip");
    let run_clip_uri = uri("interruption-run.clip");
    let sprint_clip_uri = uri("interruption-sprint.clip");
    let idle_graph_uri = uri("interruption-idle.graph");
    let run_graph_uri = uri("interruption-run.graph");
    let sprint_graph_uri = uri("interruption-sprint.graph");
    let machine_uri = uri("interruption.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&sprint_clip_uri),
            ResourceKind::AnimationClip,
            sprint_clip_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 20.0),
    );
    for (graph, clip) in [
        (&idle_graph_uri, &idle_clip_uri),
        (&run_graph_uri, &run_clip_uri),
        (&sprint_graph_uri, &sprint_clip_uri),
    ] {
        register_single_clip_graph(&asset_manager, graph, clip);
    }
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        interruptible_transition_state_machine(&idle_graph_uri, &run_graph_uri, &sprint_graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(machine_id),
                    parameters: BTreeMap::from([
                        ("start".to_string(), AnimationParameterValue::Bool(true)),
                        ("interrupt".to_string(), AnimationParameterValue::Bool(true)),
                    ]),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert!(level.record_animation_playback_times(
        replacement_epoch,
        BTreeMap::new(),
        BTreeMap::from([(entity, 0.5)]),
        BTreeMap::from([(
            entity,
            AnimationStateTransitionRuntime {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds: 1.0,
                elapsed_seconds: 0.5,
                from_time_seconds: 0.5,
                to_time_seconds: 0.5,
            },
        )]),
    ));

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 5.0);
    let interrupted = level
        .animation_playback_times(level.capture_world_replacement_epoch())
        .expect("current World exposes playback state")
        .2
        .get(&entity)
        .cloned()
        .expect("B -> C transition should replace the active A -> B transition");
    assert_eq!(interrupted.from_state, "Run");
    assert_eq!(interrupted.to_state, "Sprint");
    assert_eq!(interrupted.elapsed_seconds, 0.0);

    runtime.tick_level_seconds(&level, 0.5).unwrap();

    assert_hand_translation(&level, entity, 12.5);
}

pub(super) fn uri(name: &str) -> AssetUri {
    AssetUri::parse(&format!("res://animation/{name}.zranim")).unwrap()
}

pub(super) fn assert_hand_translation(
    level: &zircon_runtime::scene::LevelSystem,
    entity: zircon_runtime::scene::EntityId,
    expected: f32,
) {
    let pose = level.animation_pose(entity).expect("state pose");
    let hand = pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("Hand pose");
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(expected, 0.0, 0.0), 1.0e-4));
}

pub(super) fn spawn_state_machine_player(
    level: &zircon_runtime::scene::LevelSystem,
    skeleton_id: ResourceId,
    machine_id: ResourceId,
    parameters: BTreeMap<String, AnimationParameterValue>,
) -> zircon_runtime::scene::EntityId {
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(machine_id),
                    parameters,
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    })
}

fn assert_active_state(
    level: &zircon_runtime::scene::LevelSystem,
    entity: zircon_runtime::scene::EntityId,
    expected: &str,
) {
    assert_eq!(
        level.with_world(|world| world
            .animation_state_machine_player(entity)
            .and_then(|player| player.active_state.clone())),
        Some(expected.to_string())
    );
}
