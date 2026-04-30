use crate::DefaultPhysicsManager;
use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
    PhysicsJointSyncState, PhysicsJointType, PhysicsManager, PhysicsMaterialMetadata,
    PhysicsMaterialSyncState, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};

#[test]
fn physics_manager_sanitizes_external_world_sync_state() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(120);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: vec![
            body_sync_state(1, Transform::identity()),
            body_sync_state(
                2,
                Transform::from_translation(Vec3::new(f32::NAN, 0.0, 0.0)),
            ),
        ],
        colliders: vec![
            collider_sync_state_with_material_override(
                1,
                PhysicsColliderShape::Sphere { radius: 1.0 },
                PhysicsMaterialMetadata::default(),
            ),
            collider_sync_state(
                3,
                PhysicsColliderShape::Sphere {
                    radius: f32::INFINITY,
                },
            ),
        ],
        joints: vec![
            joint_sync_state(1, [0.0, 1.0, 0.0], Some([0.0, 1.0])),
            joint_sync_state(4, [f32::NAN, 1.0, 0.0], Some([0.0, 1.0])),
        ],
        materials: vec![
            material_sync_state(1, PhysicsMaterialMetadata::default()),
            material_sync_state(
                5,
                PhysicsMaterialMetadata {
                    restitution: f32::INFINITY,
                    ..PhysicsMaterialMetadata::default()
                },
            ),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, 1);
    assert_eq!(sync.colliders.len(), 1);
    assert_eq!(sync.colliders[0].entity, 1);
    assert_eq!(sync.joints.len(), 1);
    assert_eq!(sync.joints[0].entity, 1);
    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 1);
}

#[test]
fn physics_manager_drops_orphan_material_sync_entries() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(121);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![
            collider_sync_state_with_material_override(
                1,
                PhysicsColliderShape::Sphere { radius: 1.0 },
                PhysicsMaterialMetadata::default(),
            ),
            collider_sync_state(2, PhysicsColliderShape::Sphere { radius: f32::NAN }),
        ],
        joints: Vec::new(),
        materials: vec![
            material_sync_state(1, PhysicsMaterialMetadata::default()),
            material_sync_state(2, PhysicsMaterialMetadata::default()),
            material_sync_state(3, PhysicsMaterialMetadata::default()),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.colliders.len(), 1);
    assert_eq!(sync.colliders[0].entity, 1);
    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 1);
}

#[test]
fn physics_manager_drops_material_sync_entries_without_collider_material_binding() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(122);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![
            collider_sync_state(1, PhysicsColliderShape::Sphere { radius: 1.0 }),
            collider_sync_state_with_material_override(
                2,
                PhysicsColliderShape::Sphere { radius: 1.0 },
                PhysicsMaterialMetadata::default(),
            ),
        ],
        joints: Vec::new(),
        materials: vec![
            material_sync_state(1, PhysicsMaterialMetadata::default()),
            material_sync_state(2, PhysicsMaterialMetadata::default()),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 2);
}

#[test]
fn physics_manager_drops_material_sync_entries_with_mismatched_locator() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(123);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![collider_sync_state_with_material_locator(
            1,
            PhysicsColliderShape::Sphere { radius: 1.0 },
            "wood.physics_material",
        )],
        joints: Vec::new(),
        materials: vec![
            material_sync_state_with_locator(
                1,
                "metal.physics_material",
                PhysicsMaterialMetadata::default(),
            ),
            material_sync_state_with_locator(
                1,
                "wood.physics_material",
                PhysicsMaterialMetadata::default(),
            ),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 1);
    assert_eq!(
        sync.materials[0].locator.as_deref(),
        Some("wood.physics_material")
    );
}

#[test]
fn physics_manager_deduplicates_material_sync_entries_per_collider() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(124);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![collider_sync_state_with_material_locator(
            1,
            PhysicsColliderShape::Sphere { radius: 1.0 },
            "wood.physics_material",
        )],
        joints: Vec::new(),
        materials: vec![
            material_sync_state_with_locator(
                1,
                "wood.physics_material",
                PhysicsMaterialMetadata {
                    static_friction: 0.25,
                    ..PhysicsMaterialMetadata::default()
                },
            ),
            material_sync_state_with_locator(
                1,
                "wood.physics_material",
                PhysicsMaterialMetadata {
                    static_friction: 0.75,
                    ..PhysicsMaterialMetadata::default()
                },
            ),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 1);
    assert_eq!(sync.materials[0].material.static_friction, 0.25);
}

#[test]
fn physics_manager_deduplicates_external_component_sync_entries_per_entity() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(125);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: vec![
            body_sync_state_with_mass(1, 1.0),
            body_sync_state_with_mass(1, 2.0),
        ],
        colliders: vec![
            collider_sync_state_with_layer(1, PhysicsColliderShape::Sphere { radius: 1.0 }, 3),
            collider_sync_state_with_layer(1, PhysicsColliderShape::Sphere { radius: 2.0 }, 7),
        ],
        joints: vec![
            joint_sync_state(1, [0.0, 1.0, 0.0], Some([0.0, 1.0])),
            joint_sync_state(1, [1.0, 0.0, 0.0], Some([0.0, 2.0])),
        ],
        materials: Vec::new(),
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, 1);
    assert_eq!(sync.bodies[0].mass, 1.0);
    assert_eq!(sync.colliders.len(), 1);
    assert_eq!(sync.colliders[0].entity, 1);
    assert_eq!(sync.colliders[0].layer, 3);
    assert_eq!(sync.joints.len(), 1);
    assert_eq!(sync.joints[0].entity, 1);
    assert_eq!(sync.joints[0].axis, [0.0, 1.0, 0.0]);
}

