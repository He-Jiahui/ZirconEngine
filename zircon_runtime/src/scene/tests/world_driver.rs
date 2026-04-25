use crate::core::framework::physics::{PhysicsSettings, PhysicsSimulationMode};
use crate::core::manager::resolve_physics_manager;
use crate::core::math::{Transform, Vec3};
use crate::core::CoreRuntime;
use crate::foundation::FOUNDATION_MODULE_NAME;
use crate::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::{create_default_level, SCENE_MODULE_NAME};

fn create_runtime_with_scene_and_physics() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::foundation::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::physics::module_descriptor())
        .unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime
        .activate_module(crate::physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
}

#[test]
fn level_tick_integrates_dynamic_rigid_body_linear_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .handle()
        .resolve_manager::<crate::physics::DefaultPhysicsManager>(
            crate::physics::DEFAULT_PHYSICS_MANAGER_NAME,
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
fn level_tick_syncs_world_snapshot_into_physics_manager() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    assert!(physics.synchronized_world(level.handle()).is_none());

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert!(physics.synchronized_world(level.handle()).is_some());
}

#[test]
fn level_tick_records_transient_physics_state() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
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

    assert!(level.last_physics_step_plan().is_none());
    assert!(level.physics_contacts().is_empty());

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

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
