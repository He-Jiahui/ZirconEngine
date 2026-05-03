use super::*;

#[test]
fn contact_skips_non_finite_collider_transform() {
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
        let invalid = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                invalid,
                Transform::identity().with_scale(Vec3::new(f32::INFINITY, 1.0, 1.0)),
            )
            .unwrap();
        world
            .set_collider(
                invalid,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let valid = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                valid,
                Transform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                valid,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "colliders with non-finite synced transforms must not emit fallback contacts"
    );
}

#[test]
fn contact_skips_negative_box_half_extents() {
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
        let malformed = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                malformed,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::new(-1.0, 1.0, 1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let valid = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                valid,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "box with negative half extents must not emit fallback contacts"
    );
}

#[test]
fn contact_skips_non_finite_sphere_radius() {
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
        let malformed = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                malformed,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere {
                        radius: f32::INFINITY,
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let valid = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                valid,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "sphere with non-finite radius must not emit fallback contacts"
    );
}

#[test]
fn contact_skips_non_finite_scaled_capsule_radius() {
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
        let malformed = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                malformed,
                Transform::identity().with_scale(Vec3::new(f32::INFINITY, 1.0, 1.0)),
            )
            .unwrap();
        world
            .set_collider(
                malformed,
                Some(ColliderComponent {
                    shape: ColliderShape::Capsule {
                        radius: 1.0,
                        half_height: 0.5,
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let valid = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                valid,
                Some(ColliderComponent {
                    shape: ColliderShape::Box {
                        half_extents: Vec3::splat(1.0),
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "capsule with non-finite scaled radius must not emit fallback contacts"
    );
}

#[test]
fn contact_point_stays_finite_for_large_overlapping_centers() {
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
        for _ in 0..2 {
            let sphere = world.spawn_node(NodeKind::Cube);
            world
                .update_transform(
                    sphere,
                    Transform::from_translation(Vec3::new(f32::MAX, f32::MAX, f32::MAX)),
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
        }
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    let contacts = level.physics_contacts();
    assert_eq!(contacts.len(), 1);
    assert!(
        contacts[0]
            .point
            .iter()
            .all(|component| component.is_finite()),
        "contact point should remain finite for finite overlapping centers: {:?}",
        contacts[0].point
    );
}

#[test]
fn contact_skips_negative_capsule_radius() {
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
                        radius: -1.0,
                        half_height: 0.5,
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();

        let sphere = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                sphere,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.25 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "capsule with negative radius must not emit fallback contacts"
    );
}

#[test]
fn contact_uses_capsule_shape_instead_of_capsule_aabb() {
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
    let sphere = level.with_world_mut(|world| {
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

        let sphere = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                sphere,
                Transform::from_translation(Vec3::new(0.9, 2.9, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                sphere,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.05 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        sphere
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "sphere is inside the capsule AABB but outside the rounded capsule cap"
    );

    level.with_world_mut(|world| {
        world
            .update_transform(
                sphere,
                Transform::from_translation(Vec3::new(0.5, 2.5, 0.0)),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    assert_eq!(level.physics_contacts().len(), 1);
}

#[test]
fn contact_uses_box_sphere_shape_instead_of_aabb_overlap() {
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
    let sphere = level.with_world_mut(|world| {
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

        let sphere = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                sphere,
                Transform::from_translation(Vec3::new(1.04, 1.04, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                sphere,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 0.05 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        sphere
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "sphere AABB overlaps the box corner, but the sphere does not touch the box"
    );

    level.with_world_mut(|world| {
        world
            .update_transform(
                sphere,
                Transform::from_translation(Vec3::new(1.03, 1.03, 0.0)),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    assert_eq!(level.physics_contacts().len(), 1);
}

#[test]
fn contact_uses_box_capsule_shape_instead_of_aabb_overlap() {
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
    let capsule = level.with_world_mut(|world| {
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

        let capsule = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                capsule,
                Transform::from_translation(Vec3::new(1.04, 0.0, 1.04)),
            )
            .unwrap();
        world
            .set_collider(
                capsule,
                Some(ColliderComponent {
                    shape: ColliderShape::Capsule {
                        radius: 0.05,
                        half_height: 0.5,
                    },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        capsule
    });

    level.tick(&runtime.handle(), 0.0).unwrap();
    assert!(
        level.physics_contacts().is_empty(),
        "capsule AABB overlaps the box corner, but the capsule does not touch the box"
    );

    level.with_world_mut(|world| {
        world
            .update_transform(
                capsule,
                Transform::from_translation(Vec3::new(1.03, 0.0, 1.03)),
            )
            .unwrap();
    });
    level.tick(&runtime.handle(), 0.0).unwrap();
    assert_eq!(level.physics_contacts().len(), 1);
}
