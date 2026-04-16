use zircon_math::{
    affine_inverse, compose_trs, is_finite_mat4, is_finite_vec3, to_render_mat4, to_render_scalar,
    to_render_vec3, transform_to_mat4, Mat4, Quat, Real, RenderScalar, Transform, Vec3,
};

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
fn precision_contract_is_currently_f32_backed() {
    let _: Real = 1.0;
    let _: RenderScalar = 1.0;

    assert_eq!(std::mem::size_of::<Real>(), 4);
    assert_eq!(std::mem::size_of::<RenderScalar>(), 4);
}

#[test]
fn compose_trs_matches_transform_matrix_helper() {
    let transform = Transform {
        translation: Vec3::new(3.0, -2.0, 1.5),
        rotation: Quat::from_rotation_y(0.7) * Quat::from_rotation_x(-0.35),
        scale: Vec3::new(2.0, 0.5, 1.25),
    };

    let composed = compose_trs(transform.translation, transform.rotation, transform.scale);
    let helper = transform_to_mat4(transform);

    assert_mat4_approx_eq(composed, helper);
    assert_mat4_approx_eq(helper, transform.matrix());
}

#[test]
fn parent_world_multiplies_local_matrix_in_runtime_order() {
    let parent = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
    let local = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));

    let world = transform_to_mat4(parent) * transform_to_mat4(local);

    assert_eq!(world.transform_point3(Vec3::ZERO), Vec3::new(7.0, 0.0, 0.0));
}

#[test]
fn affine_inverse_round_trips_affine_matrices() {
    let transform = Transform {
        translation: Vec3::new(-4.0, 1.5, 8.0),
        rotation: Quat::from_rotation_z(0.45) * Quat::from_rotation_x(0.2),
        scale: Vec3::new(1.5, 2.0, 0.75),
    };
    let matrix = transform_to_mat4(transform);
    let inverse = affine_inverse(matrix);
    let identity = matrix * inverse;

    assert_mat4_approx_eq(identity, Mat4::IDENTITY);
}

#[test]
fn finite_checks_and_render_conversions_reject_non_finite_values() {
    let finite = Vec3::new(1.0, 2.0, 3.0);
    let invalid = Vec3::new(1.0, Real::NAN, 3.0);

    assert!(is_finite_vec3(finite));
    assert!(!is_finite_vec3(invalid));
    assert!(is_finite_mat4(Mat4::IDENTITY));

    assert_eq!(to_render_scalar(1.25), Some(1.25));
    assert_eq!(to_render_vec3(finite), Some(finite));
    assert_eq!(to_render_mat4(Mat4::IDENTITY), Some(Mat4::IDENTITY));

    assert_eq!(to_render_scalar(Real::NAN), None);
    assert_eq!(to_render_vec3(invalid), None);
    assert_eq!(
        to_render_mat4(Mat4::from_cols(
            Vec3::X.extend(0.0),
            Vec3::Y.extend(0.0),
            Vec3::Z.extend(0.0),
            Vec3::new(0.0, Real::INFINITY, 0.0).extend(1.0),
        )),
        None
    );
}
