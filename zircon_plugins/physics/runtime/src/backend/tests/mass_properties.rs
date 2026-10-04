use super::*;

fn unit_box() -> PhysicsColliderShape {
    PhysicsColliderShape::Box {
        half_extents: [0.5; 3],
    }
}

#[test]
fn auto_mass_uses_shape_volume_and_density() {
    let resolved = resolve_body_mass(
        &unit_box(),
        99.0,
        PhysicsMassProperties::AutoFromShape { density: 2.5 },
    )
    .expect("unit box supports automatic mass resolution");

    assert_eq!(resolved.mass, 2.5);
    assert_eq!(resolved.density, 2.5);
    assert_eq!(resolved.inertia_multiplier, 1.0);
}

#[test]
fn explicit_uniform_inertia_scale_maps_to_jolt_multiplier() {
    let resolved = resolve_body_mass(
        &unit_box(),
        2.0,
        PhysicsMassProperties::Explicit {
            inertia_tensor: Some([
                [2.0 / 3.0, 0.0, 0.0],
                [0.0, 2.0 / 3.0, 0.0],
                [0.0, 0.0, 2.0 / 3.0],
            ]),
        },
    )
    .expect("uniform primitive inertia scale is supported");

    assert!((resolved.inertia_multiplier - 2.0).abs() <= INERTIA_RATIO_EPSILON);
}

#[test]
fn zero_volume_shape_rejects_mass_resolution() {
    let error = resolve_body_mass(
        &PhysicsColliderShape::Box {
            half_extents: [0.0, 0.5, 0.5],
        },
        1.0,
        PhysicsMassProperties::AutoFromShape { density: 1.0 },
    )
    .expect_err("zero-volume colliders cannot define mass");

    assert!(matches!(
        error,
        PhysicsBackendError::Unsupported {
            operation: "resolve_mass_properties",
            ..
        }
    ));
}
