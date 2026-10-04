use super::*;
use zircon_runtime::core::framework::physics::{
    PhysicsBodyType, PhysicsColliderShape, PhysicsJointType,
};
use zircon_runtime::core::math::Transform;

#[test]
fn joint_resolves_entity_pair_to_handles() {
    let mut world = JoltManagedWorld::new(PhysicsSettings::default()).expect("Jolt world");
    let world_handle = WorldHandle::new(41);
    let material = PhysicsMaterialMetadata::default();
    for entity in [411, 412] {
        let transform = Transform::default();
        let body = PhysicsBodySyncState {
            entity,
            body_type: PhysicsBodyType::Dynamic,
            transform,
            mass: 1.0,
            mass_properties: Default::default(),
            linear_velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 0.0,
            ccd_mode: Default::default(),
            sleep_policy: Default::default(),
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        };
        let collider = PhysicsColliderSyncState {
            entity,
            shape: PhysicsColliderShape::Sphere { radius: 0.25 },
            sensor: false,
            layer: 0,
            collision_group: 0,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform,
        };
        world
            .synchronize_entity(world_handle, entity, &body, &collider, &material)
            .expect("synchronize body");
    }
    let joint = PhysicsJointSyncState {
        entity: 411,
        kind: PhysicsJointType::Fixed,
        connected_entity: Some(412),
        anchor: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        limits: None,
        collide_connected: false,
        constraint: Default::default(),
        skeleton_binding: None,
    };

    let (body_a, body_b) = world
        .resolved_constraint_handles(&joint)
        .expect("resolve joint body pair");
    assert_eq!(body_a, world.entities[&411].body);
    assert_eq!(body_b, Some(world.entities[&412].body));
}
