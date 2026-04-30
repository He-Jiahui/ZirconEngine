use crate::{build_world_sync_state, integrate_builtin_physics_steps, PhysicsTickPlan};
use zircon_runtime::core::framework::physics::PhysicsMaterialMetadata;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, JointComponent, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::world::World;

#[test]
fn builtin_physics_step_skips_translation_writeback_when_position_would_overflow() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    let initial_translation = Vec3::new(3.0e38, 0.0, 0.0);

    world
        .update_transform(entity, Transform::from_translation(initial_translation))
        .unwrap();
    world
        .set_rigid_body(
            entity,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Kinematic,
                linear_velocity: Vec3::new(1.0e38, 0.0, 0.0),
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();

    integrate_builtin_physics_steps(
        &mut world,
        PhysicsTickPlan {
            steps: 1,
            step_seconds: 1.0,
            remaining_seconds: 0.0,
        },
    );

    let transform = world.find_node(entity).unwrap().transform;
    assert_eq!(transform.translation, initial_translation);
    assert!(transform.translation.x.is_finite());
}

#[test]
fn build_world_sync_state_skips_collider_when_combined_transform_would_overflow() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(3.0e38, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(0.5),
                },
                local_transform: Transform::from_translation(Vec3::new(1.0e38, 0.0, 0.0)),
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(501), &world);

    assert!(
        sync.colliders.is_empty(),
        "collider sync must not export transforms that overflow to non-finite values"
    );
}

#[test]
fn build_world_sync_state_skips_collider_when_sphere_radius_is_non_finite() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere {
                    radius: f32::INFINITY,
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(505), &world);

    assert!(
        sync.colliders.is_empty(),
        "collider sync must not export non-finite sphere radius values"
    );
}

#[test]
fn build_world_sync_state_skips_collider_when_material_override_is_non_finite() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                material_override: Some(PhysicsMaterialMetadata {
                    static_friction: f32::NAN,
                    ..PhysicsMaterialMetadata::default()
                }),
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(506), &world);

    assert!(
        sync.colliders.is_empty() && sync.materials.is_empty(),
        "collider sync must not export non-finite material override values"
    );
}

#[test]
fn build_world_sync_state_skips_collider_when_layer_exceeds_mask_bits() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                layer: 32,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(508), &world);

    assert!(
        sync.colliders.is_empty(),
        "collider sync must not export layers that cannot map to a u32 mask bit"
    );
}

#[test]
fn build_world_sync_state_skips_body_when_transform_is_non_finite() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(f32::INFINITY, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(502), &world);

    assert!(
        sync.bodies.is_empty(),
        "body sync must not export transforms with non-finite values"
    );
}

#[test]
fn build_world_sync_state_skips_body_when_mass_is_non_positive() {
    let mut world = World::new();
    let zero_mass = world.spawn_node(NodeKind::Cube);
    world
        .set_rigid_body(
            zero_mass,
            Some(RigidBodyComponent {
                mass: 0.0,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    let negative_mass = world.spawn_node(NodeKind::Cube);
    world
        .set_rigid_body(
            negative_mass,
            Some(RigidBodyComponent {
                mass: -1.0,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(507), &world);

    assert!(
        sync.bodies.is_empty(),
        "body sync must not export non-positive mass values"
    );
}

#[test]
fn build_world_sync_state_skips_joint_when_axis_is_non_finite() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .set_joint(
            entity,
            Some(JointComponent {
                axis: Vec3::new(f32::INFINITY, 0.0, 0.0),
                ..JointComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(503), &world);

    assert!(
        sync.joints.is_empty(),
        "joint sync must not export non-finite axis values"
    );
}

#[test]
fn build_world_sync_state_skips_joint_when_limits_are_non_finite() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    world
        .set_joint(
            entity,
            Some(JointComponent {
                limits: Some([0.0, f32::INFINITY]),
                ..JointComponent::default()
            }),
        )
        .unwrap();

    let sync = build_world_sync_state(WorldHandle::new(504), &world);

    assert!(
        sync.joints.is_empty(),
        "joint sync must not export non-finite limit values"
    );
}
