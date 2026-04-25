use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::pipeline::manager::AssetManager;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationConditionOperatorAsset,
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset, AssetUri, ProjectAssetManager,
};
use zircon_runtime::core::framework::animation::{
    AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseSource,
};
use zircon_runtime::core::framework::physics::{PhysicsSettings, PhysicsSimulationMode};
use zircon_runtime::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, SceneViewportExtractRequest,
    ViewportRenderSettings,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::{create_default_level, SCENE_MODULE_NAME};

fn create_runtime_with_scene_and_physics() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_runtime::foundation::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::physics::module_descriptor())
        .unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
}

fn create_runtime_with_scene_physics_and_animation() -> CoreRuntime {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .register_module(zircon_runtime::animation::module_descriptor())
        .unwrap();
    runtime
        .activate_module(zircon_runtime::animation::ANIMATION_MODULE_NAME)
        .unwrap();
    runtime
}

fn create_runtime_with_scene_physics_animation_and_assets() -> CoreRuntime {
    let runtime = create_runtime_with_scene_physics_and_animation();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_tick_{label}_{unique}"))
}

fn create_sequence_test_project(root: &PathBuf) {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "SceneTickSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let sequence = AnimationSequenceAsset {
        name: Some("HeroSequence".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: vec3_channel([(0.0, [0.0, 0.0, 0.0]), (1.0, [1.0, 0.0, 0.0])]),
                },
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                    channel: scalar_channel([(0.0, 0.0), (1.0, 1.0)]),
                },
            ],
        }],
    };

    let sequence_path = paths
        .assets_root()
        .join("animation")
        .join("hero.sequence.zranim");
    if let Some(parent) = sequence_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(sequence_path, sequence.to_bytes().unwrap()).unwrap();
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.skeleton.zranim"),
        sample_animation_skeleton_asset().to_bytes().unwrap(),
    )
    .unwrap();
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.clip.zranim"),
        sample_animation_clip_asset().to_bytes().unwrap(),
    )
    .unwrap();
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.graph.zranim"),
        sample_animation_graph_asset().to_bytes().unwrap(),
    )
    .unwrap();
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.state_machine.zranim"),
        sample_animation_state_machine_asset().to_bytes().unwrap(),
    )
    .unwrap();
}

fn asset_reference(uri: &str) -> zircon_runtime::asset::AssetReference {
    zircon_runtime::asset::AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn sample_animation_skeleton_asset() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
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
                local_translation: [0.2, 0.8, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    }
}

fn sample_animation_clip_asset() -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("HeroIdle".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.2, 0.8, 0.0]), (1.0, [0.25, 0.85, 0.0])]),
            rotation: quaternion_channel([
                (0.0, [0.0, 0.0, 0.0, 1.0]),
                (1.0, [0.0, 0.38268343, 0.0, 0.9238795]),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [1.05, 1.05, 1.05])]),
        }],
    }
}

fn sample_animation_graph_asset() -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "speed".to_string(),
            default_value: AnimationParameterValue::Scalar(0.0),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Output {
                source: "idle".to_string(),
            },
        ],
    }
}

fn sample_animation_state_machine_asset() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/hero.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Locomotion".to_string(),
                graph: asset_reference("res://animation/hero.graph.zranim"),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Locomotion".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::Greater,
                value: Some(AnimationParameterValue::Scalar(0.5)),
            }],
        }],
    }
}

fn scalar_channel(keys: [(f32, f32); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Scalar(value),
                in_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                out_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
            })
            .collect(),
    }
}

fn vec3_channel(keys: [(f32, [f32; 3]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Vec3(value),
                in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
            })
            .collect(),
    }
}

