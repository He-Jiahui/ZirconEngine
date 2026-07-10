use std::collections::HashMap;

use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
        PhysicsSettings,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::{Transform, Vec3};

use crate::backend::{BodyDesc, JoltPhysicsBackend, PhysicsBackend, ShapeHandle};

const BOX_HALF_EXTENT: f32 = 0.5;
const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const SETTLE_STEPS: usize = 360;

#[test]
fn jolt_box_stack_settles_deterministically() {
    let first = run_box_stack();
    let second = run_box_stack();

    for (left, right) in first.into_iter().zip(second) {
        assert!(
            (left - right).abs() <= 1.0e-5,
            "independent fixed-step runs diverged: {left} versus {right}"
        );
    }

    let expected = [0.5, 1.5, 2.5];
    for (actual, expected) in first.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.08,
            "settled box center {actual} differs from snapshot {expected}"
        );
    }
}

#[test]
fn jolt_creates_box_sphere_and_capsule_bodies() {
    let world = WorldHandle::new(32);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let material = PhysicsMaterialMetadata {
        static_friction: 0.5,
        dynamic_friction: 0.4,
        restitution: 0.1,
        ..PhysicsMaterialMetadata::default()
    };
    let shapes = [
        PhysicsColliderShape::Box {
            half_extents: [0.5; 3],
        },
        PhysicsColliderShape::Sphere { radius: 0.5 },
        PhysicsColliderShape::Capsule {
            radius: 0.4,
            half_height: 0.8,
        },
    ];

    for (index, shape_desc) in shapes.into_iter().enumerate() {
        let shape = backend
            .create_shape(&shape_desc, &material)
            .expect("create planned Jolt shape");
        let body = backend
            .create_body(&body_desc(
                world,
                320 + index as EntityId,
                shape,
                shape_desc,
                PhysicsBodyType::Static,
                index as f32 * 3.0,
            ))
            .expect("create planned Jolt body");
        backend.destroy_body(body).expect("destroy Jolt body");
        backend.destroy_shape(shape).expect("destroy Jolt shape");
    }
}

fn run_box_stack() -> [f32; 3] {
    let world = WorldHandle::new(31);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let material = PhysicsMaterialMetadata {
        static_friction: 0.8,
        dynamic_friction: 0.6,
        restitution: 0.0,
        ..PhysicsMaterialMetadata::default()
    };
    let floor_shape_desc = PhysicsColliderShape::Box {
        half_extents: [8.0, BOX_HALF_EXTENT, 8.0],
    };
    let box_shape_desc = PhysicsColliderShape::Box {
        half_extents: [BOX_HALF_EXTENT; 3],
    };
    let floor_shape = backend
        .create_shape(&floor_shape_desc, &material)
        .expect("create Jolt floor shape");
    let box_shape = backend
        .create_shape(&box_shape_desc, &material)
        .expect("create Jolt box shape");
    backend
        .create_body(&body_desc(
            world,
            310,
            floor_shape,
            floor_shape_desc,
            PhysicsBodyType::Static,
            -BOX_HALF_EXTENT,
        ))
        .expect("create Jolt floor body");

    let mut handles = Vec::new();
    for (index, height) in [0.6, 1.8, 3.0].into_iter().enumerate() {
        handles.push(
            backend
                .create_body(&body_desc(
                    world,
                    311 + index as EntityId,
                    box_shape,
                    box_shape_desc.clone(),
                    PhysicsBodyType::Dynamic,
                    height,
                ))
                .expect("create Jolt stack body"),
        );
    }

    let mut last_heights = HashMap::new();
    for _ in 0..SETTLE_STEPS {
        backend.step(FIXED_STEP_SECONDS).expect("step Jolt world");
        let mut active = Vec::new();
        backend.read_active_states(&mut active);
        for (handle, state) in active {
            last_heights.insert(handle, state.transform.translation.y);
        }
    }

    handles
        .into_iter()
        .map(|handle| {
            *last_heights
                .get(&handle)
                .expect("each dynamic body produced an active-state snapshot")
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("box stack always contains three bodies")
}

fn body_desc(
    world: WorldHandle,
    entity: EntityId,
    shape: ShapeHandle,
    collider_shape: PhysicsColliderShape,
    body_type: PhysicsBodyType,
    height: f32,
) -> BodyDesc {
    let transform = Transform::from_translation(Vec3::new(0.0, height, 0.0));
    BodyDesc::from_sync(
        world,
        shape,
        &PhysicsBodySyncState {
            entity,
            body_type,
            transform,
            mass: 1.0,
            linear_velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            linear_damping: 0.05,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            can_sleep: true,
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        },
        &PhysicsColliderSyncState {
            entity,
            shape: collider_shape,
            sensor: false,
            layer: 0,
            collision_group: 0,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform,
        },
    )
    .expect("matching body and collider state")
}
