use super::*;
use zircon_runtime::scene::AnimationStateTransitionRuntime;

#[test]
fn invalid_or_zero_previous_transition_duration_switches_immediately_to_target_pose() {
    for duration_seconds in [0.0, -1.0, f32::NAN] {
        assert_invalid_duration_switches_immediately(duration_seconds);
    }
}

fn assert_invalid_duration_switches_immediately(duration_seconds: f32) {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri =
        AssetUri::parse("res://animation/transition-boundary.skeleton.zranim").unwrap();
    let idle_clip_uri =
        AssetUri::parse("res://animation/transition-boundary-idle.clip.zranim").unwrap();
    let run_clip_uri =
        AssetUri::parse("res://animation/transition-boundary-run.clip.zranim").unwrap();
    let idle_graph_uri =
        AssetUri::parse("res://animation/transition-boundary-idle.graph.zranim").unwrap();
    let run_graph_uri =
        AssetUri::parse("res://animation/transition-boundary-run.graph.zranim").unwrap();
    let machine_uri =
        AssetUri::parse("res://animation/transition-boundary.machine.zranim").unwrap();
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
                    parameters: BTreeMap::new(),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    level.record_animation_playback_times(
        BTreeMap::new(),
        BTreeMap::from([(entity, 0.0)]),
        BTreeMap::from([(
            entity,
            AnimationStateTransitionRuntime {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds,
                elapsed_seconds: 0.0,
                from_time_seconds: 0.0,
                to_time_seconds: 0.0,
            },
        )]),
    );

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    let pose = level.animation_pose(entity).expect("target pose");
    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(hand.local_transform.translation.is_finite());
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(10.0, 0.0, 0.0), 1.0e-4));
    assert_eq!(
        level.with_world(|world| world
            .animation_state_machine_player(entity)
            .unwrap()
            .active_state
            .clone()),
        Some("Run".to_string())
    );
    assert!(level.animation_playback_times().2.is_empty());
}
