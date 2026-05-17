use std::collections::BTreeMap;

use zircon_runtime::asset::{
    self, AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationEventTrackAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationGraphParameterAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    AssetReference, AssetUri, ProjectAssetManager,
};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationParameterValue,
};
use zircon_runtime::core::framework::physics::{PhysicsSettings, PhysicsSimulationMode};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::core::manager::{resolve_animation_manager, resolve_physics_manager};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::core::{CoreHandle, CoreRuntime};
use zircon_runtime::plugin::{
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::{foundation, scene};

#[test]
fn plugin_runtime_resolves_physics_and_animation_managers() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();

    let physics = resolve_physics_manager(&core).expect("physics manager should resolve");
    let animation = resolve_animation_manager(&core).expect("animation manager should resolve");

    assert_eq!(physics.backend_status().requested_backend, "unconfigured");
    assert!(animation.playback_settings().enabled);
}

#[test]
fn level_tick_advances_physics_and_records_contacts() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    runtime_physics_manager(&core)
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 4,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = scene::create_default_level(&core).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::X,
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        world
            .set_collider(
                body,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let blocker = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                blocker,
                Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                blocker,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        body
    });

    level.tick(&core, 1.0 / 60.0).unwrap();

    let transform = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(level.last_physics_step_plan().unwrap().steps, 1);
    assert!(transform.translation.x > 0.0);
    assert_eq!(level.physics_contacts().len(), 1);
}

#[test]
fn level_tick_without_physics_plugin_does_not_run_physics() {
    let runtime = runtime_with_scene_asset_only();
    let core = runtime.handle();
    let level = scene::create_default_level(&core).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::X,
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        world
            .set_collider(
                body,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        body
    });
    let before = level.with_world(|world| world.find_node(body).unwrap().transform);

    level.tick(&core, 1.0 / 60.0).unwrap();

    let after = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(after, before);
    assert!(level.last_physics_step_plan().is_none());
    assert!(level.physics_contacts().is_empty());
}

#[test]
fn level_tick_applies_loaded_animation_sequences_to_world_properties() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let target_entity_name = "Runtime Sequence Target";
    let sequence_uri =
        zircon_runtime::asset::AssetUri::parse("res://animation/test.sequence.zranim")
            .expect("test sequence locator");
    let sequence_id = ResourceId::from_locator(&sequence_uri);
    let asset_manager = runtime_asset_manager(&core);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(sequence_id, ResourceKind::AnimationSequence, sequence_uri),
        sequence_asset_for_entity(target_entity_name),
    );
    let level = scene::create_default_level(&core).unwrap();
    let cube = level.with_world_mut(|world| {
        let cube = world.spawn_node(NodeKind::Cube);
        world.rename_node(cube, target_entity_name).unwrap();
        world
            .set_animation_sequence_player(
                cube,
                Some(AnimationSequencePlayerComponent {
                    sequence: ResourceHandle::<AnimationSequenceMarker>::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        cube
    });

    level.tick(&core, 0.5).unwrap();

    let (translation, player_time) = level.with_world(|world| {
        (
            world.find_node(cube).unwrap().transform.translation,
            world.animation_sequence_player(cube).unwrap().time_seconds,
        )
    });
    assert_eq!(translation, Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(player_time, 0.5);
}

#[test]
fn level_tick_emits_animation_clip_event_tracks_crossed_by_player_time() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/event.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![
        AnimationEventTrackAsset {
            target_id: Some("Root/Hand".to_string()),
            event: "footstep".to_string(),
            time_seconds: 0.25,
            payload: Some("left".to_string()),
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "land".to_string(),
            time_seconds: 0.75,
            payload: None,
        },
    ];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );
    let level = scene::create_default_level(&core).unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.2,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.1).unwrap();
    let events = drain_animation_clip_events(&level);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].event, "footstep");
    assert_eq!(events[0].payload.as_deref(), Some("left"));
    assert_eq!(events[0].clip_time_seconds, 0.25);
    assert_eq!(events[0].playback_time_seconds, 0.25);
}

