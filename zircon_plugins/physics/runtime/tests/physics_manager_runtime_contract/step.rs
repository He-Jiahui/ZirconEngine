use super::*;

#[test]
fn integrate_builtin_physics_steps_ignores_non_finite_step_seconds() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::X,
                    angular_velocity: Vec3::Y,
                    gravity_scale: 1.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        integrate_builtin_physics_steps(
            world,
            PhysicsWorldStepPlan {
                steps: 1,
                step_seconds: f32::NAN,
                remaining_seconds: 0.0,
                interpolation_alpha: 0.0,
            },
        );
        body
    });

    let (transform, rigid_body) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform,
            world.rigid_body(body).unwrap().clone(),
        )
    });
    assert_eq!(transform.translation, Vec3::ZERO);
    assert_eq!(transform.rotation, Transform::identity().rotation);
    assert_eq!(rigid_body.linear_velocity, Vec3::X);
    assert_eq!(rigid_body.angular_velocity, Vec3::Y);
}

#[test]
fn integrate_builtin_physics_steps_ignores_non_finite_body_velocity() {
    let runtime = create_runtime_with_scene_and_physics();
    let level = create_default_level(&runtime.handle()).unwrap();
    let body = level.with_world_mut(|world| {
        let body = world.spawn_node(NodeKind::Cube);
        world
            .set_rigid_body(
                body,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    linear_velocity: Vec3::new(f32::NAN, 1.0, 0.0),
                    angular_velocity: Vec3::new(0.0, f32::INFINITY, 0.0),
                    gravity_scale: 0.0,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
        integrate_builtin_physics_steps(
            world,
            PhysicsWorldStepPlan {
                steps: 1,
                step_seconds: 1.0 / 60.0,
                remaining_seconds: 0.0,
                interpolation_alpha: 0.0,
            },
        );
        body
    });

    let (transform, rigid_body) = level.with_world(|world| {
        (
            world.find_node(body).unwrap().transform,
            world.rigid_body(body).unwrap().clone(),
        )
    });
    assert_eq!(transform.translation, Vec3::ZERO);
    assert_eq!(transform.rotation, Transform::identity().rotation);
    assert!(rigid_body.linear_velocity.x.is_nan());
    assert_eq!(rigid_body.linear_velocity.y, 1.0);
    assert_eq!(rigid_body.linear_velocity.z, 0.0);
    assert_eq!(rigid_body.angular_velocity.x, 0.0);
    assert_eq!(rigid_body.angular_velocity.y, f32::INFINITY);
    assert_eq!(rigid_body.angular_velocity.z, 0.0);
}

#[test]
fn unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "jolt".to_string(),
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

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let transform = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(level.last_physics_step_plan().unwrap().steps, 0);
    assert_eq!(transform.translation, Vec3::ZERO);
    assert!(level.physics_contacts().is_empty());
    assert!(resolve_physics_manager(&runtime.handle())
        .unwrap()
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .is_none());
}

#[test]
fn builtin_query_only_syncs_queries_without_fixed_step_writeback() {
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
                    shape: ColliderShape::Sphere { radius: 1.0 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        body
    });

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let transform = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(level.last_physics_step_plan().unwrap().steps, 0);
    assert_eq!(transform.translation, Vec3::ZERO);
    let hit = resolve_physics_manager(&runtime.handle())
        .unwrap()
        .ray_cast(&PhysicsRayCastQuery {
            world: level.handle(),
            origin: [0.0, 0.0, -5.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: true,
        })
        .expect("query-only builtin backend should still expose synced ray queries");
    assert_eq!(hit.entity, body);
    assert!((hit.distance - 4.0).abs() < 1.0e-4);
}

#[test]
fn builtin_fixed_step_uses_live_world_records_before_node_cache_flush() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 4,
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

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    let transform = level.with_world(|world| world.find_node(body).unwrap().transform);
    assert_eq!(level.last_physics_step_plan().unwrap().steps, 1);
    assert!(transform.translation.x > 0.0);
    assert_eq!(level.physics_contacts().len(), 1);
}

#[test]
fn fixed_step_plan_reports_zero_one_many_substeps_and_interpolation_alpha() {
    let runtime = create_runtime_with_scene_and_physics();
    let physics = runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    physics
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 2,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let world = zircon_runtime::core::framework::scene::WorldHandle::new(99);

    let zero = physics.plan_world_step(world, 0.25 / 60.0);
    assert_eq!(zero.steps, 0);
    assert!((zero.remaining_seconds - 0.25 / 60.0).abs() < 1.0e-6);
    assert!((zero.interpolation_alpha - 0.25).abs() < 1.0e-6);

    let one = physics.plan_world_step(world, 0.75 / 60.0);
    assert_eq!(one.steps, 1);
    assert_eq!(one.remaining_seconds, 0.0);
    assert_eq!(one.interpolation_alpha, 0.0);

    let many = physics.plan_world_step(world, 2.5 / 60.0);
    assert_eq!(many.steps, 2);
    assert!((many.remaining_seconds - 0.5 / 60.0).abs() < 1.0e-6);
    assert!((many.interpolation_alpha - 0.5).abs() < 1.0e-6);
}

#[test]
fn fixed_step_plan_disables_alpha_when_backend_cannot_step() {
    let runtime = create_runtime_with_scene_and_physics();
    let physics = runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    physics
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            fixed_hz: 60,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let plan = physics.plan_world_step(
        zircon_runtime::core::framework::scene::WorldHandle::new(100),
        1.0 / 60.0,
    );

    assert_eq!(plan.steps, 0);
    assert_eq!(plan.remaining_seconds, 0.0);
    assert_eq!(plan.interpolation_alpha, 0.0);
}
