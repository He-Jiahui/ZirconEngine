use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsColliderShape, PhysicsJointSyncState, PhysicsJointType, PhysicsQueryFilter,
        PhysicsRayCastQuery, PhysicsSettings, PhysicsTriggerEventKind,
    },
    scene::physics::{PhysicsJointConstraintMetadata, PhysicsMaterialMetadata},
};
use zircon_runtime::core::math::Transform;

use super::body_desc;
use crate::backend::{
    BodyCommand, BuiltinPhysicsBackend, ConstraintDesc, PhysicsBackend, PhysicsBackendError,
    PhysicsBackendObjectKind, PhysicsEventBuffer,
};
use crate::JointParams;

#[test]
fn physics_backend_generation_rejects_destroyed_body_after_slot_reuse() {
    let world = WorldHandle::new(11);
    let mut backend: Box<dyn PhysicsBackend> =
        Box::new(BuiltinPhysicsBackend::new(PhysicsSettings::default()));
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let destroyed = backend
        .create_body(&body_desc(world, 41, shape))
        .expect("first body");

    backend.destroy_body(destroyed).expect("destroy first body");
    let replacement = backend
        .create_body(&body_desc(world, 42, shape))
        .expect("replacement body");

    assert_ne!(destroyed, replacement, "slot reuse must advance generation");
    assert!(matches!(
        backend.apply_commands(&[BodyCommand::SetLinearVelocity {
            body: destroyed,
            velocity: [1.0, 0.0, 0.0],
        }]),
        Err(PhysicsBackendError::InvalidHandle {
            kind: PhysicsBackendObjectKind::Body,
            ..
        })
    ));
}

#[test]
fn builtin_physics_backend_trait_steps_active_bodies_and_answers_queries() {
    let world = WorldHandle::new(12);
    let mut backend: Box<dyn PhysicsBackend> =
        Box::new(BuiltinPhysicsBackend::new(PhysicsSettings::default()));
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let body = backend
        .create_body(&body_desc(world, 51, shape))
        .expect("dynamic body");

    backend.step(1.0 / 60.0).expect("builtin step");
    let mut active = Vec::new();
    backend.read_active_states(&mut active);

    let (_, state) = active
        .iter()
        .find(|(handle, _)| *handle == body)
        .expect("stepped body should be active");
    assert!(state.linear_velocity[1] < 0.0);
    assert!(state.transform.translation.y < 2.0);

    let filter = PhysicsQueryFilter::default();
    let mut hits = Vec::new();
    backend.ray_cast(
        &PhysicsRayCastQuery {
            world,
            origin: [0.0, 5.0, 0.0],
            direction: [0.0, -1.0, 0.0],
            max_distance: 10.0,
            mode: Default::default(),
            filter: filter.clone(),
        },
        &filter,
        &mut hits,
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entity, 51);
}

#[test]
fn builtin_backend_drain_events_reports_trigger_enter_stay_exit() {
    let world = WorldHandle::new(14);
    let mut backend = BuiltinPhysicsBackend::new(PhysicsSettings::default());
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let trigger = backend
        .create_body(&{
            let mut desc = body_desc(world, 71, shape);
            desc.collider.sensor = true;
            desc
        })
        .expect("trigger body");
    backend
        .create_body(&body_desc(world, 72, shape))
        .expect("other body");

    backend.step(1.0 / 60.0).expect("enter step");
    assert_drained_trigger(&mut backend, PhysicsTriggerEventKind::Enter);

    backend.step(1.0 / 60.0).expect("stay step");
    assert_drained_trigger(&mut backend, PhysicsTriggerEventKind::Stay);

    backend
        .apply_commands(&[BodyCommand::Teleport {
            body: trigger,
            transform: Transform::from_translation([8.0, 2.0, 0.0].into()),
        }])
        .expect("move trigger out of overlap");
    backend.step(1.0 / 60.0).expect("exit step");
    assert_drained_trigger(&mut backend, PhysicsTriggerEventKind::Exit);
}

fn assert_drained_trigger(backend: &mut BuiltinPhysicsBackend, expected: PhysicsTriggerEventKind) {
    let mut events = PhysicsEventBuffer::default();
    backend.drain_events(&mut events);
    assert_eq!(events.triggers.len(), 1);
    assert_eq!(events.triggers[0].kind, expected);
    assert_eq!(events.triggers[0].trigger_entity, 71);
    assert_eq!(events.triggers[0].other_entity, 72);
}

