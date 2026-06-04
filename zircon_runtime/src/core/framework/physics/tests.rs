use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Transform, Vec3};

use super::*;

#[test]
fn default_physics_settings_match_disabled_contract() {
    let settings = PhysicsSettings::default();

    assert_eq!(settings.backend, "unconfigured");
    assert_eq!(settings.simulation_mode, PhysicsSimulationMode::Disabled);
    assert_eq!(settings.fixed_hz, 60);
    assert_eq!(settings.max_substeps, 4);
    assert_eq!(settings.layer_names, vec!["default".to_string()]);
    assert_eq!(settings.group_names, vec!["default".to_string()]);
    assert_eq!(settings.collision_matrix, vec![0b1]);
    assert_eq!(settings.solver_groups, vec!["default".to_string()]);

    let json = serde_json::to_value(&settings).unwrap();
    assert_eq!(json["simulation_mode"], "disabled");
    assert_eq!(
        serde_json::from_value::<PhysicsSettings>(json).unwrap(),
        settings
    );
}

#[test]
fn collider_body_and_joint_contracts_use_snake_case_serde() {
    let shape = PhysicsColliderShape::Capsule {
        radius: 0.5,
        half_height: 1.25,
    };
    let shape_json = serde_json::to_value(&shape).unwrap();

    assert_eq!(shape_json["kind"], "capsule");
    assert_eq!(shape_json["radius"], 0.5);
    assert_eq!(shape_json["half_height"], 1.25);
    assert_eq!(
        serde_json::from_value::<PhysicsColliderShape>(shape_json).unwrap(),
        shape
    );
    assert_eq!(
        serde_json::to_value(PhysicsBodyType::Kinematic).unwrap(),
        "kinematic"
    );
    assert_eq!(
        serde_json::to_value(PhysicsJointType::Distance).unwrap(),
        "distance"
    );
    assert_eq!(
        serde_json::to_value(PhysicsJointType::Generic6Dof).unwrap(),
        "generic6_dof"
    );
    assert_eq!(
        serde_json::to_value(PhysicsCombineRule::Multiply).unwrap(),
        "multiply"
    );
}

#[test]
fn world_sync_default_starts_empty_and_backend_neutral() {
    let sync = PhysicsWorldSyncState::default();

    assert_eq!(sync.world, WorldHandle::new(0));
    assert!(sync.bodies.is_empty());
    assert!(sync.colliders.is_empty());
    assert!(sync.joints.is_empty());
    assert!(sync.materials.is_empty());
}