#[test]
fn clip_event_sampling_reports_loop_boundary_occurrences_in_playback_order() {
    let skeleton_uri = AssetUri::parse("res://animation/event-loop.skeleton.zranim").unwrap();
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![
        AnimationEventTrackAsset {
            target_id: None,
            event: "loop_start".to_string(),
            time_seconds: 0.0,
            payload: None,
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "mid".to_string(),
            time_seconds: 0.5,
            payload: None,
        },
    ];

    let events = zircon_plugin_animation_runtime::sample_clip_events(&clip, 7, 0.75, 1.6, true);

    assert_eq!(
        events
            .iter()
            .map(|event| (event.event.as_str(), event.playback_time_seconds))
            .collect::<Vec<_>>(),
        vec![("loop_start", 1.0), ("mid", 1.5)]
    );
}

#[test]
fn graph_player_emits_clip_events_using_graph_clip_playback_speed() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/graph-event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/graph-event.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/graph-event.graph.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: Some("Root/Hand".to_string()),
        event: "graph_hit".to_string(),
        time_seconds: 0.5,
        payload: Some("fast".to_string()),
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        clip,
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
        AnimationGraphAsset {
            name: Some("GraphEvent".to_string()),
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Clip {
                    id: "clip".to_string(),
                    clip: AssetReference::from_locator(clip_uri),
                    playback_speed: 2.0,
                    looping: false,
                },
                AnimationGraphNodeAsset::Output {
                    source: "clip".to_string(),
                },
            ],
        },
    );
    let level = scene::create_default_level(&core).unwrap();
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.3).unwrap();
    let events = drain_animation_clip_events(&level);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].event, "graph_hit");
    assert_eq!(events[0].payload.as_deref(), Some("fast"));
    assert_eq!(events[0].clip_time_seconds, 0.5);
    assert_eq!(events[0].playback_time_seconds, 0.5);
}

#[test]
fn state_machine_player_emits_active_graph_clip_events() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/state-event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/state-event.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/state-event.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/state-event.machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: Some("Root/Hand".to_string()),
        event: "state_hit".to_string(),
        time_seconds: 0.4,
        payload: Some("idle".to_string()),
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        clip,
    );
    register_single_clip_graph(&asset_manager, &graph_uri, &clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            machine_id,
            ResourceKind::AnimationStateMachine,
            machine_uri.clone(),
        ),
        single_state_machine(&graph_uri),
    );

    let level = scene::create_default_level(&core).unwrap();
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

    level.tick(&core, 0.5).unwrap();
    let events = drain_animation_clip_events(&level);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].target_id.as_deref(), Some("Root/Hand"));
    assert_eq!(events[0].event, "state_hit");
    assert_eq!(events[0].payload.as_deref(), Some("idle"));
    assert_eq!(events[0].clip_time_seconds, 0.4);
    assert_eq!(events[0].playback_time_seconds, 0.4);
}

#[test]
fn state_machine_transition_emits_from_and_to_graph_clip_events() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/transition-event.skeleton.zranim").unwrap();
    let idle_clip_uri =
        AssetUri::parse("res://animation/transition-event-idle.clip.zranim").unwrap();
    let run_clip_uri = AssetUri::parse("res://animation/transition-event-run.clip.zranim").unwrap();
    let idle_graph_uri =
        AssetUri::parse("res://animation/transition-event-idle.graph.zranim").unwrap();
    let run_graph_uri =
        AssetUri::parse("res://animation/transition-event-run.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/transition-event.machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let idle_clip_id = ResourceId::from_locator(&idle_clip_uri);
    let run_clip_id = ResourceId::from_locator(&run_clip_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    let mut idle_clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    idle_clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: None,
        event: "idle_exit".to_string(),
        time_seconds: 0.05,
        payload: None,
    }];
    let mut run_clip = single_hand_translation_clip(&skeleton_uri, 10.0);
    run_clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: None,
        event: "run_enter".to_string(),
        time_seconds: 0.05,
        payload: None,
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            idle_clip_id,
            ResourceKind::AnimationClip,
            idle_clip_uri.clone(),
        ),
        idle_clip,
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            run_clip_id,
            ResourceKind::AnimationClip,
            run_clip_uri.clone(),
        ),
        run_clip,
    );
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            machine_id,
            ResourceKind::AnimationStateMachine,
            machine_uri.clone(),
        ),
        timed_transition_state_machine(&idle_graph_uri, &run_graph_uri),
    );

    let level = scene::create_default_level(&core).unwrap();
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
                    parameters: BTreeMap::from([(
                        "advance".to_string(),
                        AnimationParameterValue::Bool(true),
                    )]),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.1).unwrap();
    let mut events = drain_animation_clip_events(&level);
    events.sort_by(|a, b| a.event.cmp(&b.event));

    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.entity == entity));
    assert_eq!(
        events
            .iter()
            .map(|event| (event.event.as_str(), event.clip_time_seconds))
            .collect::<Vec<_>>(),
        vec![("idle_exit", 0.05), ("run_enter", 0.05)]
    );
}

