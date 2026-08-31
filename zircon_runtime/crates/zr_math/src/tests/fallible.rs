use crate::{
    try_affine_inverse, try_perspective, AffineInverseError, DepthDirection, LookAtError, Mat4,
    NumericPolicy, PerspectiveError, Transform, ValidatedPerspective, Vec3,
};

#[test]
fn fallible_look_at_rejects_collinear_view_and_up_axes() {
    assert!(matches!(
        Transform::try_looking_at(Vec3::ZERO, -Vec3::Z, -Vec3::Z, NumericPolicy::STRICT,),
        Err(LookAtError::CollinearAxes { .. })
    ));
}

#[test]
fn fallible_look_at_builds_a_finite_camera_transform() {
    let transform = Transform::try_looking_at(Vec3::ZERO, -Vec3::Z, Vec3::Y, NumericPolicy::STRICT)
        .expect("orthogonal finite look-at inputs are valid");

    assert!(transform.rotation.is_finite());
    assert!((transform.forward() + Vec3::Z).length() <= f32::EPSILON);
}

#[test]
fn infallible_look_at_uses_default_forward_when_eye_equals_target() {
    let eye = Vec3::new(3.0, -2.0, 1.0);
    let transform = Transform::looking_at(eye, eye, Vec3::Y);

    assert_eq!(transform.translation, eye);
    assert!(transform.rotation.is_finite());
    assert!((transform.rotation.length() - 1.0).abs() <= 4.0 * f32::EPSILON);
    assert!((transform.forward() + Vec3::Z).length() <= 4.0 * f32::EPSILON);
}

#[test]
fn infallible_look_at_builds_an_orthonormal_basis_for_collinear_up() {
    let transform = Transform::looking_at(Vec3::ZERO, -Vec3::Z, -Vec3::Z);
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();

    assert!(transform.rotation.is_finite());
    assert!((transform.rotation.length() - 1.0).abs() <= 4.0 * f32::EPSILON);
    assert!((forward + Vec3::Z).length() <= 4.0 * f32::EPSILON);
    assert!(forward.dot(right).abs() <= 4.0 * f32::EPSILON);
    assert!(forward.dot(up).abs() <= 4.0 * f32::EPSILON);
    assert!(right.dot(up).abs() <= 4.0 * f32::EPSILON);
}

#[test]
fn fallible_perspective_rejects_invalid_field_of_view_and_near_plane() {
    assert!(matches!(
        try_perspective(0.0, 1.0, 0.1, 100.0),
        Err(PerspectiveError::FieldOfViewOutOfRange)
    ));
    assert!(matches!(
        try_perspective(1.0, 1.0, 0.0, 100.0),
        Err(PerspectiveError::NearPlaneNotPositive)
    ));
}

#[test]
fn fallible_perspective_builds_a_finite_projection_matrix() {
    let projection = try_perspective(1.0, 16.0 / 9.0, 0.1, 100.0)
        .expect("finite positive perspective inputs are valid");

    assert!(projection.is_finite());
}

#[test]
fn validated_perspective_retains_a_near_to_far_projection_contract() {
    let projection = ValidatedPerspective::new(1.0, 16.0 / 9.0, 0.1, 100.0)
        .expect("finite positive perspective inputs are valid");

    assert_eq!(projection.depth_direction(), DepthDirection::NearToFar);
    assert_eq!(projection.fov_y_radians(), 1.0);
    assert_eq!(projection.aspect_ratio(), 16.0 / 9.0);
    assert_eq!(projection.z_near(), 0.1);
    assert_eq!(projection.z_far(), 100.0);
    assert_eq!(
        projection.matrix(),
        try_perspective(1.0, 16.0 / 9.0, 0.1, 100.0).unwrap()
    );
}

#[test]
fn validated_perspective_rejects_degenerate_ranges_before_exposing_a_matrix() {
    assert!(matches!(
        ValidatedPerspective::new(1.0, 1.0, 10.0, 10.0),
        Err(PerspectiveError::FarPlaneNotAfterNear)
    ));
}

#[test]
fn fallible_affine_inverse_rejects_singular_matrix() {
    let singular = Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0));

    assert!(matches!(
        try_affine_inverse(singular, NumericPolicy::STRICT),
        Err(AffineInverseError::DeterminantTooSmall { .. })
    ));
}

#[test]
fn fallible_affine_inverse_rejects_perspective_matrix() {
    let perspective = Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);

    assert!(matches!(
        try_affine_inverse(perspective, NumericPolicy::STRICT),
        Err(AffineInverseError::NonAffineInput)
    ));
}

#[test]
fn fallible_affine_inverse_round_trips_a_valid_transform_matrix() {
    let matrix = Transform::from_translation(Vec3::new(3.0, -2.0, 1.0)).matrix();
    let inverse = try_affine_inverse(matrix, NumericPolicy::STRICT)
        .expect("translation matrix is invertible");

    assert!((matrix * inverse).is_finite());
    assert!((matrix * inverse - Mat4::IDENTITY)
        .to_cols_array()
        .into_iter()
        .all(|component| component.abs() <= f32::EPSILON,));
}