#[test]
fn backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos() {
    let status = PhysicsBackendStatus {
        requested_backend: "builtin".to_string(),
        active_backend: Some("builtin".to_string()),
        state: PhysicsBackendState::Ready,
        detail: None,
        simulation_mode: PhysicsSimulationMode::Simulate,
        feature_gate: None,
    };
    let step_plan = PhysicsWorldStepPlan {
        steps: 2,
        step_seconds: 1.0 / 60.0,
        remaining_seconds: 0.002,
        interpolation_alpha: 0.12,
    };
    let query = PhysicsRayCastQuery {
        world: WorldHandle::new(7),
        origin: [0.0, 1.0, 0.0],
        direction: [0.0, -1.0, 0.0],
        max_distance: 32.0,
        filter: PhysicsQueryFilter {
            collision_mask: Some(0b101),
            include_sensors: true,
            ..PhysicsQueryFilter::default()
        },
    };
    let overlap_query = PhysicsShapeOverlapQuery {
        world: WorldHandle::new(7),
        shape: PhysicsColliderShape::Sphere { radius: 0.75 },
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        filter: PhysicsQueryFilter {
            excluded_entities: vec![99],
            required_collision_group: Some(3),
            ..PhysicsQueryFilter::default()
        },
    };
    let shape_cast_query = PhysicsShapeCastQuery {
        world: WorldHandle::new(7),
        shape: PhysicsColliderShape::Box {
            half_extents: [0.5, 0.5, 0.5],
        },
        origin_transform: Transform::default(),
        direction: [1.0, 0.0, 0.0],
        max_distance: 8.0,
        filter: PhysicsQueryFilter::default(),
    };
    let overlap_hit = PhysicsShapeOverlapHit {
        entity: 42,
        shape: PhysicsColliderShape::Capsule {
            radius: 0.25,
            half_height: 1.0,
        },
        transform: Transform::default(),
        sensor: true,
        layer: 1,
        collision_group: 3,
    };
    let shape_cast_hit = PhysicsShapeCastHit {
        entity: 42,
        distance: 0.5,
        position: [0.5, 1.0, 0.0],
        normal: [-1.0, 0.0, 0.0],
    };
    let joint = PhysicsJointSyncState {
        entity: 77,
        kind: PhysicsJointType::Generic6Dof,
        connected_entity: Some(42),
        anchor: [0.0, 1.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        limits: Some([-0.25, 0.25]),
        collide_connected: false,
        constraint: PhysicsJointConstraintMetadata {
            linear_limits: [Some([-0.2, 0.2]), None, Some([0.0, 1.0])],
            angular_limits: [Some([-0.5, 0.5]), Some([-0.25, 0.25]), None],
            linear_drives: [
                PhysicsJointDrive {
                    target_position: 0.1,
                    stiffness: 12.0,
                    damping: 2.0,
                    max_force: 30.0,
                    ..PhysicsJointDrive::default()
                },
                PhysicsJointDrive::default(),
                PhysicsJointDrive::default(),
            ],
            angular_drives: [
                PhysicsJointDrive::default(),
                PhysicsJointDrive {
                    target_velocity: 1.5,
                    stiffness: 4.0,
                    damping: 0.5,
                    max_force: 8.0,
                    ..PhysicsJointDrive::default()
                },
                PhysicsJointDrive::default(),
            ],
            break_force: Some(120.0),
            break_torque: Some(40.0),
            projection_linear_tolerance: Some(0.01),
            projection_angular_tolerance: Some(0.02),
        },
        skeleton_binding: Some(PhysicsSkeletonJointBinding {
            skeleton_entity: 7,
            bone_path: "Armature/Hips/Spine".to_string(),
            parent_bone_path: Some("Armature/Hips".to_string()),
        }),
    };
    let trigger = PhysicsTriggerEvent {
        world: WorldHandle::new(7),
        kind: PhysicsTriggerEventKind::Enter,
        trigger_entity: 24,
        other_entity: 42,
        point: [0.25, 1.0, 0.0],
    };

    assert_eq!(
        serde_json::from_value::<PhysicsBackendStatus>(serde_json::to_value(&status).unwrap())
            .unwrap(),
        status
    );
    assert_eq!(
        serde_json::from_value::<PhysicsWorldStepPlan>(serde_json::to_value(&step_plan).unwrap())
            .unwrap(),
        step_plan
    );
    assert_eq!(
        serde_json::from_value::<PhysicsRayCastQuery>(serde_json::to_value(&query).unwrap())
            .unwrap(),
        query
    );
    assert_eq!(
        serde_json::from_value::<PhysicsShapeOverlapQuery>(
            serde_json::to_value(&overlap_query).unwrap()
        )
        .unwrap(),
        overlap_query
    );
    assert_eq!(
        serde_json::from_value::<PhysicsShapeCastQuery>(
            serde_json::to_value(&shape_cast_query).unwrap()
        )
        .unwrap(),
        shape_cast_query
    );
    assert_eq!(
        serde_json::from_value::<PhysicsShapeOverlapHit>(
            serde_json::to_value(&overlap_hit).unwrap()
        )
        .unwrap(),
        overlap_hit
    );
    assert_eq!(
        serde_json::from_value::<PhysicsShapeCastHit>(
            serde_json::to_value(&shape_cast_hit).unwrap()
        )
        .unwrap(),
        shape_cast_hit
    );
    assert_eq!(
        serde_json::from_value::<PhysicsJointSyncState>(serde_json::to_value(&joint).unwrap())
            .unwrap(),
        joint
    );
    assert_eq!(
        serde_json::from_value::<PhysicsTriggerEvent>(serde_json::to_value(&trigger).unwrap())
            .unwrap(),
        trigger
    );
    assert_eq!(serde_json::to_value(trigger.kind).unwrap(), "enter");
}

#[test]
fn body_collider_material_and_contact_sync_use_scene_identity() {
    let body = PhysicsBodySyncState {
        entity: 42,
        body_type: PhysicsBodyType::Dynamic,
        transform: Transform::default(),
        mass: 2.0,
        linear_velocity: [1.0, 0.0, 0.0],
        angular_velocity: [0.0, 1.0, 0.0],
        linear_damping: 0.1,
        angular_damping: 0.2,
        gravity_scale: 1.0,
        can_sleep: true,
        lock_translation: [false, true, false],
        lock_rotation: [false, false, true],
    };
    let collider = PhysicsColliderSyncState {
        entity: body.entity,
        shape: PhysicsColliderShape::Box {
            half_extents: [0.5, 1.0, 0.5],
        },
        sensor: true,
        layer: 1,
        collision_group: 2,
        collision_mask: 0b11,
        material: Some("res://physics/materials/metal.physics_material.toml".to_string()),
        material_override: Some(PhysicsMaterialMetadata {
            static_friction: 0.7,
            dynamic_friction: 0.5,
            restitution: 0.1,
            friction_combine: PhysicsCombineRule::Average,
            restitution_combine: PhysicsCombineRule::Maximum,
        }),
        transform: Transform::default(),
    };
    let contact = PhysicsContactEvent {
        world: WorldHandle::new(7),
        entity: body.entity,
        other_entity: 99,
        point: [0.0, 1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    };

    assert_eq!(
        serde_json::from_value::<PhysicsBodySyncState>(serde_json::to_value(&body).unwrap())
            .unwrap(),
        body
    );
    assert_eq!(
        serde_json::from_value::<PhysicsColliderSyncState>(
            serde_json::to_value(&collider).unwrap()
        )
        .unwrap(),
        collider
    );
    assert_eq!(
        serde_json::from_value::<PhysicsContactEvent>(serde_json::to_value(&contact).unwrap())
            .unwrap(),
        contact
    );
}
