use super::*;

#[test]
fn ray_cast_uses_capsule_shape_instead_of_capsule_aabb() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let rounded_cap_corner_miss = physics.ray_cast(&PhysicsRayCastQuery {
        world: level.handle(),
        origin: [0.9, 2.9, -5.0],
        direction: [0.0, 0.0, 1.0],
        max_distance: 10.0,
        mode: Default::default(),
        filter: PhysicsQueryFilter {
            include_sensors: true,
            ..PhysicsQueryFilter::default()
        },
    });
    assert!(
        rounded_cap_corner_miss.is_empty(),
        "ray lies inside the capsule AABB but outside the rounded capsule cap"
    );

    let rounded_cap_hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.5, 2.5, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
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
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 1.0, 0.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
        .expect("ray starting inside capsule should report the exit surface");
    assert!((hit.distance - 3.0).abs() < 1.0e-4);
    assert_eq!(hit.position, [0.0, 3.0, 0.0]);
    assert_eq!(hit.normal, [0.0, 1.0, 0.0]);
}

#[test]
fn ray_cast_uses_absolute_max_sphere_scale() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [5.0, 0.0, 0.0],
            direction: [-1.0, 0.0, 0.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
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
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [2.0, 0.0, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
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
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: f32::INFINITY,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .is_empty());
}

#[test]
fn ray_cast_skips_non_finite_collider_transform() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hit = physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
        .expect("valid collider should still be hit after skipping invalid collider");
    assert_eq!(hit.entity, valid);
    assert!((hit.distance - 2.5).abs() < 1.0e-4);
}

#[test]
fn ray_cast_skips_negative_sphere_radius() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .is_empty());
}

#[test]
fn ray_cast_skips_non_finite_box_half_extents() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .is_empty());
}

#[test]
fn ray_cast_skips_non_finite_scaled_box_half_extents() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    assert!(physics
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .is_empty());
}

#[test]
fn shape_overlap_uses_shared_query_filter_for_layers_sensors_groups_and_exclusions() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let (included, excluded) = level.with_world_mut(|world| {
        let included = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                included,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    layer: 1,
                    collision_group: 7,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let excluded = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                excluded,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    layer: 1,
                    collision_group: 7,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let sensor = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                sensor,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    sensor: true,
                    layer: 1,
                    collision_group: 7,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let wrong_group = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                wrong_group,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    layer: 1,
                    collision_group: 9,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        (included, excluded)
    });
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hits = physics.shape_overlap(&PhysicsShapeOverlapQuery {
        world: level.handle(),
        shape: PhysicsColliderShape::Sphere { radius: 1.0 },
        transform: Transform::identity(),
        mode: Default::default(),
        filter: PhysicsQueryFilter {
            collision_mask: Some(0b10),
            excluded_entities: vec![excluded],
            required_collision_group: Some(7),
            ..PhysicsQueryFilter::default()
        },
    });

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entity, included);
    assert_eq!(hits[0].layer, 1);
    assert_eq!(hits[0].collision_group, 7);
}

