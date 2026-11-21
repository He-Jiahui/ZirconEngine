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

#[test]
fn jolt_direct_trait_rejects_nonfinite_command_batches_before_any_mutation() {
    use zircon_runtime::core::framework::physics::PhysicsBodyType;
    for (scenario, body_type, lock_x) in [
        (0, PhysicsBodyType::Dynamic, false),
        (1, PhysicsBodyType::Dynamic, true),
        (2, PhysicsBodyType::Static, false),
    ] {
        let mut backend = JoltPhysicsBackend::new(PhysicsSettings::default())
            .expect("initialize native Jolt world");
        let shape_desc = PhysicsColliderShape::Sphere { radius: 0.5 };
        let shape = backend
            .create_shape(&shape_desc, &PhysicsMaterialMetadata::default())
            .expect("valid sphere shape");
        let mut desc = body_desc(
            WorldHandle::new(81),
            810 + scenario as u64,
            shape,
            shape_desc,
            body_type,
            2.0,
        );
        desc.body.body_type = body_type;
        desc.body.lock_translation[0] = lock_x;
        desc.body.gravity_scale = 0.0;
        desc.body.linear_damping = 0.0;
        desc.body.angular_damping = 0.0;
        let body = backend.create_body(&desc).expect("valid body");
        let original = desc.body.transform;
        let mut moved = original;
        moved.translation.x += 9.0;
        let mut invalid_transform = original;
        invalid_transform.translation.x = f32::NAN;
        let mut invalid_rotation = original;
        invalid_rotation.rotation.x = f32::INFINITY;
        let mut invalid_scale = original;
        invalid_scale.scale.y = f32::NEG_INFINITY;
        for invalid in [
            BodyCommand::SetLinearVelocity {
                body,
                velocity: [f32::NAN, 0.0, 0.0],
            },
            BodyCommand::SetAngularVelocity {
                body,
                velocity: [0.0, f32::INFINITY, 0.0],
            },
            BodyCommand::ApplyForce {
                body,
                force: [0.0, 0.0, f32::NEG_INFINITY],
            },
            BodyCommand::ApplyImpulse {
                body,
                impulse: [f32::NAN, 0.0, 0.0],
            },
            BodyCommand::Teleport {
                body,
                transform: invalid_transform,
            },
            BodyCommand::Teleport {
                body,
                transform: invalid_rotation,
            },
            BodyCommand::Teleport {
                body,
                transform: invalid_scale,
            },
        ] {
            let provider: &mut dyn PhysicsBackend = &mut backend;
            let result = provider.apply_commands(&[
                BodyCommand::Teleport {
                    body,
                    transform: moved,
                },
                invalid,
            ]);
            assert!(matches!(
                result,
                Err(PhysicsBackendError::InvalidDescriptor { .. })
            ));
        }
        // Static and locked-axis cases must reject storage pollution too, before integration.
        backend
            .apply_commands(&[BodyCommand::SetBodyType {
                body,
                body_type: PhysicsBodyType::Dynamic,
            }])
            .expect("valid command remains accepted after rejected batches");
        backend
            .step(1.0 / 60.0)
            .expect("finite body can still step");
        let mut active = Vec::new();
        backend.read_active_states(&mut active);
        let state = active
            .iter()
            .find_map(|(handle, state)| (*handle == body).then_some(state))
            .expect("body remains active");
        assert_eq!(state.transform.translation, original.translation);
        assert_eq!(state.linear_velocity, [0.0; 3]);
        assert_eq!(state.angular_velocity, [0.0; 3]);
        assert!(state.transform.rotation.is_finite());
        assert!(state.transform.scale.is_finite());
    }
}

#[test]
fn native_offset_floor_stops_sphere_at_authored_center() {
    let row = native_point_probe("offset", NativePointPlacement::Offset);
    assert!(
        row.correct(),
        "native solver must honor the authored collider center: {row:?}"
    );
}