fn quaternion_channel(keys: [(f32, [f32; 4]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Quaternion(value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    }
}

#[test]
fn level_tick_integrates_dynamic_rigid_body_linear_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<zircon_runtime::physics::DefaultPhysicsManager>(
            "PhysicsModule.Manager.DefaultPhysicsManager",
        )
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(body, Transform::from_translation(Vec3::ZERO))
            .unwrap();
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::new(2.0, 0.0, 0.0),
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        body
    });

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let (translation, velocity) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform.translation,
            world.rigid_body(body).unwrap().linear_velocity,
        )
    });
    assert!((translation.x - (2.0 / 60.0)).abs() < 1.0e-6);
    assert_eq!(translation.y, 0.0);
    assert_eq!(translation.z, 0.0);
    assert_eq!(velocity, Vec3::new(2.0, 0.0, 0.0));
}

#[test]
fn level_tick_integrates_dynamic_rigid_body_angular_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<zircon_runtime::physics::DefaultPhysicsManager>(
            "PhysicsModule.Manager.DefaultPhysicsManager",
        )
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    angular_velocity: Vec3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0),
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        body
    });

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let (rotation, angular_velocity) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform.rotation,
            world.rigid_body(body).unwrap().angular_velocity,
        )
    });
    let expected = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 / 60.0);
    assert!(
        rotation.abs_diff_eq(expected, 1.0e-6),
        "expected {expected:?}, got {rotation:?}"
    );
    assert_eq!(
        angular_velocity,
        Vec3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0)
    );
}

#[test]
fn level_tick_integrates_kinematic_rigid_body_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<zircon_runtime::physics::DefaultPhysicsManager>(
            "PhysicsModule.Manager.DefaultPhysicsManager",
        )
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Kinematic,
                    linear_velocity: Vec3::new(3.0, 0.0, 0.0),
                    angular_velocity: Vec3::new(0.0, std::f32::consts::PI, 0.0),
                    linear_damping: 30.0,
                    angular_damping: 30.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        body
    });

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let (transform, rigid_body) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform,
            world.rigid_body(body).unwrap().clone(),
        )
    });
    assert!((transform.translation.x - (3.0 / 60.0)).abs() < 1.0e-6);
    assert_eq!(transform.translation.y, 0.0);
    assert_eq!(transform.translation.z, 0.0);
    let expected_rotation = Quat::from_rotation_y(std::f32::consts::PI / 60.0);
    assert!(
        transform.rotation.abs_diff_eq(expected_rotation, 1.0e-6),
        "expected {expected_rotation:?}, got {:?}",
        transform.rotation
    );
    assert_eq!(rigid_body.linear_velocity, Vec3::new(3.0, 0.0, 0.0));
    assert_eq!(
        rigid_body.angular_velocity,
        Vec3::new(0.0, std::f32::consts::PI, 0.0)
    );
}

#[test]
fn level_tick_syncs_world_and_records_transient_physics_state() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();
    let (left, right) = level.with_world_mut(|world| {
        let left = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(left, Transform::from_translation(Vec3::ZERO))
            .unwrap();
        world
            .set_rigid_body(
                left,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Static,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        world
            .set_collider(
                left,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(0.5),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let right = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(right, Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)))
            .unwrap();
        world
            .set_rigid_body(
                right,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        world
            .set_collider(
                right,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(0.5),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        (left, right)
    });

    assert!(physics.synchronized_world(level.handle()).is_none());
    assert!(level.last_physics_step_plan().is_none());
    assert!(level.physics_contacts().is_empty());

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert!(physics.synchronized_world(level.handle()).is_some());
    let plan = level
        .last_physics_step_plan()
        .expect("expected tick to store a physics step plan");
    assert!(plan.step_seconds > 0.0);
    assert!(plan.remaining_seconds >= 0.0);

    let contacts = level.physics_contacts();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].world, level.handle());
    assert_eq!(contacts[0].entity, left);
    assert_eq!(contacts[0].other_entity, right);
}