#[test]
fn shape_overlap_rejects_non_finite_query_rotation() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level.with_world_mut(|world| {
        let target = world.spawn_node(NodeKind::Cube);
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
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let hits = physics.shape_overlap(&PhysicsShapeOverlapQuery {
        world: level.handle(),
        shape: PhysicsColliderShape::Sphere { radius: 1.0 },
        transform: Transform::identity().with_rotation(Quat::from_array([f32::NAN, 0.0, 0.0, 1.0])),
        mode: Default::default(),
        filter: PhysicsQueryFilter {
            include_sensors: true,
            ..PhysicsQueryFilter::default()
        },
    });

    assert!(
        hits.is_empty(),
        "invalid query rotation must not produce fallback overlap hits"
    );
}

#[test]
fn builtin_shape_cast_reports_initial_overlap_and_swept_hit() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let target = level.with_world_mut(|world| {
        let target = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                target,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        target
    });
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let initial_hit = physics
        .shape_cast(&PhysicsShapeCastQuery {
            world: level.handle(),
            shape: PhysicsColliderShape::Sphere { radius: 1.0 },
            origin_transform: Transform::identity(),
            direction: [1.0, 0.0, 0.0],
            max_distance: 8.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
        .expect("builtin fallback should report initial overlap");
    assert_eq!(initial_hit.entity, target);
    assert_eq!(initial_hit.distance, 0.0);

    let swept_hit = physics
        .shape_cast(&PhysicsShapeCastQuery {
            world: level.handle(),
            shape: PhysicsColliderShape::Sphere { radius: 1.0 },
            origin_transform: Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)),
            direction: [1.0, 0.0, 0.0],
            max_distance: 8.0,
            mode: Default::default(),
            filter: PhysicsQueryFilter {
                include_sensors: true,
                ..PhysicsQueryFilter::default()
            },
        })
        .into_iter()
        .next()
        .expect("builtin sweep should hit the target");
    assert_eq!(swept_hit.entity, target);
    assert!((swept_hit.distance - 2.5).abs() <= f32::EPSILON);
    assert_eq!(swept_hit.normal, [-1.0, 0.0, 0.0]);
}

#[test]
fn query_all_returns_distance_sorted_hits() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let (far, near) = level.with_world_mut(|world| {
        let far = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(far, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
            .unwrap();
        world
            .set_collider(
                far,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let near = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                near,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let ignored_sensor = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                ignored_sensor,
                Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                ignored_sensor,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    sensor: true,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        (far, near)
    });
    tick_physics_level(&runtime, &level);
    let physics = physics_manager(&runtime);

    let query = PhysicsShapeCastQuery {
        world: level.handle(),
        shape: PhysicsColliderShape::Sphere { radius: 0.5 },
        origin_transform: Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)),
        direction: [1.0, 0.0, 0.0],
        max_distance: 10.0,
        mode: PhysicsQueryMode::All,
        filter: PhysicsQueryFilter::default(),
    };

    let all = physics.shape_cast(&query);
    assert_eq!(all.len(), 2);
    assert_eq!([all[0].entity, all[1].entity], [near, far]);
    assert!((all[0].distance - 3.0).abs() < 1.0e-4);
    assert!((all[1].distance - 7.0).abs() < 1.0e-4);

    let first = physics.shape_cast(&PhysicsShapeCastQuery {
        mode: PhysicsQueryMode::First,
        ..query.clone()
    });
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].entity, far);

    let closest = physics.shape_cast(&PhysicsShapeCastQuery {
        mode: PhysicsQueryMode::Closest,
        ..query
    });
    assert_eq!(closest.len(), 1);
    assert_eq!(closest[0].entity, near);

    let ray_all = physics.ray_cast(&PhysicsRayCastQuery {
        world: level.handle(),
        origin: [-4.0, 0.0, 0.0],
        direction: [1.0, 0.0, 0.0],
        max_distance: 10.0,
        mode: PhysicsQueryMode::All,
        filter: PhysicsQueryFilter::default(),
    });
    assert_eq!(ray_all.len(), 2);
    assert_eq!([ray_all[0].entity, ray_all[1].entity], [near, far]);
    assert!(ray_all[0].distance <= ray_all[1].distance);

    let overlap_query = PhysicsShapeOverlapQuery {
        world: level.handle(),
        shape: PhysicsColliderShape::Sphere { radius: 10.0 },
        transform: Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)),
        mode: PhysicsQueryMode::All,
        filter: PhysicsQueryFilter::default(),
    };
    let overlap_all = physics.shape_overlap(&overlap_query);
    assert_eq!(overlap_all.len(), 2);
    assert_eq!([overlap_all[0].entity, overlap_all[1].entity], [near, far]);

    let overlap_first = physics.shape_overlap(&PhysicsShapeOverlapQuery {
        mode: PhysicsQueryMode::First,
        ..overlap_query
    });
    assert_eq!(overlap_first.len(), 1);
    assert_eq!(overlap_first[0].entity, far);
}