#[test]
fn level_tick_without_animation_plugin_does_not_advance_sequence_players() {
    let runtime = runtime_with_scene_asset_only();
    let core = runtime.handle();
    let target_entity_name = "Runtime Sequence Target";
    let sequence_uri =
        zircon_runtime::asset::AssetUri::parse("res://animation/test.sequence.zranim")
            .expect("test sequence locator");
    let sequence_id = ResourceId::from_locator(&sequence_uri);
    let asset_manager = runtime_asset_manager(&core);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(sequence_id, ResourceKind::AnimationSequence, sequence_uri),
        sequence_asset_for_entity(target_entity_name),
    );
    let level = scene::create_default_level(&core).unwrap();
    let cube = level.with_world_mut(|world| {
        let cube = world.spawn_node(NodeKind::Cube);
        world.rename_node(cube, target_entity_name).unwrap();
        world
            .set_animation_sequence_player(
                cube,
                Some(AnimationSequencePlayerComponent {
                    sequence: ResourceHandle::<AnimationSequenceMarker>::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        cube
    });

    level.tick(&core, 0.5).unwrap();

    let (translation, player_time) = level.with_world(|world| {
        (
            world.find_node(cube).unwrap().transform.translation,
            world.animation_sequence_player(cube).unwrap().time_seconds,
        )
    });
    assert_eq!(translation, Vec3::ZERO);
    assert_eq!(player_time, 0.0);
    assert!(level.animation_pose(cube).is_none());
}

fn drain_animation_clip_events(
    level: &zircon_runtime::scene::LevelSystem,
) -> Vec<zircon_plugin_animation_runtime::AnimationClipEvent> {
    level.with_world_mut(|world| {
        world.update_events::<zircon_plugin_animation_runtime::AnimationClipEvent>();
        world
            .events::<zircon_plugin_animation_runtime::AnimationClipEvent>()
            .map(|events| events.iter().cloned().collect())
            .unwrap_or_default()
    })
}

#[test]
fn level_tick_blends_animation_graph_clip_pose_weights() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/blend.skeleton.zranim").unwrap();
    let clip_a_uri = AssetUri::parse("res://animation/blend-a.clip.zranim").unwrap();
    let clip_b_uri = AssetUri::parse("res://animation/blend-b.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/blend.graph.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &clip_a_uri, &clip_b_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
        two_clip_blend_graph(&clip_a_uri, &clip_b_uri, 0.25),
    );

    let level = scene::create_default_level(&core).unwrap();
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.0).unwrap();

    let pose = level
        .animation_pose(entity)
        .expect("graph player should cache a blended pose");
    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(2.5, 0.0, 0.0), 1.0e-4));
}

