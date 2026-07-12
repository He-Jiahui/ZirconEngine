use std::collections::HashMap;

use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
        PhysicsJointType, PhysicsMeshAsset, PhysicsSettings, PhysicsTriggerEventKind,
    },
    scene::physics::{
        PhysicsCcdMode, PhysicsMassProperties, PhysicsMaterialMetadata, PhysicsSleepPolicy,
    },
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::{AssetReference, ResourceLocator};

use crate::backend::{
    BodyCommand, BodyDesc, ConstraintDesc, JoltPhysicsBackend, PhysicsBackend, PhysicsBackendError,
    PhysicsEventBuffer, ShapeHandle,
};
use crate::{AxisConstraint, JointParams};

const BOX_HALF_EXTENT: f32 = 0.5;
const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const SETTLE_STEPS: usize = 360;

#[test]
fn hinge_pendulum_period_matches_analytic() {
    let world = WorldHandle::new(37);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.2 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create pendulum shape");
    let mut desc = body_desc(world, 370, shape, shape_desc, PhysicsBodyType::Dynamic, 0.0);
    let length = 1.0;
    let initial_angle = 0.15_f32;
    desc.body.transform.translation = Vec3::new(
        length * initial_angle.sin(),
        -length * initial_angle.cos(),
        0.0,
    );
    desc.collider.transform = desc.body.transform;
    let body = backend.create_body(&desc).expect("create pendulum body");
    backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Hinge,
            body_a: body,
            body_b: None,
            anchor_a: Transform::from_translation(Vec3::new(0.0, length, 0.0)),
            anchor_b: Transform::default(),
            params: JointParams::Hinge {
                axis: [0.0, 0.0, 1.0],
                limit: None,
                motor: None,
            },
            collide_connected: false,
        })
        .expect("create hinge constraint");

    let mut previous_x = desc.body.transform.translation.x;
    let mut positive_crossings = Vec::new();
    for step in 1..600 {
        backend.step(FIXED_STEP_SECONDS).expect("step pendulum");
        let mut active = Vec::new();
        backend.read_active_states(&mut active);
        if let Some(state) = active
            .into_iter()
            .find_map(|(candidate, state)| (candidate == body).then_some(state))
        {
            let x = state.transform.translation.x;
            if previous_x <= 0.0 && x > 0.0 {
                positive_crossings.push(step as f32 * FIXED_STEP_SECONDS);
                if positive_crossings.len() == 2 {
                    break;
                }
            }
            previous_x = x;
        }
    }

    assert_eq!(
        positive_crossings.len(),
        2,
        "pendulum must complete a period"
    );
    let measured = positive_crossings[1] - positive_crossings[0];
    let analytic = 2.0 * std::f32::consts::PI * (length / 9.81).sqrt();
    assert!(
        (measured - analytic).abs() <= 0.35,
        "measured period {measured} differs from analytic {analytic}"
    );
}

#[test]
fn slider_limit_clamps_travel() {
    let world = WorldHandle::new(38);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.25 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create slider shape");
    let mut desc = body_desc(world, 380, shape, shape_desc, PhysicsBodyType::Dynamic, 0.0);
    desc.body.transform.translation.x = 3.0;
    desc.collider.transform = desc.body.transform;
    desc.body.gravity_scale = 0.0;
    let body = backend.create_body(&desc).expect("create slider body");
    backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Slider,
            body_a: body,
            body_b: None,
            anchor_a: Transform::default(),
            anchor_b: Transform::default(),
            params: JointParams::Slider {
                axis: [1.0, 0.0, 0.0],
                limit: Some([-0.5, 0.5]),
                motor: None,
            },
            collide_connected: false,
        })
        .expect("create slider constraint");

    backend.step(FIXED_STEP_SECONDS).expect("step slider");
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let state = active
        .into_iter()
        .find_map(|(candidate, state)| (candidate == body).then_some(state))
        .expect("projected slider body");
    assert!(state.transform.translation.x <= 0.5001);
}