#[test]
fn native_rotated_offset_floor_stops_sphere_at_authored_center() {
    let row = native_point_probe("rotated", NativePointPlacement::RotatedOffset);
    assert!(
        row.correct(),
        "native solver must resolve the collider center in the body frame: {row:?}"
    );
}

#[test]
fn native_zero_offset_floor_keeps_existing_response() {
    let row = native_point_probe("zero", NativePointPlacement::Zero);
    assert!(
        row.correct(),
        "zero-offset native floor must retain its response: {row:?}"
    );
}

#[test]
fn native_translated_body_floor_keeps_existing_response() {
    let row = native_point_probe("translated", NativePointPlacement::Translated);
    assert!(
        row.correct(),
        "translated native body must retain its response: {row:?}"
    );
}

#[derive(Clone, Copy)]
enum NativePointPlacement {
    Offset,
    RotatedOffset,
    Zero,
    Translated,
}

#[derive(Debug)]
struct NativePointRow {
    target_y: f32,
    other_y: f32,
    target_vy: f32,
    other_vy: f32,
    target_tail_range: f32,
}

impl NativePointRow {
    fn correct(&self) -> bool {
        (self.target_y - 0.25).abs() < 0.06
            && self.other_y < -25.0
            && self.target_vy.abs() < 0.1
            && self.other_vy < -20.0
            && self.target_tail_range < 0.02
    }
}