#[test]
fn animation_graph_evaluation_reports_additive_mask_and_clip_targets() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let animation = resolve_animation_manager(&runtime.handle()).unwrap();
    let base_uri = AssetUri::parse("res://animation/additive-base.clip.zranim").unwrap();
    let add_uri = AssetUri::parse("res://animation/additive-layer.clip.zranim").unwrap();
    let graph = AnimationGraphAsset {
        name: Some("AdditiveMask".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "upper".to_string(),
            default_value: AnimationParameterValue::Scalar(0.5),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "base".to_string(),
                clip: AssetReference::from_locator(base_uri.clone()),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Clip {
                id: "add".to_string(),
                clip: AssetReference::from_locator(add_uri.clone()),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Additive {
                id: "additive".to_string(),
                base: "base".to_string(),
                additive: "add".to_string(),
                weight_parameter: Some("upper".to_string()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "masked".to_string(),
                input: "additive".to_string(),
                target_ids: vec!["Root/Hand".to_string()],
            },
            AnimationGraphNodeAsset::Output {
                source: "masked".to_string(),
            },
        ],
    };

    let evaluation = animation.evaluate_graph(&graph, &BTreeMap::new());

    assert_eq!(evaluation.output_node.as_deref(), Some("masked"));
    assert_eq!(evaluation.mask_target_ids, vec!["Root/Hand".to_string()]);
    assert_eq!(evaluation.clips.len(), 2);
    let base = evaluation
        .clips
        .iter()
        .find(|clip| clip.clip.locator == base_uri)
        .unwrap();
    assert_eq!(base.blend_mode, AnimationGraphBlendMode::Base);
    assert_eq!(base.target_ids, vec!["Root/Hand".to_string()]);
    let additive = evaluation
        .clips
        .iter()
        .find(|clip| clip.clip.locator == add_uri)
        .unwrap();
    assert_eq!(additive.blend_mode, AnimationGraphBlendMode::Additive);
    assert_eq!(additive.weight, 0.5);
    assert_eq!(additive.target_ids, vec!["Root/Hand".to_string()]);
}

#[test]
fn level_tick_applies_additive_graph_layer_only_to_mask_targets() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/additive-mask.skeleton.zranim").unwrap();
    let base_uri = AssetUri::parse("res://animation/additive-mask-base.clip.zranim").unwrap();
    let add_uri = AssetUri::parse("res://animation/additive-mask-add.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/additive-mask.graph.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &base_uri, &add_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&graph_uri),
            ResourceKind::AnimationGraph,
            graph_uri,
        ),
        additive_mask_graph(&base_uri, &add_uri),
    );

    let level = scene::create_default_level(&core).unwrap();
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.0).unwrap();

    let pose = level
        .animation_pose(entity)
        .expect("additive masked graph should cache a pose");
    let root = pose.bones.iter().find(|bone| bone.name == "Root").unwrap();
    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(root
        .local_transform
        .translation
        .abs_diff_eq(Vec3::ZERO, 1.0e-4));
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(10.0, 0.0, 0.0), 1.0e-4));
}

#[test]
fn clip_sampling_resolves_track_target_id_before_bone_name_fallback() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let animation = resolve_animation_manager(&runtime.handle()).unwrap();
    let skeleton = two_bone_skeleton();
    let mut clip = single_hand_translation_clip(
        &AssetUri::parse("res://animation/target-id.clip.zranim").unwrap(),
        5.0,
    );
    clip.tracks[0].bone_name = "MissingByName".to_string();
    clip.tracks[0].target_id = Some("Root/Hand".to_string());

    let pose = animation
        .sample_clip_pose(&skeleton, &clip, 0.0, false)
        .unwrap();

    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(5.0, 0.0, 0.0), 1.0e-4));
}