#[test]
fn six_dof_swing_twist_respects_limits() {
    let world = WorldHandle::new(39);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.25 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create six-dof shape");
    let mut desc = body_desc(world, 390, shape, shape_desc, PhysicsBodyType::Dynamic, 0.0);
    desc.body.transform.translation = Vec3::new(2.0, -2.0, 1.0);
    desc.collider.transform = desc.body.transform;
    desc.body.angular_velocity = [3.0, -4.0, 5.0];
    desc.body.gravity_scale = 0.0;
    let body = backend.create_body(&desc).expect("create six-dof body");
    let linear = std::array::from_fn(|_| AxisConstraint {
        limit: Some([-0.25, 0.25]),
        drive: None,
    });
    let angular = std::array::from_fn(|_| AxisConstraint {
        limit: Some([-0.5, 0.5]),
        drive: None,
    });
    backend
        .create_constraint(&ConstraintDesc {
            joint_type: PhysicsJointType::Generic6Dof,
            body_a: body,
            body_b: None,
            anchor_a: Transform::default(),
            anchor_b: Transform::default(),
            params: JointParams::Generic6Dof {
                axis: [1.0, 0.0, 0.0],
                linear,
                angular,
            },
            collide_connected: false,
        })
        .expect("create six-dof constraint");

    backend.step(FIXED_STEP_SECONDS).expect("step six-dof");
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let state = active
        .into_iter()
        .find_map(|(candidate, state)| (candidate == body).then_some(state))
        .expect("projected six-dof body");
    assert!(state
        .transform
        .translation
        .to_array()
        .into_iter()
        .all(|value| value.abs() <= 0.2501));
    assert!(state
        .angular_velocity
        .into_iter()
        .all(|value| value.abs() <= 0.5001));
}

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

#[test]
fn convex_hull_round_trips_through_jolt() {
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::ConvexHull {
        points: vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.0, 0.5, -0.5],
            [0.0, 0.0, 0.5],
        ],
    };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create planned Jolt convex hull");

    backend
        .destroy_shape(shape)
        .expect("destroy Jolt convex hull");
}

#[test]
fn asset_backed_mesh_shapes_round_trip_through_jolt_after_registration() {
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let triangle_mesh = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/triangle.physics_mesh").unwrap(),
    );
    backend
        .register_mesh_asset(
            triangle_mesh.clone(),
            PhysicsMeshAsset::TriangleMesh {
                vertices: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
                indices: vec![[0, 1, 2]],
            },
        )
        .expect("register triangle mesh asset");
    let height_field = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/terrain.physics_mesh").unwrap(),
    );
    backend
        .register_mesh_asset(
            height_field.clone(),
            PhysicsMeshAsset::HeightField {
                resolution: [2, 2],
                heights: vec![0.0, 0.25, 0.5, 0.75],
            },
        )
        .expect("register height field asset");

    for shape_desc in [
        PhysicsColliderShape::TriangleMesh {
            mesh: triangle_mesh,
        },
        PhysicsColliderShape::HeightField {
            resolution: [2, 2],
            heights: height_field,
        },
    ] {
        let shape = backend
            .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
            .expect("create registered Jolt mesh shape");
        backend.destroy_shape(shape).expect("destroy mesh shape");
    }
}

#[test]
fn triangle_mesh_shapes_reject_non_static_jolt_bodies() {
    let world = WorldHandle::new(33);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let reference = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/static_triangle.physics_mesh").unwrap(),
    );
    backend
        .register_mesh_asset(
            reference.clone(),
            PhysicsMeshAsset::TriangleMesh {
                vertices: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
                indices: vec![[0, 1, 2]],
            },
        )
        .expect("register triangle mesh asset");
    let shape_desc = PhysicsColliderShape::TriangleMesh { mesh: reference };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create registered triangle mesh shape");

    let error = backend
        .create_body(&body_desc(
            world,
            330,
            shape,
            shape_desc.clone(),
            PhysicsBodyType::Dynamic,
            0.0,
        ))
        .expect_err("triangle meshes must reject dynamic bodies");
    assert!(matches!(
        error,
        PhysicsBackendError::InvalidDescriptor { .. }
    ));

    let body = backend
        .create_body(&body_desc(
            world,
            331,
            shape,
            shape_desc,
            PhysicsBodyType::Static,
            0.0,
        ))
        .expect("triangle meshes support static bodies");
    backend
        .destroy_body(body)
        .expect("destroy static mesh body");
    backend.destroy_shape(shape).expect("destroy mesh shape");
}

#[test]
fn auto_mass_matches_shape_volume() {
    let world = WorldHandle::new(34);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Box {
        half_extents: [0.5; 3],
    };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create unit-volume box shape");
    let mut desc = body_desc(world, 340, shape, shape_desc, PhysicsBodyType::Dynamic, 0.0);
    desc.body.mass_properties = PhysicsMassProperties::AutoFromShape { density: 2.0 };
    desc.body.gravity_scale = 0.0;
    let body = backend.create_body(&desc).expect("create auto-mass body");

    backend
        .apply_commands(&[BodyCommand::ApplyImpulse {
            body,
            impulse: [2.0, 0.0, 0.0],
        }])
        .expect("apply impulse to auto-mass body");
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let state = active
        .into_iter()
        .find_map(|(handle, state)| (handle == body).then_some(state))
        .expect("auto-mass body stays active after impulse");
    assert!((state.linear_velocity[0] - 1.0).abs() <= 1.0e-4);
}