fn native_point_probe(label: &str, placement: NativePointPlacement) -> NativePointRow {
    use zircon_runtime::core::math::Quat;
    const STEPS: usize = 180;
    let mut backend = JoltPhysicsBackend::new(PhysicsSettings::default())
        .expect("initialize real native Jolt world, without a fallback");
    assert_eq!(backend.name(), "jolt");
    let material = PhysicsMaterialMetadata {
        restitution: 0.0,
        ..PhysicsMaterialMetadata::default()
    };
    let floor_geometry = PhysicsColliderShape::Box {
        half_extents: [0.5; 3],
    };
    let ball_geometry = PhysicsColliderShape::Sphere { radius: 0.25 };
    let floor_shape = backend
        .create_shape(&floor_geometry, &material)
        .expect("native floor geometry");
    let ball_shape = backend
        .create_shape(&ball_geometry, &material)
        .expect("shared native sphere geometry");
    let (body_origin, rotation, center, other) = match placement {
        NativePointPlacement::Offset => (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Vec3::new(8.0, -0.5, 0.0),
            Vec3::ZERO,
        ),
        NativePointPlacement::RotatedOffset => (
            Vec3::new(3.0, -0.5, 4.0),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::new(3.0, -0.5, -4.0),
            Vec3::new(3.0, 0.0, 4.0),
        ),
        NativePointPlacement::Zero => (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::X * 8.0,
        ),
        NativePointPlacement::Translated => (
            Vec3::new(8.0, -0.5, 0.0),
            Quat::IDENTITY,
            Vec3::new(8.0, -0.5, 0.0),
            Vec3::ZERO,
        ),
    };
    let world = WorldHandle::new(890);
    let mut floor = body_desc(
        world,
        8900,
        floor_shape,
        floor_geometry,
        PhysicsBodyType::Static,
        -0.5,
    );
    floor.body.transform.translation = body_origin;
    floor.body.transform.rotation = rotation;
    floor.collider.transform.translation = center;
    floor.collider.transform.rotation = rotation;
    let original_floor = floor.clone();
    let floor_handle = backend
        .create_body(&floor)
        .expect("native static body with separate collider center");
    assert_eq!(floor, original_floor);
    native_body_origin_matches(&backend, floor_handle, floor.body.transform);
    let starts = [
        Vec3::new(center.x, 2.0, center.z),
        Vec3::new(other.x, 2.0, other.z),
    ];
    assert!(starts[0].distance(starts[1]) > 7.9);
    let mut handles = Vec::new();
    for (index, start) in starts.iter().copied().enumerate() {
        let mut ball = body_desc(
            world,
            8901 + index as u64,
            ball_shape,
            ball_geometry.clone(),
            PhysicsBodyType::Dynamic,
            2.0,
        );
        ball.body.transform.translation = start;
        ball.collider.transform = ball.body.transform;
        ball.body.sleep_policy = PhysicsSleepPolicy::Never;
        ball.body.linear_damping = 0.0;
        ball.body.angular_damping = 0.0;
        let handle = backend.create_body(&ball).expect("native dynamic sphere");
        assert_eq!(
            backend.debug_body_runtime_policy(handle).unwrap().2,
            PhysicsSleepPolicy::Never
        );
        handles.push(handle);
    }
    assert!(
        backend.destroy_shape(ball_shape).is_err(),
        "shared shape stays referenced by both bodies"
    );
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    for (handle, start) in handles.iter().zip(starts) {
        let state = active
            .iter()
            .find_map(|(h, s)| (h == handle).then_some(s))
            .expect("new sphere is natively active");
        assert!(state.transform.translation.distance(start) < 1e-5);
    }
    let mut heights = [Vec::new(), Vec::new()];
    let mut velocities = [Vec::new(), Vec::new()];
    for _ in 0..STEPS {
        backend
            .step(FIXED_STEP_SECONDS)
            .expect("actual JPC native physics update");
        active.clear();
        backend.read_active_states(&mut active);
        for index in 0..2 {
            let state = active
                .iter()
                .find_map(|(h, s)| (*h == handles[index]).then_some(s))
                .expect("NeverSleep native sphere must produce every sampled state");
            let p = state.transform.translation;
            assert!(p.is_finite() && state.linear_velocity.iter().all(|v| v.is_finite()));
            assert!((p.x - starts[index].x).abs() < 0.02 && (p.z - starts[index].z).abs() < 0.02);
            assert_eq!(state.body_type, PhysicsBodyType::Dynamic);
            heights[index].push(p.y);
            velocities[index].push(state.linear_velocity[1]);
        }
    }
    for ys in &heights {
        assert_eq!(ys.len(), STEPS);
        assert!(
            ys[0] < 2.0 && ys[0] > 1.9 && ys[29] < 1.0 && ys[29] > 0.3,
            "actual initial free-fall, not a canned rest pose"
        );
    }
    let tail = &heights[0][STEPS - 30..];
    let low = tail.iter().copied().fold(f32::INFINITY, f32::min);
    let high = tail.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let row = NativePointRow {
        target_y: heights[0][STEPS - 1],
        other_y: heights[1][STEPS - 1],
        target_vy: velocities[0][STEPS - 1],
        other_vy: velocities[1][STEPS - 1],
        target_tail_range: high - low,
    };
    native_body_origin_matches(&backend, floor_handle, floor.body.transform);
    println!("NATIVE_POINT label={label} target_y={} other_y={} target_vy={} other_vy={} target_tail={} initial_y={} early_y={} frames={} native=true commands=0 constraints=0 events_used=false", row.target_y, row.other_y, row.target_vy, row.other_vy, row.target_tail_range, heights[0][0], heights[0][29], STEPS);
    println!("NATIVE_ORIGIN label={label} static_origin=pass before_after=true");
    for handle in handles {
        backend
            .destroy_body(handle)
            .expect("release both dynamic references");
    }
    backend
        .destroy_body(floor_handle)
        .expect("release floor body");
    backend
        .destroy_shape(ball_shape)
        .expect("release shared sphere shape after both bodies");
    backend
        .destroy_shape(floor_shape)
        .expect("release original floor shape");
    row
}

fn native_body_origin_matches(
    backend: &JoltPhysicsBackend,
    handle: crate::backend::BodyHandle,
    expected: Transform,
) {
    let observed = backend
        .debug_native_body_state(handle)
        .expect("native body exists");
    assert!(
        observed
            .transform
            .translation
            .distance(expected.translation)
            < 1e-4,
        "direct native origin is not the collider/COM: {:?} versus {:?}",
        observed.transform,
        expected
    );
    assert!(observed.transform.rotation.dot(expected.rotation).abs() > 1.0 - 1e-5);
}

