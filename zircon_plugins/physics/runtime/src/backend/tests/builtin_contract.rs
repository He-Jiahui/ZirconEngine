use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsColliderShape, PhysicsJointType, PhysicsQueryFilter, PhysicsRayCastQuery,
        PhysicsSettings,
    },
    scene::physics::{PhysicsJointConstraintMetadata, PhysicsMaterialMetadata},
};
use zircon_runtime::core::math::Transform;

use super::body_desc;
use crate::backend::{
    BodyCommand, BuiltinPhysicsBackend, ConstraintDesc, PhysicsBackend, PhysicsBackendError,
    PhysicsBackendObjectKind,
};

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
            filter: filter.clone(),
        },
        &filter,
        &mut hits,
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entity, 51);
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
            metadata: PhysicsJointConstraintMetadata::default(),
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
