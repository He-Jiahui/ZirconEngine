use crate::core::framework::physics::{
    PhysicsBackendState, PhysicsManager, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::manager::{resolve_config_manager, resolve_physics_manager};
use crate::core::math::{Transform, Vec3};
use crate::core::CoreRuntime;
use crate::foundation::FOUNDATION_MODULE_NAME;
use crate::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

#[test]
fn physics_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("physics")
            .join("mod.rs"),
    )
    .expect("physics mod source");

    for forbidden in [
        "pub struct PhysicsConfig",
        "pub struct PhysicsModule",
        "pub fn module_descriptor(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected physics/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn physics_manager_persists_settings_to_runtime_config() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::foundation::module_descriptor())
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
