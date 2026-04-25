use super::*;

#[test]
fn ray_cast_uses_capsule_shape_instead_of_capsule_aabb() {
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
    level.with_world_mut(|world| {
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
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    let rounded_cap_corner_miss = physics.ray_cast(&PhysicsRayCastQuery {
        world: level.handle(),
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

    let rounded_cap_hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.5, 2.5, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray through the rounded capsule cap should hit");
    assert!((rounded_cap_hit.distance - 4.2928934).abs() < 1.0e-4);
    assert_eq!(rounded_cap_hit.position, [0.5, 2.5, -0.7071066]);
    assert!(
        Vec3::from_array(rounded_cap_hit.normal)
            .abs_diff_eq(Vec3::new(0.5, 0.5, -0.7071066), 1.0e-4),
        "unexpected normal: {:?}",
        rounded_cap_hit.normal
    );
}

#[test]
fn ray_cast_reports_capsule_exit_hit_when_origin_starts_inside_axis() {
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
    level.with_world_mut(|world| {
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
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 1.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray starting inside capsule should report the exit surface");
    assert!((hit.distance - 3.0).abs() < 1.0e-4);
    assert_eq!(hit.position, [0.0, 3.0, 0.0]);
    assert_eq!(hit.normal, [0.0, 1.0, 0.0]);
}

#[test]
fn ray_cast_uses_absolute_max_sphere_scale() {
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
    level.with_world_mut(|world| {
        let sphere = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                sphere,
                Transform::identity().with_scale(Vec3::new(-3.0, -1.0, -1.0)),
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
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [5.0, 0.0, 0.0],
            direction: [-1.0, 0.0, 0.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray should hit scaled sphere");
    assert!(
        (hit.distance - 2.0).abs() < 1.0e-4,
        "unexpected hit distance: {}",
        hit.distance
    );
    assert_eq!(hit.position, [3.0, 0.0, 0.0]);
    assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
}

#[test]
fn ray_cast_uses_scaled_collider_local_transform() {
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
    level.with_world_mut(|world| {
        let owner = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                owner,
                Transform::identity().with_scale(Vec3::new(2.0, 1.0, 1.0)),
            )
            .unwrap();
        world
            .set_collider(
                owner,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.25 },
                    local_transform: Transform::from_translation(Vec3::X),
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [2.0, 0.0, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("ray should hit collider after parent scale moves local offset");
    assert!(
        (hit.distance - 4.5).abs() < 1.0e-4,
        "unexpected hit distance: {}",
        hit.distance
    );
    assert_eq!(hit.position, [2.0, 0.0, -0.5]);
    assert_eq!(hit.normal, [0.0, 0.0, -1.0]);
}

#[test]
fn ray_cast_rejects_non_finite_query_input() {
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
    level.with_world_mut(|world| {
        let target = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                target,
                Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            )
            .unwrap();
        world
            .set_collider(
                target,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: f32::INFINITY,
            collision_mask: None,
            include_sensors: true,
        })
        .is_none());
}

#[test]
fn ray_cast_skips_non_finite_collider_transform() {
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
    let valid = level.with_world_mut(|world| {
        let invalid = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                invalid,
                Transform::from_translation(Vec3::new(f32::NAN, 0.0, 1.0)),
            )
            .unwrap();
        world
            .set_collider(
                invalid,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 10.0 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let valid = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(valid, Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)))
            .unwrap();
        world
            .set_collider(
                valid,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        valid
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("valid collider should still be hit after skipping invalid collider");
    assert_eq!(hit.entity, valid);
    assert!((hit.distance - 2.5).abs() < 1.0e-4);
}

#[test]
fn ray_cast_skips_negative_sphere_radius() {
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
    level.with_world_mut(|world| {
        let sphere = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                sphere,
                Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            )
            .unwrap();
        world
            .set_collider(
                sphere,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: -1.0 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .is_none());
}

#[test]
fn ray_cast_skips_non_finite_box_half_extents() {
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
    level.with_world_mut(|world| {
        let box_collider = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                box_collider,
                Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            )
            .unwrap();
        world
            .set_collider(
                box_collider,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::new(f32::NAN, 1.0, 1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .is_none());
}

#[test]
fn ray_cast_skips_non_finite_scaled_box_half_extents() {
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
    level.with_world_mut(|world| {
        let box_collider = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                box_collider,
                Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)).with_scale(Vec3::new(
                    f32::INFINITY,
                    1.0,
                    1.0,
                )),
            )
            .unwrap();
        world
            .set_collider(
                box_collider,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::new(1.0, 1.0, 1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    let physics = resolve_physics_manager(&runtime.handle()).unwrap();

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .is_none());
}
