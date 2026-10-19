//! 真实 LevelSystem 帧入口同时推进物理、动画、事件和场景姿态的集成契约。
use std::collections::BTreeMap;

#[path = "additive_reference_pose.rs"]
mod additive_reference_pose;
#[path = "animation_assets.rs"]
mod animation_assets;
#[path = "blend_space_state.rs"]
mod blend_space_state;
#[path = "cache_invalidation.rs"]
mod cache_invalidation;
#[path = "evaluation_diagnostics.rs"]
mod evaluation_diagnostics;
#[path = "runtime_helpers.rs"]
mod runtime_helpers;
#[path = "state_machine_boundaries.rs"]
mod state_machine_boundaries;
#[path = "state_machine_interruption.rs"]
mod state_machine_interruption;
#[path = "target_resolution.rs"]
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
use zircon_runtime::core::math::{Real, Transform, Vec3};
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
        let body = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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

        let blocker = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
        let body = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
    let sequence_asset = sequence_asset_for_entity(target_entity_name);
    let sequence_duration = sequence_asset.duration_seconds;
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(sequence_id, ResourceKind::AnimationSequence, sequence_uri),
        sequence_asset,
    );
    let level = runtime.create_default_level().unwrap();
    let cube = level.with_world_mut(|world| {
        let cube = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
    assert!(level.animation_requires_continuous_frame());

    let set_playback = |playing, playback_speed, time_seconds, looping| {
        level.with_world_mut(|world| {
            let mut player = world.animation_sequence_player(cube).unwrap().clone();
            player.playing = playing;
            player.playback_speed = playback_speed;
            player.time_seconds = time_seconds;
            player.looping = looping;
            world
                .set_animation_sequence_player(cube, Some(player))
                .unwrap();
        });
    };
    set_playback(false, 1.0, 0.5, false);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
    set_playback(true, 1.0, 0.5, true);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(level.animation_requires_continuous_frame());
    set_playback(true, 0.0, 0.5, true);
    runtime.tick_level_seconds(&level, 0.5).unwrap();
    assert!(!level.animation_requires_continuous_frame());
    set_playback(true, 1.0, sequence_duration, false);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
    set_playback(true, -1.0, 0.5, false);
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert!(level.animation_requires_continuous_frame());
    set_playback(true, -1.0, 0.0, true);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
}

include!("physics_tick/clip_events.rs");

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
        let cube = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
    assert!(!level.animation_requires_continuous_frame());
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
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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

#[test]
fn ready_clip_frame_demand_tracks_real_worker_results_and_idle_boundaries() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let assets = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/frame-demand.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/frame-demand.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let clip = single_hand_translation_clip(&skeleton_uri, 3.0);
    let duration = clip.duration_seconds;
    assert!(duration.is_finite() && duration > 0.0);
    assert!(clip.event_tracks.is_empty());
    assets.resource_manager().register_ready(
        ResourceRecord::new(skeleton_id, ResourceKind::AnimationSkeleton, skeleton_uri),
        two_bone_skeleton(),
    );
    assets.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: duration * 0.25,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    let set_playback = |playing, speed, time, looping, weight| {
        level.with_world_mut(|world| {
            let mut player = world.animation_player(entity).unwrap().clone();
            player.playing = playing;
            player.playback_speed = speed;
            player.time_seconds = time;
            player.looping = looping;
            player.weight = weight;
            world.set_animation_player(entity, Some(player)).unwrap();
        });
    };

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(level.animation_pose(entity).is_some());
    assert!(level.animation_requires_continuous_frame());
    // An inactive producer cannot erase the first worker's true result.
    level.with_world_mut(|world| {
        let idle = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_skeleton(
                idle,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                idle,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 0.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(level.animation_requires_continuous_frame());

    for (playing, speed, time, looping, weight, expected) in [
        (false, 1.0, duration * 0.25, true, 1.0, false),
        (true, 0.0, duration * 0.25, true, 1.0, false),
        (true, Real::NAN, duration * 0.25, true, 1.0, false),
        (true, 1.0, duration, false, 1.0, false),
        (true, 1.0, duration * 2.0, true, 1.0, true),
        (true, -1.0, duration * 0.5, false, 1.0, true),
        (true, -1.0, 0.0, true, 1.0, false),
        (true, 1.0, duration * 0.25, true, 0.0, false),
    ] {
        set_playback(playing, speed, time, looping, weight);
        runtime.tick_level_seconds(&level, 0.0).unwrap();
        assert_eq!(level.animation_requires_continuous_frame(), expected);
    }
    level.with_world_mut(|world| {
        let mut player = world.animation_player(entity).unwrap().clone();
        player.clip = ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
            "frame-demand.unavailable-clip",
        ));
        player.weight = 1.0;
        world.set_animation_player(entity, Some(player)).unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
}

#[test]
fn ready_graph_and_machine_frame_demand_uses_weighted_contributions() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let assets = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/demand-graph.skeleton.zranim").unwrap();
    let once_uri = AssetUri::parse("res://animation/demand-once.clip.zranim").unwrap();
    let loop_uri = AssetUri::parse("res://animation/demand-loop.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/demand.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/demand.machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);
    let once_clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    let clip_duration = once_clip.duration_seconds;
    assert!(clip_duration.is_finite() && clip_duration > 0.0);
    assets.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    assets.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&once_uri),
            ResourceKind::AnimationClip,
            once_uri.clone(),
        ),
        once_clip,
    );
    assets.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&loop_uri),
            ResourceKind::AnimationClip,
            loop_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 10.0),
    );
    let register_graph = |weight| {
        let mut graph = two_clip_blend_graph(&once_uri, &loop_uri, weight);
        for node in &mut graph.nodes {
            if let AnimationGraphNodeAsset::Clip { clip, looping, .. } = node {
                *looping = clip.locator == loop_uri;
            }
        }
        assets.resource_manager().register_ready(
            ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri.clone()),
            graph,
        );
    };
    register_graph(0.0);
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
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
    runtime
        .tick_level_seconds(&level, f64::from(clip_duration) * 2.0)
        .unwrap();
    assert!(level.animation_pose(entity).is_some());
    assert!(!level.animation_requires_continuous_frame());
    register_graph(1.0);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(level.animation_requires_continuous_frame());
    level.with_world_mut(|world| {
        let mut player = world.animation_graph_player(entity).unwrap().clone();
        player.playing = false;
        world
            .set_animation_graph_player(entity, Some(player))
            .unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());

    assets.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        single_state_machine(&graph_uri),
    );
    level.with_world_mut(|world| {
        world.set_animation_graph_player(entity, None).unwrap();
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
    });
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert!(level.animation_requires_continuous_frame());
    level.with_world_mut(|world| {
        let mut player = world
            .animation_state_machine_player(entity)
            .unwrap()
            .clone();
        player.playing = false;
        world
            .set_animation_state_machine_player(entity, Some(player))
            .unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
    level.with_world_mut(|world| {
        let mut player = world
            .animation_state_machine_player(entity)
            .unwrap()
            .clone();
        player.playing = true;
        player.state_machine = ResourceHandle::<AnimationStateMachineMarker>::new(
            ResourceId::from_stable_label("frame-demand.unavailable-machine"),
        );
        world
            .set_animation_state_machine_player(entity, Some(player))
            .unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(!level.animation_requires_continuous_frame());
}
