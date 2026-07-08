use zircon_runtime::core::framework::render::{
    cubemap_face_size_from_equirect_height, cubemap_texel_direction, cubemap_texel_solid_angle,
    equirect_uv_from_direction, CubemapFace,
};
use zircon_runtime::core::math::Real;

const TEST_TAU: Real = std::f32::consts::TAU;

#[test]
fn runtime_environment_cubemap_projection_contract_matches_cmft_face_axes() {
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::PositiveX, 0, 0, 1),
        [1.0, 0.0, 0.0],
    );
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::NegativeX, 0, 0, 1),
        [-1.0, 0.0, 0.0],
    );
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::PositiveY, 0, 0, 1),
        [0.0, 1.0, 0.0],
    );
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::NegativeY, 0, 0, 1),
        [0.0, -1.0, 0.0],
    );
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::PositiveZ, 0, 0, 1),
        [0.0, 0.0, 1.0],
    );
    assert_vec3_close(
        cubemap_texel_direction(CubemapFace::NegativeZ, 0, 0, 1),
        [0.0, 0.0, -1.0],
    );

    let positive_x_top_left = cubemap_texel_direction(CubemapFace::PositiveX, 0, 0, 2);
    assert!(positive_x_top_left[0] > 0.0);
    assert!(positive_x_top_left[1] > 0.0);
    assert!(positive_x_top_left[2] > 0.0);
}

#[test]
fn runtime_environment_equirect_uv_contract_matches_cmft_latlong_axes() {
    assert_vec2_close(equirect_uv_from_direction([0.0, 0.0, 1.0]), [0.5, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([1.0, 0.0, 0.0]), [0.75, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([-1.0, 0.0, 0.0]), [0.25, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([0.0, 1.0, 0.0]), [0.5, 0.0]);
    assert_vec2_close(equirect_uv_from_direction([0.0, -1.0, 0.0]), [0.5, 1.0]);
}

#[test]
fn runtime_environment_cubemap_projection_contract_preserves_unit_sphere_area() {
    let face_size = 16;
    let mut total = 0.0;
    for face in CubemapFace::ALL {
        assert_eq!(CubemapFace::from_index(face.index()), Some(face));
        for y in 0..face_size {
            for x in 0..face_size {
                total += cubemap_texel_solid_angle(x, y, face_size);
            }
        }
    }

    assert!(
        (total - TEST_TAU * 2.0).abs() <= 0.0001,
        "cubemap texel solid angle sum {total}"
    );
}

#[test]
fn runtime_environment_cubemap_projection_contract_uses_cmft_face_size_rule() {
    assert_eq!(cubemap_face_size_from_equirect_height(512), 256);
    assert_eq!(cubemap_face_size_from_equirect_height(513), 257);
    assert_eq!(cubemap_face_size_from_equirect_height(0), 1);
}

fn assert_vec2_close(actual: [Real; 2], expected: [Real; 2]) {
    for index in 0..2 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.00001,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}

fn assert_vec3_close(actual: [Real; 3], expected: [Real; 3]) {
    for index in 0..3 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.00001,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}