#[test]
fn physics_manager_drops_external_colliders_with_invalid_layer() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(126);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![
            collider_sync_state_with_layer(1, PhysicsColliderShape::Sphere { radius: 1.0 }, 32),
            collider_sync_state_with_layer(2, PhysicsColliderShape::Sphere { radius: 1.0 }, 31),
        ],
        joints: Vec::new(),
        materials: Vec::new(),
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.colliders.len(), 1);
    assert_eq!(sync.colliders[0].entity, 2);
    assert_eq!(sync.colliders[0].layer, 31);
}

#[test]
fn physics_manager_drops_external_body_sync_entries_with_non_positive_mass() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(127);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: vec![
            body_sync_state_with_mass(1, 0.0),
            body_sync_state_with_mass(2, -1.0),
            body_sync_state_with_mass(3, 1.0),
        ],
        colliders: Vec::new(),
        joints: Vec::new(),
        materials: Vec::new(),
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, 3);
    assert_eq!(sync.bodies[0].mass, 1.0);
}

#[test]
fn physics_manager_drops_material_sync_entries_with_empty_locator() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(128);

    manager.sync_world(PhysicsWorldSyncState {
        world,
        bodies: Vec::new(),
        colliders: vec![
            collider_sync_state_with_material_locator(
                1,
                PhysicsColliderShape::Sphere { radius: 1.0 },
                "   ",
            ),
            collider_sync_state_with_material_locator(
                2,
                PhysicsColliderShape::Sphere { radius: 1.0 },
                "wood.physics_material",
            ),
        ],
        joints: Vec::new(),
        materials: vec![
            material_sync_state_with_locator(1, "   ", PhysicsMaterialMetadata::default()),
            material_sync_state_with_locator(
                2,
                "wood.physics_material",
                PhysicsMaterialMetadata::default(),
            ),
        ],
    });

    let sync = manager
        .synchronized_world(world)
        .expect("manager should store sanitized sync state");

    assert_eq!(sync.colliders.len(), 1);
    assert_eq!(sync.colliders[0].entity, 2);
    assert_eq!(sync.materials.len(), 1);
    assert_eq!(sync.materials[0].entity, 2);
    assert_eq!(
        sync.materials[0].locator.as_deref(),
        Some("wood.physics_material")
    );
}

fn body_sync_state(entity: u64, transform: Transform) -> PhysicsBodySyncState {
    PhysicsBodySyncState {
        entity,
        body_type: PhysicsBodyType::Dynamic,
        transform,
        mass: 1.0,
        linear_velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        can_sleep: true,
        lock_translation: [false; 3],
        lock_rotation: [false; 3],
    }
}

fn body_sync_state_with_mass(entity: u64, mass: f32) -> PhysicsBodySyncState {
    PhysicsBodySyncState {
        mass,
        ..body_sync_state(entity, Transform::identity())
    }
}

fn collider_sync_state(entity: u64, shape: PhysicsColliderShape) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        entity,
        shape,
        sensor: false,
        layer: 0,
        collision_group: 0,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform: Transform::identity(),
    }
}

fn collider_sync_state_with_layer(
    entity: u64,
    shape: PhysicsColliderShape,
    layer: u32,
) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        layer,
        ..collider_sync_state(entity, shape)
    }
}

fn collider_sync_state_with_material_override(
    entity: u64,
    shape: PhysicsColliderShape,
    material_override: PhysicsMaterialMetadata,
) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        material_override: Some(material_override),
        ..collider_sync_state(entity, shape)
    }
}

fn collider_sync_state_with_material_locator(
    entity: u64,
    shape: PhysicsColliderShape,
    material_locator: &str,
) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        material: Some(material_locator.to_string()),
        ..collider_sync_state(entity, shape)
    }
}

fn joint_sync_state(
    entity: u64,
    axis: [f32; 3],
    limits: Option<[f32; 2]>,
) -> PhysicsJointSyncState {
    PhysicsJointSyncState {
        entity,
        kind: PhysicsJointType::Hinge,
        connected_entity: None,
        anchor: [0.0, 0.0, 0.0],
        axis,
        limits,
        collide_connected: false,
    }
}

fn material_sync_state(entity: u64, material: PhysicsMaterialMetadata) -> PhysicsMaterialSyncState {
    PhysicsMaterialSyncState {
        entity,
        locator: None,
        material,
    }
}

fn material_sync_state_with_locator(
    entity: u64,
    locator: &str,
    material: PhysicsMaterialMetadata,
) -> PhysicsMaterialSyncState {
    PhysicsMaterialSyncState {
        entity,
        locator: Some(locator.to_string()),
        material,
    }
}
