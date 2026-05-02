use crate::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, ProjectAssetManager,
};
use crate::core::framework::physics::{PhysicsSettings, PhysicsSimulationMode};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::manager::{resolve_animation_manager, resolve_physics_manager};
use crate::core::math::{Transform, Vec3};
use crate::core::resource::{AnimationSequenceMarker, ResourceHandle, ResourceId, ResourceRecord};
use crate::core::{CoreHandle, CoreRuntime};
use crate::scene::components::{
    AnimationSequencePlayerComponent, ColliderComponent, ColliderShape, NodeKind,
    RigidBodyComponent, RigidBodyType,
};
use crate::{animation, asset, foundation, physics, scene};

#[test]
fn builtin_runtime_resolves_physics_and_animation_managers() {
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
fn level_tick_applies_loaded_animation_sequences_to_world_properties() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let target_entity_name = "Runtime Sequence Target";
    let sequence_uri = crate::asset::AssetUri::parse("res://animation/test.sequence.zranim")
        .expect("test sequence locator");
    let sequence_id = ResourceId::from_locator(&sequence_uri);
    let asset_manager = runtime_asset_manager(&core);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            sequence_id,
            crate::core::resource::ResourceKind::AnimationSequence,
            sequence_uri,
        ),
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

fn runtime_with_physics_animation_scene_asset() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation::module_descriptor())
        .unwrap();
    runtime.register_module(asset::module_descriptor()).unwrap();
    runtime.register_module(scene::module_descriptor()).unwrap();
    runtime
        .register_module(physics::module_descriptor())
        .unwrap();
    runtime
        .register_module(animation::module_descriptor())
        .unwrap();
    runtime
        .activate_module(foundation::FOUNDATION_MODULE_NAME)
        .unwrap();
    runtime.activate_module(asset::ASSET_MODULE_NAME).unwrap();
    runtime.activate_module(scene::SCENE_MODULE_NAME).unwrap();
    runtime
        .activate_module(physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(animation::ANIMATION_MODULE_NAME)
        .unwrap();
    runtime
}

fn runtime_physics_manager(core: &CoreHandle) -> std::sync::Arc<physics::DefaultPhysicsManager> {
    core.resolve_manager::<physics::DefaultPhysicsManager>(physics::DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
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
