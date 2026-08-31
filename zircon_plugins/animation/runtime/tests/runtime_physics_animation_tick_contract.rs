use std::collections::BTreeMap;

#[path = "runtime_physics_animation_tick_contract/additive_reference_pose.rs"]
mod additive_reference_pose;
#[path = "runtime_physics_animation_tick_contract/animation_assets.rs"]
mod animation_assets;
#[path = "runtime_physics_animation_tick_contract/blend_space_state.rs"]
mod blend_space_state;
#[path = "runtime_physics_animation_tick_contract/cache_invalidation.rs"]
mod cache_invalidation;
#[path = "runtime_physics_animation_tick_contract/evaluation_diagnostics.rs"]
mod evaluation_diagnostics;
#[path = "runtime_physics_animation_tick_contract/runtime_helpers.rs"]
mod runtime_helpers;
#[path = "runtime_physics_animation_tick_contract/state_machine_boundaries.rs"]
mod state_machine_boundaries;
#[path = "runtime_physics_animation_tick_contract/state_machine_interruption.rs"]
mod state_machine_interruption;
#[path = "runtime_physics_animation_tick_contract/target_resolution.rs"]
mod target_resolution;

use animation_assets::{
    additive_mask_graph, interruptible_transition_state_machine, register_animation_blend_assets,
    register_single_clip_graph, sequence_asset_for_entity, single_hand_translation_clip,
    single_state_machine, timed_transition_state_machine, two_bone_skeleton, two_clip_blend_graph,
};
use runtime_helpers::{
    runtime_asset_manager, runtime_physics_query_bridge,
    runtime_with_physics_animation_scene_asset, runtime_with_scene_asset_only,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationEventTrackAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationGraphParameterAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationParameterValue,
};
use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsQueryFilter, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsShapeCastQuery, PhysicsShapeOverlapQuery, PhysicsSimulationMode,
};
use zircon_runtime::core::manager::{
    animation_manager_handle, physics_manager_handle, resolve_manager_service,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};

#[test]
fn plugin_runtime_resolves_physics_and_animation_managers() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();

    let physics = resolve_manager_service(
        &core,
        physics_manager_handle(&core).expect("physics manager handle"),
    )
    .expect("physics manager should resolve");
    let animation = resolve_manager_service(
        &core,
        animation_manager_handle(&core).expect("animation manager handle"),
    )
    .expect("animation manager should resolve");

    assert_eq!(physics.backend_status().requested_backend, "unconfigured");
    assert!(animation.playback_settings().enabled);
}

#[test]
fn level_tick_advances_physics_and_records_contacts() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let physics = resolve_manager_service(
        &core,
        physics_manager_handle(&core).expect("physics manager handle"),
    )
    .expect("physics manager should resolve");
    physics
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 4,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let physics_query = runtime_physics_query_bridge(runtime.extension_report());
    let level = runtime.create_default_level().unwrap();
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

    runtime.tick_level_seconds(&level, 1.0 / 60.0).unwrap();

    let transform = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(level.last_physics_step_plan().unwrap().steps, 1);
    assert!(transform.translation.x > 0.0);
    assert_eq!(level.physics_contacts().len(), 1);

    let world = level.world_handle();
    let ray_hit = physics_query
        .call(|physics| {
            physics.ray_cast(&PhysicsRayCastQuery {
                world,
                origin: [-4.0, 0.0, 0.0],
                direction: [1.0, 0.0, 0.0],
                max_distance: 16.0,
                mode: Default::default(),
                filter: PhysicsQueryFilter::default(),
            })
        })
        .expect("physics.query ray cast should be enabled");
    assert!(!ray_hit.is_empty());

    let overlap_hits = physics_query
        .call(|physics| {
            physics.shape_overlap(&PhysicsShapeOverlapQuery {
                world,
                shape: PhysicsColliderShape::Box {
                    half_extents: [2.0, 2.0, 2.0],
                },
                transform: Transform::default(),
                mode: Default::default(),
                filter: PhysicsQueryFilter::default(),
            })
        })
        .expect("physics.query overlap should be enabled");
    assert!(!overlap_hits.is_empty());

    let shape_cast_hit = physics_query
        .call(|physics| {
            physics.shape_cast(&PhysicsShapeCastQuery {
                world,
                shape: PhysicsColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                origin_transform: Transform::default(),
                direction: [1.0, 0.0, 0.0],
                max_distance: 4.0,
                mode: Default::default(),
                filter: PhysicsQueryFilter::default(),
            })
        })
        .expect("physics.query shape cast should be enabled");
    assert!(!shape_cast_hit.is_empty());
}

#[test]
fn level_tick_without_physics_plugin_does_not_run_physics() {
    let runtime = runtime_with_scene_asset_only();
    let level = runtime.create_default_level().unwrap();
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

    runtime.tick_level_seconds(&level, 1.0 / 60.0).unwrap();

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
    let level = runtime.create_default_level().unwrap();
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

    runtime.tick_level_seconds(&level, 0.5).unwrap();

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
    let level = runtime.create_default_level().unwrap();
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

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].event, "footstep");
    assert_eq!(events[0].payload.as_deref(), Some("left"));
    assert_eq!(events[0].clip_time_seconds, 0.25);
    assert_eq!(events[0].playback_time_seconds, 0.25);
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new().into(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.3).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

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
                    parameters: BTreeMap::new().into(),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.5).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

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
                    parameters: BTreeMap::from([(
                        "advance".to_string(),
                        AnimationParameterValue::Bool(true),
                    )])
                    .into(),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    let mut events = drain_animation_clip_events(&level, &mut event_subscription);
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
    let level = runtime.create_default_level().unwrap();
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

    runtime.tick_level_seconds(&level, 0.5).unwrap();

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

fn subscribe_animation_clip_events(
    level: &zircon_runtime::scene::LevelSystem,
) -> zircon_runtime::scene::EventSubscription<zircon_plugin_animation_runtime::AnimationClipEvent> {
    level.with_world_mut(|world| {
        let mut subscription = world
            .register_dormant_event_subscription::<
                zircon_plugin_animation_runtime::AnimationClipEvent,
            >();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    })
}

fn drain_animation_clip_events(
    level: &zircon_runtime::scene::LevelSystem,
    subscription: &mut zircon_runtime::scene::EventSubscription<
        zircon_plugin_animation_runtime::AnimationClipEvent,
    >,
) -> Vec<zircon_plugin_animation_runtime::AnimationClipEvent> {
    level.with_world_mut(|world| {
        world.update_events::<zircon_plugin_animation_runtime::AnimationClipEvent>();
        world
            .read_event_subscription(subscription)
            .cloned()
            .collect()
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new().into(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();

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
    let core = runtime.handle();
    let animation = resolve_manager_service(
        &core,
        animation_manager_handle(&core).expect("animation manager handle"),
    )
    .unwrap();
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new().into(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();

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
