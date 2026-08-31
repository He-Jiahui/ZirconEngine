use crate::{
    Axis3, NumericError, NumericPolicy, NumericPolicyError, NumericPolicyField,
    NumericPolicyThresholds, NumericValue, Quat, Transform, UnitDirection3, UnitQuaternion,
    ValidatedTransform, Vec3,
};

#[test]
fn numeric_policy_rejects_non_finite_and_negative_minimums() {
    assert!(matches!(
        NumericPolicy::try_new(NumericPolicyThresholds {
            normalized_length_squared: f32::NAN,
            scale_absolute: 0.0,
            matrix_determinant_absolute: 0.0,
        }),
        Err(NumericPolicyError::NonFiniteMinimum {
            field: NumericPolicyField::NormalizedLengthSquared,
        })
    ));
    assert!(matches!(
        NumericPolicy::try_new(NumericPolicyThresholds {
            normalized_length_squared: 0.0,
            scale_absolute: -1.0,
            matrix_determinant_absolute: 0.0,
        }),
        Err(NumericPolicyError::NegativeMinimum {
            field: NumericPolicyField::ScaleAbsolute,
        })
    ));
    assert!(matches!(
        NumericPolicy::try_new(NumericPolicyThresholds {
            normalized_length_squared: 0.0,
            scale_absolute: 0.0,
            matrix_determinant_absolute: -1.0,
        }),
        Err(NumericPolicyError::NegativeMinimum {
            field: NumericPolicyField::MatrixDeterminantAbsolute,
        })
    ));
}

#[test]
fn unit_direction_normalizes_finite_nonzero_input() {
    let direction = UnitDirection3::try_new(Vec3::new(0.0, 3.0, 4.0), NumericPolicy::STRICT)
        .expect("finite nonzero direction is valid");

    assert!((direction.as_vec3().length() - 1.0).abs() <= f32::EPSILON);
    assert!((direction.as_vec3().y - 0.6).abs() <= f32::EPSILON);
    assert!((direction.as_vec3().z - 0.8).abs() <= f32::EPSILON);
}

#[test]
fn numeric_policy_enforces_configured_direction_floor() {
    let policy = NumericPolicy::try_new(NumericPolicyThresholds {
        normalized_length_squared: 0.5,
        scale_absolute: 0.0,
        matrix_determinant_absolute: 0.0,
    })
    .expect("finite nonnegative policy values are valid");
    assert_eq!(
        policy.thresholds(),
        NumericPolicyThresholds {
            normalized_length_squared: 0.5,
            scale_absolute: 0.0,
            matrix_determinant_absolute: 0.0,
        }
    );
    assert!(matches!(
        UnitDirection3::try_new(Vec3::new(0.5, 0.0, 0.0), policy),
        Err(NumericError::NormTooSmall {
            minimum_squared: 0.5,
            ..
        })
    ));
}

#[test]
fn unit_direction_rejects_zero_input() {
    assert!(matches!(
        UnitDirection3::try_new(Vec3::ZERO, NumericPolicy::STRICT),
        Err(NumericError::NormTooSmall { .. })
    ));
}

#[test]
fn unit_quaternion_normalizes_finite_nonzero_input() {
    let rotation = UnitQuaternion::try_new(Quat::from_rotation_y(0.5) * 2.0, NumericPolicy::STRICT)
        .expect("finite nonzero quaternion is valid");

    assert!((rotation.as_quat().length() - 1.0).abs() <= f32::EPSILON);
}

#[test]
fn unit_quaternion_rejects_non_finite_input() {
    assert!(matches!(
        UnitQuaternion::try_new(
            Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0),
            NumericPolicy::STRICT,
        ),
        Err(NumericError::NonFinite {
            value: NumericValue::Rotation,
        })
    ));
}

#[test]
fn validated_transform_normalizes_rotation_and_rejects_degenerate_scale() {
    let transform = Transform {
        translation: Vec3::new(3.0, -2.0, 1.0),
        rotation: Quat::from_rotation_x(0.25) * 3.0,
        scale: Vec3::new(1.0, 2.0, 3.0),
    };
    let validated = transform
        .validate(NumericPolicy::STRICT)
        .expect("finite transform with nonzero scale is valid");
    assert!((validated.as_transform().rotation.length() - 1.0).abs() <= f32::EPSILON);

    let degenerate = Transform {
        scale: Vec3::new(1.0, 0.0, 1.0),
        ..transform
    };
    assert!(matches!(
        ValidatedTransform::try_new(degenerate, NumericPolicy::STRICT),
        Err(NumericError::ScaleTooSmall { axis: Axis3::Y, .. })
    ));
}