#[test]
fn sequence_runtime_resolves_target_id_before_entity_path_fallback() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let target_entity_name = "Runtime Sequence Target";
    let sequence_uri = AssetUri::parse("res://animation/target-id.sequence.zranim").unwrap();
    let sequence_id = ResourceId::from_locator(&sequence_uri);
    let asset_manager = runtime_asset_manager(&core);
    let mut sequence = sequence_asset_for_entity("Wrong/Path");
    sequence.bindings[0].target_id = Some(target_entity_name.to_string());
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(sequence_id, ResourceKind::AnimationSequence, sequence_uri),
        sequence,
    );
    let level = scene::create_default_level(&core).unwrap();
    let cube = level.with_world_mut(|world| {
        let cube = world.spawn_node(NodeKind::Cube);
        world.rename_node(cube, target_entity_name).unwrap();
        world
            .set_animation_sequence_player(
                cube,
                Some(AnimationSequencePlayerComponent {
                    sequence: ResourceHandle::<AnimationSequenceMarker>::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        cube
    });

    level.tick(&core, 0.5).unwrap();

    let translation =
        level.with_world(|world| world.find_node(cube).unwrap().transform.translation);
    assert_eq!(translation, Vec3::new(2.0, 0.0, 0.0));
}

#[test]
fn level_tick_blends_state_machine_transition_until_duration_completes() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/transition.skeleton.zranim").unwrap();
    let idle_clip_uri = AssetUri::parse("res://animation/transition-idle.clip.zranim").unwrap();
    let run_clip_uri = AssetUri::parse("res://animation/transition-run.clip.zranim").unwrap();
    let idle_graph_uri = AssetUri::parse("res://animation/transition-idle.graph.zranim").unwrap();
    let run_graph_uri = AssetUri::parse("res://animation/transition-run.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/transition.state_machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        timed_transition_state_machine(&idle_graph_uri, &run_graph_uri),
    );

    let level = scene::create_default_level(&core).unwrap();
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
                    parameters: BTreeMap::from([(
                        "advance".to_string(),
                        AnimationParameterValue::Bool(true),
                    )]),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    level.tick(&core, 0.1).unwrap();

    let midway_pose = level
        .animation_pose(entity)
        .expect("transition should produce a blended pose");
    let midway_hand = midway_pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .unwrap();
    assert!(midway_hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(5.0, 0.0, 0.0), 1.0e-4));
    assert_eq!(
        level.with_world(|world| world
            .animation_state_machine_player(entity)
            .unwrap()
            .active_state
            .clone()),
        Some("Idle".to_string())
    );

    level.tick(&core, 0.1).unwrap();

    let final_pose = level
        .animation_pose(entity)
        .expect("completed transition should keep producing target pose");
    let final_hand = final_pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .unwrap();
    assert!(final_hand
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
}

fn runtime_with_physics_animation_scene_asset() -> CoreRuntime {
    let physics_registration = RuntimePluginRegistrationReport::from_plugin(
        &zircon_plugin_physics_runtime::runtime_plugin(),
    );
    let animation_registration = RuntimePluginRegistrationReport::from_plugin(
        &zircon_plugin_animation_runtime::runtime_plugin(),
    );
    assert!(
        physics_registration.is_success(),
        "{:?}",
        physics_registration.diagnostics
    );
    assert!(
        animation_registration.is_success(),
        "{:?}",
        animation_registration.diagnostics
    );
    let extension_report = RuntimePluginCatalog::from_registration_reports(
        [physics_registration, animation_registration],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    )
    .runtime_extensions();
    assert!(
        extension_report.is_success(),
        "{:?}",
        extension_report.fatal_diagnostics
    );

    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation::module_descriptor())
        .unwrap();
    runtime.register_module(asset::module_descriptor()).unwrap();
    runtime.register_module(scene::module_descriptor()).unwrap();
    for module in extension_report.registry.modules() {
        runtime.register_module(module.clone()).unwrap();
    }
    runtime
        .install_scene_runtime_hooks(&extension_report.registry)
        .unwrap();
    runtime
        .activate_module(foundation::FOUNDATION_MODULE_NAME)
        .unwrap();
    runtime.activate_module(asset::ASSET_MODULE_NAME).unwrap();
    runtime.activate_module(scene::SCENE_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_plugin_physics_runtime::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_plugin_animation_runtime::ANIMATION_MODULE_NAME)
        .unwrap();
    runtime
}

fn runtime_physics_manager(
    core: &CoreHandle,
) -> std::sync::Arc<zircon_plugin_physics_runtime::DefaultPhysicsManager> {
    core.resolve_manager::<zircon_plugin_physics_runtime::DefaultPhysicsManager>(
        zircon_plugin_physics_runtime::DEFAULT_PHYSICS_MANAGER_NAME,
    )
    .unwrap()
}

fn runtime_with_scene_asset_only() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation::module_descriptor())
        .unwrap();
    runtime.register_module(asset::module_descriptor()).unwrap();
    runtime.register_module(scene::module_descriptor()).unwrap();
    runtime
        .activate_module(foundation::FOUNDATION_MODULE_NAME)
        .unwrap();
    runtime.activate_module(asset::ASSET_MODULE_NAME).unwrap();
    runtime.activate_module(scene::SCENE_MODULE_NAME).unwrap();
    runtime
}

fn runtime_asset_manager(core: &CoreHandle) -> std::sync::Arc<ProjectAssetManager> {
    core.resolve_manager::<ProjectAssetManager>(asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap()
}

fn sequence_asset_for_entity(entity_path: &str) -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("RuntimeSequenceTick".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse(entity_path).unwrap(),
            target_id: Some(entity_path.to_string()),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
                            in_tangent: None,
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 0.5,
                            value: AnimationChannelValueAsset::Vec3([2.0, 0.0, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: None,
                        },
                    ],
                },
            }],
        }],
    }
}