#[test]
fn kinematic_to_dynamic_preserves_velocity() {
    let world = WorldHandle::new(35);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.5 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create kinematic sphere shape");
    let mut desc = body_desc(
        world,
        350,
        shape,
        shape_desc,
        PhysicsBodyType::Kinematic,
        0.0,
    );
    desc.body.linear_velocity = [3.0, 0.0, 0.0];
    desc.body.angular_velocity = [0.0, 2.0, 0.0];
    desc.body.gravity_scale = 0.0;
    let body = backend.create_body(&desc).expect("create kinematic body");

    backend
        .apply_commands(&[BodyCommand::SetBodyType {
            body,
            body_type: PhysicsBodyType::Dynamic,
        }])
        .expect("switch kinematic body to dynamic");
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let state = active
        .into_iter()
        .find_map(|(handle, state)| (handle == body).then_some(state))
        .expect("switched body remains active");
    assert_eq!(state.body_type, PhysicsBodyType::Dynamic);
    assert_eq!(state.linear_velocity, [3.0, 0.0, 0.0]);
    assert_eq!(state.angular_velocity, [0.0, 2.0, 0.0]);
}

#[test]
fn ccd_and_sleep_policy_switch_without_recreating_body() {
    let world = WorldHandle::new(36);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.5 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create policy sphere shape");
    let body = backend
        .create_body(&body_desc(
            world,
            360,
            shape,
            shape_desc,
            PhysicsBodyType::Dynamic,
            0.0,
        ))
        .expect("create policy body");

    backend
        .apply_commands(&[
            BodyCommand::SetCcdMode {
                body,
                mode: PhysicsCcdMode::LinearCast,
            },
            BodyCommand::SetSleepPolicy {
                body,
                policy: PhysicsSleepPolicy::Never,
            },
        ])
        .expect("switch CCD and sleep policy");
    assert_eq!(
        backend.debug_body_runtime_policy(body),
        Some((
            PhysicsBodyType::Dynamic,
            PhysicsCcdMode::LinearCast,
            PhysicsSleepPolicy::Never,
        ))
    );
}

#[test]
fn jolt_drain_events_reports_trigger_lifecycle() {
    let world = WorldHandle::new(37);
    let mut backend =
        JoltPhysicsBackend::new(PhysicsSettings::default()).expect("initialize native Jolt world");
    let shape_desc = PhysicsColliderShape::Sphere { radius: 0.5 };
    let shape = backend
        .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
        .expect("create Jolt trigger shape");
    let mut trigger_desc = body_desc(
        world,
        370,
        shape,
        shape_desc.clone(),
        PhysicsBodyType::Dynamic,
        0.0,
    );
    trigger_desc.body.gravity_scale = 0.0;
    trigger_desc.collider.sensor = true;
    let trigger = backend
        .create_body(&trigger_desc)
        .expect("create Jolt trigger body");
    let mut other_desc = body_desc(world, 371, shape, shape_desc, PhysicsBodyType::Static, 0.0);
    other_desc.body.gravity_scale = 0.0;
    backend
        .create_body(&other_desc)
        .expect("create Jolt overlap body");

    step_read_and_assert_trigger(&mut backend, PhysicsTriggerEventKind::Enter);
    step_read_and_assert_trigger(&mut backend, PhysicsTriggerEventKind::Stay);

    backend
        .apply_commands(&[BodyCommand::Teleport {
            body: trigger,
            transform: Transform::from_translation(Vec3::new(8.0, 0.0, 0.0)),
        }])
        .expect("move Jolt trigger out of overlap");
    step_read_and_assert_trigger(&mut backend, PhysicsTriggerEventKind::Exit);
}

fn step_read_and_assert_trigger(
    backend: &mut JoltPhysicsBackend,
    expected: PhysicsTriggerEventKind,
) {
    backend.step(FIXED_STEP_SECONDS).expect("step Jolt events");
    backend.read_active_states(&mut Vec::new());
    let mut events = PhysicsEventBuffer::default();
    backend.drain_events(&mut events);
    assert_eq!(events.triggers.len(), 1);
    assert_eq!(events.triggers[0].kind, expected);
    assert_eq!(events.triggers[0].trigger_entity, 370);
    assert_eq!(events.triggers[0].other_entity, 371);
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
            mass_properties: PhysicsMassProperties::Explicit {
                inertia_tensor: None,
            },
            linear_velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            linear_damping: 0.05,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            ccd_mode: PhysicsCcdMode::Disabled,
            sleep_policy: PhysicsSleepPolicy::Allow,
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
