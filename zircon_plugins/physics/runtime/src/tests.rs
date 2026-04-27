use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsManager, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::manager::{resolve_config_manager, resolve_physics_manager};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::world::World;

#[test]
fn physics_plugin_registration_contributes_runtime_module() {
    let report = super::plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == super::PHYSICS_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn physics_manager_persists_settings_to_runtime_config() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_runtime::foundation::module_descriptor())
        .unwrap();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(super::PHYSICS_MODULE_NAME).unwrap();

    let manager = runtime
        .handle()
        .resolve_manager::<super::DefaultPhysicsManager>(super::DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    let settings = PhysicsSettings {
        backend: "jolt".to_string(),
        simulation_mode: PhysicsSimulationMode::Simulate,
        fixed_hz: 60,
        max_substeps: 3,
        layer_names: vec!["default".to_string(), "characters".to_string()],
        group_names: vec!["world".to_string(), "player".to_string()],
        collision_matrix: vec![0b11, 0b11],
        solver_groups: vec!["default".to_string()],
    };

    manager.store_settings(settings.clone()).unwrap();

    let facade = resolve_physics_manager(&runtime.handle()).unwrap();
    assert_eq!(facade.settings(), settings);

    let config = resolve_config_manager(&runtime.handle()).unwrap();
    assert!(config
        .get_value(super::PHYSICS_SETTINGS_CONFIG_KEY)
        .is_some());
}

#[test]
fn physics_manager_tracks_fixed_step_accumulator_per_world() {
    let runtime = CoreRuntime::new();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime.activate_module(super::PHYSICS_MODULE_NAME).unwrap();

    let manager = runtime
        .handle()
        .resolve_manager::<super::DefaultPhysicsManager>(super::DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 3,
            layer_names: vec!["default".to_string()],
            group_names: vec!["default".to_string()],
            collision_matrix: vec![0b1],
            solver_groups: vec!["default".to_string()],
        })
        .unwrap();

    let world = WorldHandle::new(7);
    let first = manager.advance_clock(world, 1.0 / 15.0);
    assert_eq!(first.steps, 3);
    assert!((first.step_seconds - (1.0 / 60.0)).abs() < 1.0e-6);
    assert!((first.remaining_seconds - (1.0 / 60.0)).abs() < 1.0e-6);

    let second = manager.advance_clock(world, 1.0 / 60.0);
    assert_eq!(second.steps, 2);
    assert!(second.remaining_seconds.abs() < 1.0e-6);
}

#[test]
fn physics_manager_ignores_non_finite_delta_seconds() {
    let runtime = CoreRuntime::new();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime.activate_module(super::PHYSICS_MODULE_NAME).unwrap();

    let manager = runtime
        .handle()
        .resolve_manager::<super::DefaultPhysicsManager>(super::DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 3,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let world = WorldHandle::new(70);
    let infinite = manager.advance_clock(world, f32::INFINITY);
    assert_eq!(infinite.steps, 0);
    assert_eq!(infinite.remaining_seconds, 0.0);

    let nan = manager.advance_clock(world, f32::NAN);
    assert_eq!(nan.steps, 0);
    assert_eq!(nan.remaining_seconds, 0.0);

    let next = manager.advance_clock(world, 1.0 / 60.0);
    assert_eq!(next.steps, 1);
    assert_eq!(next.remaining_seconds, 0.0);
}

#[test]
fn physics_manager_reports_jolt_unavailable_downgrade_without_panicking() {
    let manager = super::DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "jolt".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let status = manager.backend_status();

    if super::JOLT_ENABLED {
        assert_eq!(status.state, PhysicsBackendState::Ready);
        assert_eq!(status.active_backend.as_deref(), Some("jolt"));
        assert!(status.detail.is_none());
    } else {
        assert_eq!(status.state, PhysicsBackendState::Unavailable);
        assert!(status.active_backend.is_none());
        assert_eq!(status.feature_gate.as_deref(), Some("jolt"));
        assert!(
            status
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("feature `jolt`")),
            "unexpected detail: {:?}",
            status.detail
        );
    }
}

#[test]
fn physics_manager_syncs_world_snapshot_and_exposes_queries_and_contacts() {
    let manager = super::DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "contract_snapshot".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let mut world = World::new();
    let first = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(first, Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)))
        .unwrap();
    world
        .set_rigid_body(
            first,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Static,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    world
        .set_collider(
            first,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(0.5),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let second = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            second,
            Transform::from_translation(Vec3::new(0.5, 0.0, 4.0)),
        )
        .unwrap();
    world
        .set_rigid_body(
            second,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    world
        .set_collider(
            second,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(0.5),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let world_handle = WorldHandle::new(99);
    let sync = super::build_world_sync_state(world_handle, &world);
    manager.sync_world(sync.clone());

    assert_eq!(manager.synchronized_world(world_handle), Some(sync));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: None,
            include_sensors: false,
        })
        .expect("expected ray cast hit");
    assert_eq!(hit.entity, first);
    assert!(
        (hit.distance - 3.5).abs() < 1.0e-3,
        "unexpected distance: {}",
        hit.distance
    );

    let contacts = manager.drain_contacts(world_handle);
    assert_eq!(contacts.len(), 1);
    let contact = &contacts[0];
    assert_eq!((contact.entity, contact.other_entity), (first, second));
    assert!(manager.drain_contacts(world_handle).is_empty());
}

#[test]
fn physics_manager_ray_cast_reports_exit_hit_when_origin_starts_inside_sphere() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

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
fn physics_manager_ray_cast_reports_exit_hit_when_origin_starts_inside_box() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

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
fn physics_manager_ray_cast_reports_surface_normal_when_starting_on_box_surface() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

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
fn physics_manager_ray_cast_uses_capsule_shape_instead_of_capsule_aabb() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

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

#[test]
fn physics_manager_contacts_respect_collider_collision_masks() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "overlapping colliders with masks that exclude each other's layers must not emit contacts"
    );
}

#[test]
fn physics_manager_ray_cast_query_mask_filters_by_collider_layer_not_collider_mask() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

    let hit = manager
        .ray_cast(&PhysicsRayCastQuery {
            world: world_handle,
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            max_distance: 10.0,
            collision_mask: Some(1 << 2),
            include_sensors: true,
        })
        .expect("query mask should select the collider's layer even when collider contact mask is empty");

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
fn physics_manager_contacts_respect_project_collision_matrix() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

    assert!(
        manager.drain_contacts(world_handle).is_empty(),
        "project collision matrix must reject contacts even when collider masks allow each other"
    );
}

#[test]
fn physics_manager_contacts_skip_sensor_colliders_but_queries_can_include_them() {
    let manager = super::DefaultPhysicsManager::default();
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
    manager.sync_world(super::build_world_sync_state(world_handle, &world));

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