#[test]
fn builtin_constraint_gap_is_a_typed_unsupported_error() {
    let world = WorldHandle::new(13);
    let mut backend = BuiltinPhysicsBackend::new(PhysicsSettings::default());
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let body_a = backend
        .create_body(&body_desc(world, 61, shape))
        .expect("first constraint body");
    let body_b = backend
        .create_body(&body_desc(world, 62, shape))
        .expect("second constraint body");

    let error = backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Hinge,
            body_a,
            body_b: Some(body_b),
            anchor_a: Transform::default(),
            anchor_b: Transform::default(),
            params: JointParams::Hinge {
                axis: [0.0, 1.0, 0.0],
                limit: None,
                motor: None,
            },
            collide_connected: false,
        })
        .expect_err("hinge remains outside builtin M1 scope");

    assert!(matches!(
        error,
        PhysicsBackendError::Unsupported {
            backend: "builtin",
            operation: "create_constraint",
            ..
        }
    ));
}

#[test]
fn joint_resolves_entity_pair_to_handles() {
    let world = WorldHandle::new(15);
    let mut backend = BuiltinPhysicsBackend::new(PhysicsSettings::default());
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let body_a = backend
        .create_body(&body_desc(world, 81, shape))
        .expect("joint owner body");
    let body_b = backend
        .create_body(&body_desc(world, 82, shape))
        .expect("connected joint body");
    let joint = PhysicsJointSyncState {
        entity: 81,
        kind: PhysicsJointType::Slider,
        connected_entity: Some(82),
        anchor: [0.0, 0.5, 0.0],
        axis: [1.0, 0.0, 0.0],
        limits: Some([-0.25, 0.75]),
        collide_connected: true,
        constraint: PhysicsJointConstraintMetadata::default(),
        skeleton_binding: None,
    };

    let desc = ConstraintDesc::from_joint_sync(&joint, |entity| match entity {
        81 => Some(body_a),
        82 => Some(body_b),
        _ => None,
    })
    .expect("joint entities should resolve to body handles");

    assert_eq!(desc.body_a, body_a);
    assert_eq!(desc.body_b, Some(body_b));
    assert_eq!(desc.joint_type, PhysicsJointType::Slider);
    assert_eq!(desc.anchor_a.translation.to_array(), joint.anchor);
    assert_eq!(desc.anchor_b.translation.to_array(), joint.anchor);
    assert_eq!(desc.params.axis(), joint.axis);
    assert!(desc.collide_connected);
}

#[test]
fn builtin_fixed_and_distance_constraints_project_body_motion() {
    let world = WorldHandle::new(16);
    let mut backend = BuiltinPhysicsBackend::new(PhysicsSettings::default());
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .expect("builtin sphere shape");
    let mut first = body_desc(world, 91, shape);
    first.body.gravity_scale = 0.0;
    first.body.transform.translation.x = 2.0;
    first.collider.transform = first.body.transform;
    let body_a = backend.create_body(&first).expect("fixed body");
    backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Fixed,
            body_a,
            body_b: None,
            anchor_a: Transform::default(),
            anchor_b: Transform::default(),
            params: JointParams::Fixed,
            collide_connected: false,
        })
        .expect("builtin fixed constraint");
    backend.step(1.0 / 60.0).expect("fixed projection step");

    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let fixed = active
        .iter()
        .find(|(handle, _)| *handle == body_a)
        .expect("fixed body state");
    assert!(fixed.1.transform.translation.length() < 1.0e-5);

    let mut second = body_desc(world, 92, shape);
    second.body.gravity_scale = 0.0;
    second.body.transform.translation.x = 4.0;
    second.collider.transform = second.body.transform;
    let body_b = backend.create_body(&second).expect("distance body");
    backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Distance,
            body_a: body_b,
            body_b: Some(body_a),
            anchor_a: Transform::default(),
            anchor_b: Transform::default(),
            params: JointParams::Distance {
                min: 1.0,
                max: 1.0,
                spring: None,
            },
            collide_connected: false,
        })
        .expect("builtin distance constraint");
    backend.step(1.0 / 60.0).expect("distance projection step");
    active.clear();
    backend.read_active_states(&mut active);
    let position = |handle| {
        active
            .iter()
            .find(|(candidate, _)| *candidate == handle)
            .expect("projected body state")
            .1
            .transform
            .translation
    };
    assert!((position(body_b) - position(body_a)).length() <= 1.0001);
}

#[test]
fn trimesh_on_builtin_reports_unsupported() {
    use zircon_runtime::core::resource::{AssetReference, ResourceLocator};

    let mut backend = BuiltinPhysicsBackend::new(PhysicsSettings::default());
    let mesh = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/static_level.physics_mesh").unwrap(),
    );
    let error = backend
        .create_shape(
            &PhysicsColliderShape::TriangleMesh { mesh },
            &PhysicsMaterialMetadata::default(),
        )
        .expect_err("builtin must reject production triangle meshes explicitly");

    assert!(matches!(
        error,
        PhysicsBackendError::Unsupported {
            backend: "builtin",
            operation: "create_shape",
            ..
        }
    ));
}