#[test]
fn level_tick_advances_playing_animation_clocks() {
    let runtime = create_runtime_with_scene_physics_and_animation();
    let level = create_default_level(&runtime.handle()).unwrap();
    let (clip_entity, sequence_entity, paused_entity) = level.with_world_mut(|world| {
        let clip_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_player(
                clip_entity,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/hero.clip.zranim",
                        ),
                    ),
                    playback_speed: 2.0,
                    time_seconds: 1.5,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();

        let sequence_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_sequence_player(
                sequence_entity,
                Some(AnimationSequencePlayerComponent {
                    sequence: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/hero.sequence.zranim",
                        ),
                    ),
                    playback_speed: 0.5,
                    time_seconds: 2.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();

        let paused_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_player(
                paused_entity,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/hero_idle.clip.zranim",
                        ),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 4.0,
                    weight: 1.0,
                    looping: true,
                    playing: false,
                }),
            )
            .unwrap();

        (clip_entity, sequence_entity, paused_entity)
    });

    level.tick(&runtime.handle(), 0.25).unwrap();

    level.with_world(|world| {
        let clip_player = world.animation_player(clip_entity).unwrap();
        let sequence_player = world.animation_sequence_player(sequence_entity).unwrap();
        let paused_player = world.animation_player(paused_entity).unwrap();

        assert!(
            (clip_player.time_seconds - 2.0).abs() < 1.0e-6,
            "expected playing clip clock to advance by delta * playback_speed"
        );
        assert!(
            (sequence_player.time_seconds - 2.125).abs() < 1.0e-6,
            "expected playing sequence clock to advance by delta * playback_speed"
        );
        assert!(
            (paused_player.time_seconds - 4.0).abs() < 1.0e-6,
            "expected paused clip clock to remain unchanged"
        );
    });
}