#[test]
fn native_dynamic_offset_retains_body_origin_and_hits_authored_geometry() {
    let row = native_owned_point_probe("dynamic", false);
    assert!(
        row.correct(),
        "density-owned dynamic geometry must use collider center, not move body origin: {row:?}"
    );
}

#[test]
fn native_kinematic_offset_retains_body_origin_and_hits_authored_geometry() {
    let row = native_owned_point_probe("kinematic", true);
    assert!(
        row.correct(),
        "density-owned kinematic geometry must use collider center, not move body origin: {row:?}"
    );
}

#[derive(Debug)]
struct NativeOwnedPointRow {
    center: Vec3,
    vy: f32,
    tail: f32,
    center_frame_error: f32,
    floor_center_error: f32,
}

impl NativeOwnedPointRow {
    fn correct(&self) -> bool {
        (self.center.y - 0.25).abs() < 0.06
            && (self.center.x - 3.0).abs() < 0.5
            && (self.center.z + 4.0).abs() < 0.5
            && self.vy.abs() < 0.1
            && self.tail < 0.02
            && self.center_frame_error < 1e-4
            && self.floor_center_error < 1e-4
    }
}

fn native_owned_point_probe(label: &str, kinematic_floor: bool) -> NativeOwnedPointRow {
    use zircon_runtime::core::math::Quat;
    const STEPS: usize = 180;
    let mut backend = JoltPhysicsBackend::new(PhysicsSettings::default())
        .expect("actual native Jolt, no fallback");
    assert_eq!(backend.name(), "jolt");
    let material = PhysicsMaterialMetadata {
        restitution: 0.0,
        ..PhysicsMaterialMetadata::default()
    };
    let box_geometry = PhysicsColliderShape::Box {
        half_extents: [0.5; 3],
    };
    let ball_geometry = PhysicsColliderShape::Sphere { radius: 0.25 };
    let floor_shape = backend.create_shape(&box_geometry, &material).unwrap();
    let ball_shape = backend.create_shape(&ball_geometry, &material).unwrap();
    let world = WorldHandle::new(891);
    let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let geometry_xz = Vec3::new(3.0, 0.0, -4.0);
    let original_xz = Vec3::new(3.0, 0.0, 4.0);
    let floor_type = if kinematic_floor {
        PhysicsBodyType::Kinematic
    } else {
        PhysicsBodyType::Static
    };
    let mut floor = body_desc(world, 8910, floor_shape, box_geometry, floor_type, -0.5);
    let floor_xz = if kinematic_floor {
        original_xz
    } else {
        geometry_xz
    };
    floor.body.transform.translation = floor_xz - Vec3::Y * 0.5;
    floor.body.transform.rotation = if kinematic_floor {
        rotation
    } else {
        Quat::IDENTITY
    };
    floor.body.sleep_policy = PhysicsSleepPolicy::Never;
    floor.body.gravity_scale = 0.0;
    floor.collider.transform = floor.body.transform;
    floor.collider.transform.translation = geometry_xz - Vec3::Y * 0.5;
    let floor_body = backend
        .create_body(&floor)
        .expect("static/kinematic native support");
    native_body_origin_matches(&backend, floor_body, floor.body.transform);
    let floor_center_error = backend
        .debug_native_center_of_mass(floor_body)
        .unwrap()
        .distance(floor.collider.transform.translation);
    let mut ball = body_desc(
        world,
        8911,
        ball_shape,
        ball_geometry,
        PhysicsBodyType::Dynamic,
        2.0,
    );
    let start_xz = if kinematic_floor {
        geometry_xz
    } else {
        original_xz
    };
    let start = start_xz + Vec3::Y * 2.0;
    ball.body.transform.translation = start;
    ball.body.transform.rotation = if kinematic_floor {
        Quat::IDENTITY
    } else {
        rotation
    };
    ball.body.sleep_policy = PhysicsSleepPolicy::Never;
    ball.body.linear_damping = 0.0;
    ball.body.angular_damping = 0.0;
    ball.collider.transform = ball.body.transform;
    ball.collider.transform.translation = geometry_xz + Vec3::Y * 2.0;
    let ball_body = backend.create_body(&ball).expect("dynamic native probe");
    native_body_origin_matches(&backend, ball_body, ball.body.transform);
    assert_eq!(
        backend.debug_body_runtime_policy(ball_body).unwrap().2,
        PhysicsSleepPolicy::Never
    );
    assert!(backend.destroy_shape(floor_shape).is_err());
    assert!(backend.destroy_shape(ball_shape).is_err());
    let mut active = Vec::new();
    let mut heights = Vec::with_capacity(STEPS);
    let local_sphere_center = if kinematic_floor {
        Vec3::ZERO
    } else {
        Vec3::X * 8.0
    };
    let mut center_frame_error = 0.0_f32;
    let mut final_state = None;
    for _ in 0..STEPS {
        backend
            .step(FIXED_STEP_SECONDS)
            .expect("actual native update");
        active.clear();
        backend.read_active_states(&mut active);
        let state = active
            .iter()
            .find_map(|(h, s)| (*h == ball_body).then_some(s))
            .expect("NeverSleep dynamic origin observed every step");
        let p = state.transform.translation;
        assert!(p.is_finite() && state.linear_velocity.iter().all(|v| v.is_finite()));
        assert!(
            (p.x - start.x).abs() < 0.02 && (p.z - start.z).abs() < 0.02,
            "observed dynamic origin must remain distinct from the offset geometry"
        );
        // For this uniform sphere the native COM is its geometric center, not the body origin.
        let center = backend.debug_native_center_of_mass(ball_body).unwrap();
        assert!(center.is_finite() && state.transform.rotation.is_finite());
        let expected = p + state.transform.rotation * local_sphere_center;
        center_frame_error = center_frame_error.max(center.distance(expected));
        heights.push(center.y);
        final_state = Some(state.clone());
    }
    native_body_origin_matches(&backend, floor_body, floor.body.transform);
    let floor_center_error = floor_center_error.max(
        backend
            .debug_native_center_of_mass(floor_body)
            .unwrap()
            .distance(floor.collider.transform.translation),
    );
    let last = final_state.unwrap();
    let direct = backend
        .debug_native_body_state(ball_body)
        .expect("native origin even without active filter");
    assert!(
        direct
            .transform
            .translation
            .distance(last.transform.translation)
            < 1e-5
    );
    assert_eq!(direct.linear_velocity, last.linear_velocity);
    assert_eq!(heights.len(), STEPS);
    assert!(heights[0] > 1.9 && heights[0] < 2.0 && heights[29] > 0.3 && heights[29] < 1.0);
    let tail = &heights[STEPS - 30..];
    let range = tail.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - tail.iter().copied().fold(f32::INFINITY, f32::min);
    let row = NativeOwnedPointRow {
        center: backend.debug_native_center_of_mass(ball_body).unwrap(),
        vy: last.linear_velocity[1],
        tail: range,
        center_frame_error,
        floor_center_error,
    };
    let origin = direct.transform.translation;
    println!("NATIVE_OWNED label={label} center_x={} center_y={} center_z={} vy={} tail={} origin_x={} origin_y={} origin_z={} center_frame_error={} floor_center_error={} initial_y={} early_y={} frames={} direct_origin=true native_center=true floor_origin=true events_used=false", row.center.x, row.center.y, row.center.z, row.vy, row.tail, origin.x, origin.y, origin.z, row.center_frame_error, row.floor_center_error, heights[0], heights[29], STEPS);
    backend.destroy_body(ball_body).unwrap();
    backend.destroy_body(floor_body).unwrap();
    backend.destroy_shape(ball_shape).unwrap();
    backend.destroy_shape(floor_shape).unwrap();
    row
}
