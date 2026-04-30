use crate::{build_world_sync_state, DefaultPhysicsManager};
use zircon_runtime::core::framework::physics::{PhysicsManager, PhysicsRayCastQuery};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, NodeKind};
use zircon_runtime::scene::world::World;

#[test]
fn physics_manager_ray_cast_reports_exit_hit_when_origin_starts_inside_sphere() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let sphere = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            sphere,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: 1.0 },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(101);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("expected exit hit when ray starts inside sphere collider");

    assert_eq!(hit.entity, sphere);
    assert!((hit.distance - 1.0).abs() < 1.0e-4);
    assert_eq!(hit.position, [1.0, 0.0, 0.0]);
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_accepts_large_finite_direction() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let sphere = world.spawn_node(NodeKind::Cube);

    world
        .update_transform(
            sphere,
            Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            sphere,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: 1.0 },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(118);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [1.0e38, 0.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("large finite direction should still normalize to a valid ray");

    assert_eq!(hit.entity, sphere);
    assert!((hit.distance - 3.0).abs() < 1.0e-4);
    assert_eq!(hit.position, [3.0, 0.0, 0.0]);
    assert_eq!(hit.normal, [-1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_inside_large_sphere_returns_finite_exit_hit() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const RADIUS: f32 = 1.0e38;

    let sphere = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            sphere,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(114);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            max_distance: 2.0e38,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("expected finite exit hit from inside huge sphere collider");

    assert_eq!(hit.entity, sphere);
    assert!(hit.distance.is_finite(), "hit distance must stay finite");
    assert!(
        (hit.distance - RADIUS).abs() <= RADIUS * 1.0e-6,
        "unexpected large sphere exit distance: {}",
        hit.distance
    );
    assert!(hit.position.iter().all(|component| component.is_finite()));
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_skips_sphere_when_hit_position_overflows() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const CENTER: f32 = 3.0e38;
    const RADIUS: f32 = 1.0e38;

    let sphere = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            sphere,
            Transform::from_translation(Vec3::new(CENTER, 0.0, 0.0)),
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

    let world_handle = WorldHandle::new(119);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager.ray_cast(&PhysicsRayCastQuery {
        world: world_handle,
        origin: [CENTER, 0.0, 0.0],
        direction: [1.0, 0.0, 0.0],
        max_distance: 2.0e38,
        collision_mask: None,
        include_sensors: true,
    });

    assert!(
        hit.is_none(),
        "ray fallback must not emit non-finite hit positions"
    );
}

#[test]
fn physics_manager_ray_cast_inside_large_capsule_returns_finite_exit_hit() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    const RADIUS: f32 = 1.0e38;

    let capsule = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            capsule,
            Some(ColliderComponent {
                shape: ColliderShape::Capsule {
                    radius: RADIUS,
                    half_height: 1.0,
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(115);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            max_distance: 2.0e38,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("expected finite exit hit from inside huge capsule collider");

    assert_eq!(hit.entity, capsule);
    assert!(hit.distance.is_finite(), "hit distance must stay finite");
    assert!(
        (hit.distance - RADIUS).abs() <= RADIUS * 1.0e-6,
        "unexpected large capsule exit distance: {}",
        hit.distance
    );
    assert!(hit.position.iter().all(|component| component.is_finite()));
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_reports_exit_hit_when_origin_starts_inside_box() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let box_entity = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            box_entity,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(102);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("expected exit hit when ray starts inside box collider");

    assert_eq!(hit.entity, box_entity);
    assert!((hit.distance - 1.0).abs() < 1.0e-4);
    assert_eq!(hit.position, [1.0, 0.0, 0.0]);
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_skips_box_when_scaled_bounds_overflow() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let box_entity = world.spawn_node(NodeKind::Cube);
    const CENTER: f32 = 3.0e38;
    const HALF_EXTENT: f32 = 1.0e38;

    world
        .update_transform(
            box_entity,
            Transform::from_translation(Vec3::new(CENTER, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_collider(
            box_entity,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(HALF_EXTENT),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(117);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager.ray_cast(&PhysicsRayCastQuery {
        world: world_handle,
        origin: [CENTER, 0.0, 0.0],
        direction: [-1.0, 0.0, 0.0],
        max_distance: 2.0e38,
        collision_mask: None,
        include_sensors: true,
    });

    assert!(
        hit.is_none(),
        "box ray fallback must reject non-finite AABB bounds"
    );
}

#[test]
fn physics_manager_ray_cast_reports_surface_normal_when_starting_on_box_surface() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let box_entity = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            box_entity,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(1.0),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(106);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [1.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("expected surface hit when ray starts on box surface");

    assert_eq!(hit.entity, box_entity);
    assert_eq!(hit.distance, 0.0);
    assert_eq!(hit.position, [1.0, 0.0, 0.0]);
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn physics_manager_ray_cast_query_mask_filters_by_collider_layer_not_collider_mask() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let target = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            target,
            Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        )
        .unwrap();
    world
        .set_collider(
            target,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(0.5),
                },
                layer: 2,
                collision_mask: 0,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(104);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: Some(1 << 2),
            include_sensors: true,
        })
        .expect(
            "query mask should select the collider's layer even when collider contact mask is empty",
        );

    assert_eq!(hit.entity, target);

    let missed = manager.ray_cast(&PhysicsRayCastQuery {
        world: world_handle,
        origin: [0.0, 0.0, 0.0],
        direction: [0.0, 0.0, 1.0],
        max_distance: 10.0,
        collision_mask: Some(1 << 1),
        include_sensors: true,
    });

    assert!(
        missed.is_none(),
        "query mask that excludes the collider layer must not hit"
    );
}

#[test]
fn physics_manager_ray_cast_uses_capsule_shape_instead_of_capsule_aabb() {
    let manager = DefaultPhysicsManager::default();
    let mut world = World::new();
    let capsule = world.spawn_node(NodeKind::Cube);
    world
        .set_collider(
            capsule,
            Some(ColliderComponent {
                shape: ColliderShape::Capsule {
                    radius: 1.0,
                    half_height: 2.0,
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(108);
    manager.sync_world(build_world_sync_state(world_handle, &world));

    let rounded_cap_corner_miss = manager.ray_cast(&PhysicsRayCastQuery {
        world: world_handle,
        origin: [0.9, 2.9, -5.0],
        direction: [0.0, 0.0, 1.0],
        max_distance: 10.0,
        collision_mask: None,
        include_sensors: true,
    });
    assert!(
        rounded_cap_corner_miss.is_none(),
        "ray lies inside the capsule AABB but outside the rounded capsule cap"
    );

    let rounded_cap_hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.5, 2.5, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray through the rounded capsule cap should hit");
    assert_eq!(rounded_cap_hit.entity, capsule);
    assert!((rounded_cap_hit.distance - 4.2928934).abs() < 1.0e-4);
}