#[test]
fn level_tick_respects_animation_playback_settings_gates() {
    let runtime = create_runtime_with_scene_physics_and_animation();
    let animation_manager = runtime
        .handle()
        .resolve_manager::<zircon_runtime::animation::DefaultAnimationManager>(
            "AnimationModule.Manager.DefaultAnimationManager",
        )
        .unwrap();
    animation_manager
        .store_playback_settings(AnimationPlaybackSettings {
            enabled: true,
            property_tracks: false,
            skeletal_clips: false,
            graphs: true,
            state_machines: true,
        })
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let (clip_entity, sequence_entity) = level.with_world_mut(|world| {
        let clip_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_player(
                clip_entity,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/hero.clip.zranim",
                        ),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 1.0,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();

        let sequence_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_sequence_player(
                sequence_entity,
                Some(AnimationSequencePlayerComponent {
                    sequence: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/hero.sequence.zranim",
                        ),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 2.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();

        (clip_entity, sequence_entity)
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    level.with_world(|world| {
        assert!(
            (world.animation_player(clip_entity).unwrap().time_seconds - 1.0).abs() < 1.0e-6,
            "expected skeletal clip playback gate to block clip clock advancement"
        );
        assert!(
            (world
                .animation_sequence_player(sequence_entity)
                .unwrap()
                .time_seconds
                - 2.0)
                .abs()
                < 1.0e-6,
            "expected property-track playback gate to block sequence clock advancement"
        );
    });
}

#[test]
fn level_tick_applies_asset_backed_sequence_tracks_to_world() {
    let root = unique_temp_project_root("asset_sequence");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let sequence_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let root_entity = world.spawn_node(NodeKind::Mesh);
        world.rename_node(root_entity, "Root").unwrap();

        let hero = world.spawn_node(NodeKind::Mesh);
        world.rename_node(hero, "Hero").unwrap();
        world.set_parent_checked(hero, Some(root_entity)).unwrap();
        world
            .set_animation_player(
                hero,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/placeholder.clip.zranim",
                        ),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 0.1,
                    looping: true,
                    playing: false,
                }),
            )
            .unwrap();
        world
            .set_animation_sequence_player(
                hero,
                Some(AnimationSequencePlayerComponent {
                    sequence: zircon_runtime::core::resource::ResourceHandle::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    level.with_world(|world| {
        let node = world.find_node(hero).unwrap();
        let animation_player = world.animation_player(hero).unwrap();
        let sequence_player = world.animation_sequence_player(hero).unwrap();

        assert!(
            (node.transform.translation.x - 0.5).abs() < 1.0e-6,
            "expected asset-backed sequence track to update world transform"
        );
        assert!(
            (animation_player.weight - 0.5).abs() < 1.0e-6,
            "expected asset-backed sequence track to update component property"
        );
        assert!(
            (sequence_player.time_seconds - 0.5).abs() < 1.0e-6,
            "expected sequence playback clock to advance before sampling"
        );
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_records_asset_backed_clip_pose_output() {
    let root = unique_temp_project_root("clip_pose");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let clip_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.clip.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                hero,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache sampled clip pose");
    assert_eq!(pose.source, AnimationPoseSource::Clip);
    assert_eq!(pose.bones.len(), 2);
    let hand = pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("expected sampled hand bone");
    assert!(
        (hand.local_transform.translation.x - 0.225).abs() < 1.0e-6,
        "expected clip sampling to interpolate hand translation at current clip time"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_records_asset_backed_graph_pose_output() {
    let root = unique_temp_project_root("graph_pose");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let graph_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                hero,
                Some(AnimationGraphPlayerComponent {
                    graph: zircon_runtime::core::resource::ResourceHandle::new(graph_id),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache evaluated graph pose");
    assert_eq!(pose.source, AnimationPoseSource::Graph);
    assert_eq!(pose.active_state, None);
    assert_eq!(pose.bones.len(), 2);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_clamps_non_looping_clip_pose_at_last_frame() {
    let root = unique_temp_project_root("clip_non_looping");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let clip_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.clip.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                hero,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.75,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache sampled clip pose");
    assert_eq!(pose.source, AnimationPoseSource::Clip);
    assert!(
        (sampled_hand_translation_x(pose) - 0.25).abs() < 1.0e-6,
        "expected non-looping clip playback to clamp to the last frame after overshooting duration"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_wraps_looping_sequence_sample_time() {
    let root = unique_temp_project_root("sequence_looping_wrap");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let sequence_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let root_entity = world.spawn_node(NodeKind::Mesh);
        world.rename_node(root_entity, "Root").unwrap();

        let hero = world.spawn_node(NodeKind::Mesh);
        world.rename_node(hero, "Hero").unwrap();
        world.set_parent_checked(hero, Some(root_entity)).unwrap();
        world
            .set_animation_player(
                hero,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(
                        zircon_runtime::core::resource::ResourceId::from_stable_label(
                            "res://animation/placeholder.clip.zranim",
                        ),
                    ),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 0.0,
                    looping: false,
                    playing: false,
                }),
            )
            .unwrap();
        world
            .set_animation_sequence_player(
                hero,
                Some(AnimationSequencePlayerComponent {
                    sequence: zircon_runtime::core::resource::ResourceHandle::new(sequence_id),
                    playback_speed: 1.0,
                    time_seconds: 0.75,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    level.with_world(|world| {
        let node = world.find_node(hero).unwrap();
        let animation_player = world.animation_player(hero).unwrap();

        assert!(
            (node.transform.translation.x - 0.15625).abs() < 1.0e-6,
            "expected looping sequence translation track to wrap to 0.25s instead of clamping to the end"
        );
        assert!(
            (animation_player.weight - 0.15625).abs() < 1.0e-6,
            "expected looping sequence scalar track to wrap to 0.25s instead of clamping to the end"
        );
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_clamps_non_looping_graph_clip_pose() {
    let root = unique_temp_project_root("graph_non_looping");
    create_sequence_test_project(&root);

    let paths = ProjectPaths::from_root(&root).unwrap();
    let mut graph = sample_animation_graph_asset();
    for node in &mut graph.nodes {
        if let AnimationGraphNodeAsset::Clip { looping, .. } = node {
            *looping = false;
        }
    }
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.graph.zranim"),
        graph.to_bytes().unwrap(),
    )
    .unwrap();

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let graph_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                hero,
                Some(AnimationGraphPlayerComponent {
                    graph: zircon_runtime::core::resource::ResourceHandle::new(graph_id),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 1.25).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache evaluated graph pose");
    assert_eq!(pose.source, AnimationPoseSource::Graph);
    assert!(
        (sampled_hand_translation_x(pose) - 0.25).abs() < 1.0e-6,
        "expected non-looping graph clip to clamp to the last frame after overshooting duration"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_clamps_non_looping_state_machine_clip_pose() {
    let root = unique_temp_project_root("state_machine_non_looping");
    create_sequence_test_project(&root);

    let paths = ProjectPaths::from_root(&root).unwrap();
    let mut graph = sample_animation_graph_asset();
    for node in &mut graph.nodes {
        if let AnimationGraphNodeAsset::Clip { looping, .. } = node {
            *looping = false;
        }
    }
    fs::write(
        paths
            .assets_root()
            .join("animation")
            .join("hero.graph.zranim"),
        graph.to_bytes().unwrap(),
    )
    .unwrap();

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let state_machine_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                hero,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: zircon_runtime::core::resource::ResourceHandle::new(
                        state_machine_id,
                    ),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    active_state: None,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 1.25).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache evaluated state-machine pose");
    assert_eq!(pose.source, AnimationPoseSource::StateMachine);
    assert_eq!(pose.active_state.as_deref(), Some("Locomotion"));
    assert!(
        (sampled_hand_translation_x(pose) - 0.25).abs() < 1.0e-6,
        "expected non-looping state-machine clip to clamp to the last frame after overshooting duration"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_records_asset_backed_state_machine_pose_output_and_active_state() {
    let root = unique_temp_project_root("state_machine_pose");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let state_machine_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                hero,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: zircon_runtime::core::resource::ResourceHandle::new(
                        state_machine_id,
                    ),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    active_state: None,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    let pose = level
        .animation_pose(hero)
        .expect("expected tick to cache evaluated state-machine pose");
    assert_eq!(pose.source, AnimationPoseSource::StateMachine);
    assert_eq!(pose.active_state.as_deref(), Some("Locomotion"));
    level.with_world(|world| {
        let player = world
            .animation_state_machine_player(hero)
            .expect("expected state-machine player to remain attached");
        assert_eq!(player.active_state.as_deref(), Some("Locomotion"));
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_ignores_paused_graph_and_state_machine_players() {
    let root = unique_temp_project_root("paused_graph_state_machine");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let graph_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
        .unwrap();
    let state_machine_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let (graph_entity, state_machine_entity) = level.with_world_mut(|world| {
        let graph_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                graph_entity,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                graph_entity,
                Some(AnimationGraphPlayerComponent {
                    graph: zircon_runtime::core::resource::ResourceHandle::new(graph_id),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    playing: false,
                }),
            )
            .unwrap();

        let state_machine_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                state_machine_entity,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                state_machine_entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: zircon_runtime::core::resource::ResourceHandle::new(
                        state_machine_id,
                    ),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    active_state: None,
                    playing: false,
                }),
            )
            .unwrap();

        (graph_entity, state_machine_entity)
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    assert!(
        level.animation_pose(graph_entity).is_none(),
        "paused graph player should not emit a cached pose"
    );
    assert!(
        level.animation_pose(state_machine_entity).is_none(),
        "paused state-machine player should not emit a cached pose"
    );
    level.with_world(|world| {
        let player = world
            .animation_state_machine_player(state_machine_entity)
            .expect("expected paused state-machine player to remain attached");
        assert_eq!(
            player.active_state, None,
            "paused state-machine player should not advance active state"
        );
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_tick_accumulates_graph_and_state_machine_playback_time_across_ticks() {
    let root = unique_temp_project_root("graph_state_machine_time");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let graph_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
        .unwrap();
    let state_machine_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let (graph_entity, state_machine_entity) = level.with_world_mut(|world| {
        let graph_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                graph_entity,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                graph_entity,
                Some(AnimationGraphPlayerComponent {
                    graph: zircon_runtime::core::resource::ResourceHandle::new(graph_id),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    playing: true,
                }),
            )
            .unwrap();

        let state_machine_entity = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                state_machine_entity,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                state_machine_entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: zircon_runtime::core::resource::ResourceHandle::new(
                        state_machine_id,
                    ),
                    parameters: BTreeMap::from([(
                        "speed".to_string(),
                        AnimationParameterValue::Scalar(1.0),
                    )]),
                    active_state: None,
                    playing: true,
                }),
            )
            .unwrap();

        (graph_entity, state_machine_entity)
    });

    level.tick(&runtime.handle(), 0.25).unwrap();
    let first_graph_x = sampled_hand_translation_x(
        level
            .animation_pose(graph_entity)
            .expect("expected first graph pose sample"),
    );
    let first_state_machine_pose = level
        .animation_pose(state_machine_entity)
        .expect("expected first state-machine pose sample");
    let first_state_machine_x = sampled_hand_translation_x(first_state_machine_pose.clone());
    assert_eq!(
        first_state_machine_pose.active_state.as_deref(),
        Some("Locomotion"),
        "expected first state-machine tick to transition into Locomotion"
    );

    level.tick(&runtime.handle(), 0.25).unwrap();
    let second_graph_x = sampled_hand_translation_x(
        level
            .animation_pose(graph_entity)
            .expect("expected second graph pose sample"),
    );
    let second_state_machine_x = sampled_hand_translation_x(
        level
            .animation_pose(state_machine_entity)
            .expect("expected second state-machine pose sample"),
    );

    assert!(
        second_graph_x > first_graph_x + 1.0e-6,
        "expected graph playback time to accumulate across ticks"
    );
    assert!(
        second_state_machine_x > first_state_machine_x + 1.0e-6,
        "expected state-machine playback time to accumulate across ticks"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_render_extract_carries_cached_clip_pose_for_skinned_entity() {
    let root = unique_temp_project_root("render_extract_animation_pose");
    create_sequence_test_project(&root);

    let runtime = create_runtime_with_scene_physics_animation_and_assets();
    let asset_manager = runtime
        .handle()
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let skeleton_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap())
        .unwrap();
    let clip_id = asset_manager
        .resolve_asset_id(&AssetUri::parse("res://animation/hero.clip.zranim").unwrap())
        .unwrap();

    let level = create_default_level(&runtime.handle()).unwrap();
    let hero = level.with_world_mut(|world| {
        let hero = world.spawn_node(NodeKind::Mesh);
        world
            .set_animation_skeleton(
                hero,
                Some(AnimationSkeletonComponent {
                    skeleton: zircon_runtime::core::resource::ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                hero,
                Some(AnimationPlayerComponent {
                    clip: zircon_runtime::core::resource::ResourceHandle::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        hero
    });

    level.tick(&runtime.handle(), 0.5).unwrap();

    let extract = level.build_render_frame_extract(&RenderExtractContext::new(
        level.handle().into(),
        SceneViewportExtractRequest {
            settings: ViewportRenderSettings::default(),
            active_camera_override: None,
            camera: None,
            viewport_size: None,
            virtual_geometry_debug: None,
        },
    ));

    let pose = extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == hero)
        .expect("expected render extract to carry cached pose for the skinned mesh entity");
    assert_eq!(pose.skeleton, skeleton_id);
    assert_eq!(pose.pose.source, AnimationPoseSource::Clip);
    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == hero),
        "expected render extract to keep the skinned mesh in the geometry list"
    );

    let _ = fs::remove_dir_all(root);
}

fn sampled_hand_translation_x(
    pose: zircon_runtime::core::framework::animation::AnimationPoseOutput,
) -> f32 {
    pose.bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("expected sampled hand bone")
        .local_transform
        .translation
        .x
}
