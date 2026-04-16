use zircon_math::{Mat4, Quat, Transform, Vec3};

fn assert_mat4_approx_eq(actual: Mat4, expected: Mat4) {
    let actual = actual.to_cols_array();
    let expected = expected.to_cols_array();
    for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() <= 1.0e-5,
            "matrix element {index} differed: actual={actual} expected={expected}"
        );
    }
}

#[test]
fn transform_matrix_contains_translation() {
    let transform = Transform::from_translation(Vec3::new(3.0, 2.0, 1.0));
    let matrix = transform.matrix();

    assert_eq!(matrix.w_axis.truncate(), Vec3::new(3.0, 2.0, 1.0));
}

#[test]
fn look_at_faces_target() {
    let transform = Transform::looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);

    assert!((transform.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.001);
}

#[test]
fn transform_matrix_helpers_round_trip_common_trs_paths() {
    let transform = Transform {
        translation: Vec3::new(3.0, -2.0, 1.5),
        rotation: Quat::from_rotation_y(0.7) * Quat::from_rotation_x(-0.35),
        scale: Vec3::new(2.0, 0.5, 1.25),
    };

    let helper = zircon_math::transform_to_mat4(transform);
    let inverse = zircon_math::affine_inverse(helper);

    assert_mat4_approx_eq(
        zircon_math::compose_trs(transform.translation, transform.rotation, transform.scale),
        helper,
    );
    assert_mat4_approx_eq(helper * inverse, Mat4::IDENTITY);
}