fn register_animation_blend_assets(
    asset_manager: &ProjectAssetManager,
    skeleton_uri: &AssetUri,
    clip_a_uri: &AssetUri,
    clip_b_uri: &AssetUri,
) {
    let skeleton_id = ResourceId::from_locator(skeleton_uri);
    let clip_a_id = ResourceId::from_locator(clip_a_uri);
    let clip_b_id = ResourceId::from_locator(clip_b_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_a_id, ResourceKind::AnimationClip, clip_a_uri.clone()),
        single_hand_translation_clip(skeleton_uri, 0.0),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_b_id, ResourceKind::AnimationClip, clip_b_uri.clone()),
        single_hand_translation_clip(skeleton_uri, 10.0),
    );
}

fn register_single_clip_graph(
    asset_manager: &ProjectAssetManager,
    graph_uri: &AssetUri,
    clip_uri: &AssetUri,
) {
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(graph_uri),
            ResourceKind::AnimationGraph,
            graph_uri.clone(),
        ),
        single_clip_graph(clip_uri),
    );
}

fn two_bone_skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("BlendSkeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".to_string(),
                parent_index: None,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".to_string(),
                parent_index: Some(0),
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    }
}

fn single_hand_translation_clip(skeleton_uri: &AssetUri, translation_x: f32) -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some(format!("Hand{translation_x}")),
        skeleton: AssetReference::from_locator(skeleton_uri.clone()),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            target_id: Some("Root/Hand".to_string()),
            translation: constant_vec3_channel([translation_x, 0.0, 0.0]),
            rotation: constant_quaternion_channel([0.0, 0.0, 0.0, 1.0]),
            scale: constant_vec3_channel([1.0, 1.0, 1.0]),
        }],
        event_tracks: Vec::new(),
    }
}

fn two_clip_blend_graph(
    clip_a_uri: &AssetUri,
    clip_b_uri: &AssetUri,
    blend_weight: f32,
) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("TwoClipBlend".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "blend".to_string(),
            default_value: AnimationParameterValue::Scalar(blend_weight),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "a".to_string(),
                clip: AssetReference::from_locator(clip_a_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Clip {
                id: "b".to_string(),
                clip: AssetReference::from_locator(clip_b_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend".to_string(),
                inputs: vec!["a".to_string(), "b".to_string()],
                weight_parameter: Some("blend".to_string()),
            },
            AnimationGraphNodeAsset::Output {
                source: "blend".to_string(),
            },
        ],
    }
}

fn additive_mask_graph(base_uri: &AssetUri, additive_uri: &AssetUri) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("AdditiveMaskGraph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "additive_weight".to_string(),
            default_value: AnimationParameterValue::Scalar(1.0),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "base".to_string(),
                clip: AssetReference::from_locator(base_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Clip {
                id: "add".to_string(),
                clip: AssetReference::from_locator(additive_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Additive {
                id: "additive".to_string(),
                base: "base".to_string(),
                additive: "add".to_string(),
                weight_parameter: Some("additive_weight".to_string()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "masked".to_string(),
                input: "additive".to_string(),
                target_ids: vec!["Root/Hand".to_string()],
            },
            AnimationGraphNodeAsset::Output {
                source: "masked".to_string(),
            },
        ],
    }
}

fn single_clip_graph(clip_uri: &AssetUri) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("SingleClipGraph".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "clip".to_string(),
                clip: AssetReference::from_locator(clip_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Output {
                source: "clip".to_string(),
            },
        ],
    }
}

fn timed_transition_state_machine(
    idle_graph_uri: &AssetUri,
    run_graph_uri: &AssetUri,
) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("TimedTransition".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: AssetReference::from_locator(idle_graph_uri.clone()),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: AssetReference::from_locator(run_graph_uri.clone()),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.2,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "advance".to_string(),
                operator: asset::AnimationConditionOperatorAsset::Equal,
                value: Some(AnimationParameterValue::Bool(true)),
            }],
        }],
    }
}

fn single_state_machine(graph_uri: &AssetUri) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("SingleState".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![AnimationStateAsset {
            name: "Idle".to_string(),
            graph: AssetReference::from_locator(graph_uri.clone()),
        }],
        transitions: Vec::new(),
    }
}

fn constant_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Vec3(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn constant_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Quaternion(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}
