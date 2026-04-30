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

mod contact;
mod integration;
mod ray;
mod sync;

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

    let resolved_manager = resolve_physics_manager(&runtime.handle()).unwrap();
    assert_eq!(resolved_manager.settings(), settings);

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
