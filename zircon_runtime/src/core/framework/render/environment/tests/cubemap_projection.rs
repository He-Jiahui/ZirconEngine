use super::*;
use crate::core::framework::render::{RenderEnvironmentCaptureRequest, ViewProjectionMatrixPair};
use crate::core::math::{UVec2, Vec3};

#[test]
fn render_env_equirect_to_cube_golden_directions() {
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
fn equirect_uv_from_direction_matches_cmft_latlong_axes() {
    assert_vec2_close(equirect_uv_from_direction([0.0, 0.0, 1.0]), [0.5, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([1.0, 0.0, 0.0]), [0.75, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([-1.0, 0.0, 0.0]), [0.25, 0.5]);
    assert_vec2_close(equirect_uv_from_direction([0.0, 1.0, 0.0]), [0.5, 0.0]);
    assert_vec2_close(equirect_uv_from_direction([0.0, -1.0, 0.0]), [0.5, 1.0]);
}

#[test]
fn cubemap_face_scaled_uv_roundtrips_face_centers() {
    for face in CubemapFace::ALL {
        let direction = cubemap_texel_direction(face, 0, 0, 1);
        let (actual_face, scaled_uv) = cubemap_face_scaled_uv_from_direction(direction);

        assert_eq!(actual_face, face);
        assert_vec2_close(scaled_uv, [0.0, 0.0]);
    }
}

#[test]
fn cubemap_projection_axes_match_face_texel_directions() {
    for face in CubemapFace::ALL {
        let axes = face.projection_axes();
        assert_vec3_close(axes.forward, cubemap_texel_direction(face, 0, 0, 1));
        assert!((dot3(axes.u, axes.v)).abs() <= 0.00001);
        assert!((dot3(axes.u, axes.forward)).abs() <= 0.00001);
        assert!((dot3(axes.v, axes.forward)).abs() <= 0.00001);
        assert_vec3_close(cross3(axes.u, axes.v), negate3(axes.forward));
        assert_vec3_close(cross3(axes.forward, negate3(axes.v)), negate3(axes.u));
        assert!((length_squared(axes.u) - 1.0).abs() <= 0.00001);
        assert!((length_squared(axes.v) - 1.0).abs() <= 0.00001);
        assert!((length_squared(axes.forward) - 1.0).abs() <= 0.00001);
    }
}

#[test]
fn cubemap_capture_view_maps_texture_axes_and_reports_winding_reflection() {
    let origin = [3.0, -2.0, 5.0];
    let origin_vec = Vec3::from_array(origin);

    for face in CubemapFace::ALL {
        let axes = face.projection_axes();
        let view = cubemap_capture_view_from_world(face, origin);
        let forward = Vec3::from_array(axes.forward);

        assert!(view.reverses_winding);
        assert_vec3_close(
            view.view_from_world
                .transform_point3(origin_vec + forward)
                .to_array(),
            [0.0, 0.0, -1.0],
        );
        assert_vec3_close(
            view.view_from_world
                .transform_point3(origin_vec + forward + Vec3::from_array(axes.u))
                .to_array(),
            [1.0, 0.0, -1.0],
        );
        assert_vec3_close(
            view.view_from_world
                .transform_point3(origin_vec + forward + Vec3::from_array(axes.v))
                .to_array(),
            [0.0, -1.0, -1.0],
        );
        assert!((view.view_from_world.determinant() + 1.0).abs() <= 0.00001);
    }
}

#[test]
fn cubemap_capture_camera_matches_canonical_face_projection() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [3.0, -2.0, 5.0], 1)
        .unwrap()
        .with_clip_planes(0.25, 320.0)
        .unwrap()
        .with_face_size(256)
        .unwrap();
    let origin = Vec3::from_array(request.position());

    for face in CubemapFace::ALL {
        let axes = face.projection_axes();
        let capture = cubemap_capture_camera(face, &request);
        let pair = ViewProjectionMatrixPair::from_camera(
            &capture.camera,
            UVec2::splat(request.face_size()),
        );
        let forward = Vec3::from_array(axes.forward);

        assert!(capture.reverses_winding);
        assert!(capture.camera.hdr);
        assert_eq!(capture.camera.msaa_samples, 1);
        assert_eq!(capture.camera.aspect_ratio, 1.0);
        assert_eq!(capture.camera.z_near, request.near_plane());
        assert_eq!(capture.camera.z_far, request.far_plane());
        assert_vec3_close(
            pair.clip_from_world_unjittered
                .project_point3(origin + forward)
                .to_array(),
            [0.0, 0.0, expected_projected_depth(&request)],
        );
        assert_vec2_close(
            pair.clip_from_world_unjittered
                .project_point3(origin + forward + Vec3::from_array(axes.u))
                .truncate()
                .to_array(),
            [1.0, 0.0],
        );
        assert_vec2_close(
            pair.clip_from_world_unjittered
                .project_point3(origin + forward + Vec3::from_array(axes.v))
                .truncate()
                .to_array(),
            [0.0, -1.0],
        );

        let reflected = cubemap_capture_view_from_world(face, request.position());
        let expected = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_2,
            1.0,
            request.near_plane(),
            request.far_plane(),
        ) * reflected.view_from_world;
        assert_mat4_close(pair.clip_from_world_unjittered, expected);
    }
}

#[test]
fn cubemap_texel_solid_angles_cover_unit_sphere() {
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
        (total - CUBEMAP_TAU * 2.0).abs() <= 0.0001,
        "cubemap texel solid angle sum {total}"
    );
}

#[test]
fn cubemap_face_size_from_equirect_height_matches_cmft_hemisphere_rule() {
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

fn assert_mat4_close(actual: Mat4, expected: Mat4) {
    let actual = actual.to_cols_array();
    let expected = expected.to_cols_array();
    for index in 0..16 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.00001,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}

fn expected_projected_depth(request: &RenderEnvironmentCaptureRequest) -> Real {
    Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_2,
        1.0,
        request.near_plane(),
        request.far_plane(),
    )
    .project_point3(Vec3::NEG_Z)
    .z
}

fn dot3(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length_squared(value: [Real; 3]) -> Real {
    dot3(value, value)
}

fn cross3(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn negate3(value: [Real; 3]) -> [Real; 3] {
    [-value[0], -value[1], -value[2]]
}
