use crate::{build_world_sync_state, DefaultPhysicsManager};
use zircon_runtime::core::framework::physics::{
    PhysicsManager, PhysicsRayCastQuery, PhysicsSettings, PhysicsSimulationMode,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, NodeKind};
use zircon_runtime::scene::world::World;

#[test]
fn physics_manager_contacts_do_not_overflow_large_sphere_overlap_test() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const CENTER_OFFSET: f32 = 1.1e38;
    const RADIUS: f32 = 1.0e38;

    let left = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            left,
            Transform::from_translation(Vec3::new(-CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            left,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let right = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            right,
            Transform::from_translation(Vec3::new(CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            right,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(109);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "large finite sphere overlap checks must not collapse both sides to infinity"
    );
}

#[test]
fn physics_manager_contacts_keep_large_contact_normal_finite_test() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const CENTER_OFFSET: f32 = 5.0e37;
    const RADIUS: f32 = 1.0e38;

    let left = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            left,
            Transform::from_translation(Vec3::new(-CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            left,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let right = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            right,
            Transform::from_translation(Vec3::new(CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            right,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(116);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let contacts = manager.drain_contacts(world_handle);
    assert_eq!(contacts.len(), 1);
    let contact = &contacts[0];
    assert_eq!((contact.entity, contact.other_entity), (left, right));
    assert!(contact.normal.iter().all(|component| component.is_finite()));
    assert_eq!(contact.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_contacts_do_not_overflow_large_sphere_capsule_overlap_test() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const CENTER_OFFSET: f32 = 1.1e38;
    const RADIUS: f32 = 1.0e38;

    let sphere = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            sphere,
            Transform::from_translation(Vec3::new(-CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            sphere,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let capsule = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            capsule,
            Transform::from_translation(Vec3::new(CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            capsule,
            Some(ColliderComponent {
                shape: ColliderShape::Capsule {
                    radius: RADIUS,
                    half_height: 0.0,
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(110);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "large finite sphere/capsule overlap checks must not collapse both sides to infinity"
    );
}

#[test]
fn physics_manager_contacts_do_not_overflow_large_sphere_box_overlap_test() {
    assert_large_finite_separated_colliders_do_not_contact(
        ColliderShape::Sphere { radius: 1.0e38 },
        ColliderShape::Box {
            half_extents: Vec3::splat(0.5),
        },
        WorldHandle::new(111),
        "large finite sphere/box overlap checks must not collapse both sides to infinity",
    );
}

#[test]
fn physics_manager_contacts_do_not_overflow_large_capsule_box_overlap_test() {
    assert_large_finite_separated_colliders_do_not_contact(
        ColliderShape::Capsule {
            radius: 1.0e38,
            half_height: 0.0,
        },
        ColliderShape::Box {
            half_extents: Vec3::splat(0.5),
        },
        WorldHandle::new(112),
        "large finite capsule/box overlap checks must not collapse both sides to infinity",
    );
}

#[test]
fn physics_manager_contacts_do_not_overflow_large_capsule_capsule_overlap_test() {
    assert_large_finite_separated_colliders_do_not_contact(
        ColliderShape::Capsule {
            radius: 1.0e38,
            half_height: 0.0,
        },
        ColliderShape::Capsule {
            radius: 1.0e38,
            half_height: 0.0,
        },
        WorldHandle::new(113),
        "large finite capsule/capsule overlap checks must not collapse both sides to infinity",
    );
}

fn assert_large_finite_separated_colliders_do_not_contact(
    left_shape: ColliderShape,
    right_shape: ColliderShape,
    world_handle: WorldHandle,
    failure_message: &str,
) {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const CENTER_OFFSET: f32 = 1.1e38;

    let left = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            left,
            Transform::from_translation(Vec3::new(-CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            left,
            Some(ColliderComponent {
                shape: left_shape,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let right = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            right,
            Transform::from_translation(Vec3::new(CENTER_OFFSET, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            right,
            Some(ColliderComponent {
                shape: right_shape,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "{failure_message}"
    );
}

#[test]
fn physics_manager_contacts_respect_collider_collision_masks() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let first = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            first,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                layer: 1,
                collision_mask: 1 << 1,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let second = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            second,
            Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            second,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                layer: 2,
                collision_mask: 1 << 2,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(103);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "overlapping colliders with masks that exclude each other's layers must not emit contacts"
    );
}

#[test]
fn physics_manager_contacts_respect_project_collision_matrix() {
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "contract_snapshot".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            layer_names: vec![
                "default".to_string(),
                "characters".to_string(),
                "projectiles".to_string(),
            ],
            collision_matrix: vec![1 << 0, 1 << 1, 1 << 2],
            ..PhysicsSettings::default()
        })
        .unwrap();

    let mut world = World::new();
    let character = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            character,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                layer: 1,
                collision_mask: 1 << 2,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let projectile = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            projectile,
            Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            projectile,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                layer: 2,
                collision_mask: 1 << 1,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(105);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "project collision matrix must reject contacts even when collider masks allow each other"
    );
}

#[test]
fn physics_manager_contacts_skip_sensor_colliders_but_queries_can_include_them() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let sensor = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            sensor,
            Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        )
        .unwrap();
    world
        .set_collider(
            sensor,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                sensor: true,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let solid = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(solid, Transform::from_translation(Vec3::new(0.5, 0.0, 4.0)))
        .unwrap();
    world
        .set_collider(
            solid,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(107);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "sensor colliders must not emit physics contact events"
    );

    let sensor_hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray query with include_sensors=true should hit the sensor collider");
    assert_eq!(sensor_hit.entity, sensor);

    let solid_hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: false,
        })
        .expect("ray query with include_sensors=false should still hit the solid collider");
    assert_eq!(solid_hit.entity, solid);
}
