use crate::core::framework::render::{
    ProjectionMode, RenderEnvironmentCaptureRequest, ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, Real, Transform, Vec3};

const CUBEMAP_PI: Real = std::f32::consts::PI;
const CUBEMAP_TAU: Real = std::f32::consts::TAU;

const FACE_UVN: [[[Real; 3]; 3]; 6] = [
    [[0.0, 0.0, -1.0], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0]],
    [[0.0, 0.0, 1.0], [0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]],
    [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
    [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]],
    [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]],
];

/// Cubemap face order used by cmft, Unreal, and wgpu cube-array layer uploads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubemapFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubemapFace {
    pub const ALL: [Self; 6] = [
        Self::PositiveX,
        Self::NegativeX,
        Self::PositiveY,
        Self::NegativeY,
        Self::PositiveZ,
        Self::NegativeZ,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::PositiveX => 0,
            Self::NegativeX => 1,
            Self::PositiveY => 2,
            Self::NegativeY => 3,
            Self::PositiveZ => 4,
            Self::NegativeZ => 5,
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::PositiveX),
            1 => Some(Self::NegativeX),
            2 => Some(Self::PositiveY),
            3 => Some(Self::NegativeY),
            4 => Some(Self::PositiveZ),
            5 => Some(Self::NegativeZ),
            _ => None,
        }
    }

    /// Returns the D3D/cmft projection axes for this cube-array layer.
    ///
    /// These are texture axes, not a right-handed camera basis: `u` and `v`
    /// increase with texel X and Y, while `forward` points through the face
    /// center. A `-Z`-forward right-handed camera looking along `forward` with
    /// image-up `-v` has screen-right `-u`, so a scene capture must include an
    /// explicit clip-X reflection (and account for its winding reversal).
    pub const fn projection_axes(self) -> CubemapFaceProjectionAxes {
        let axes = FACE_UVN[self.index()];
        CubemapFaceProjectionAxes {
            u: axes[0],
            v: axes[1],
            forward: axes[2],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubemapFaceProjectionAxes {
    pub u: [Real; 3],
    pub v: [Real; 3],
    pub forward: [Real; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubemapCaptureView {
    pub view_from_world: Mat4,
    pub reverses_winding: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CubemapCaptureCamera {
    pub camera: ViewportCameraSnapshot,
    pub reverses_winding: bool,
}

/// Builds the reflection-aware view used to rasterize a scene into one
/// canonical cubemap layer.
///
/// Multiplying this matrix by the engine's regular 90-degree right-handed
/// projection maps increasing cubemap U to clip X and increasing cubemap V to
/// decreasing clip Y. The view has a negative determinant, so raster pipeline
/// selection must reverse its normal front-face winding.
pub fn cubemap_capture_view_from_world(
    face: CubemapFace,
    capture_origin: [Real; 3],
) -> CubemapCaptureView {
    let axes = face.projection_axes();
    let image_up = [-axes.v[0], -axes.v[1], -axes.v[2]];
    let view_from_world = Mat4::from_cols_array_2d(&[
        [axes.u[0], image_up[0], -axes.forward[0], 0.0],
        [axes.u[1], image_up[1], -axes.forward[1], 0.0],
        [axes.u[2], image_up[2], -axes.forward[2], 0.0],
        [
            -dot3(axes.u, capture_origin),
            -dot3(image_up, capture_origin),
            dot3(axes.forward, capture_origin),
            1.0,
        ],
    ]);

    CubemapCaptureView {
        view_from_world,
        reverses_winding: true,
    }
}

/// Builds one request-scoped scene camera for a canonical cubemap layer.
///
/// `Transform::looking_at` remains a regular right-handed camera transform.
/// The clip-X reflection converts its screen-right `-u` into the cmft/D3D
/// cubemap `+u` direction, while the returned flag keeps raster winding an
/// explicit pipeline concern.
pub fn cubemap_capture_camera(
    face: CubemapFace,
    request: &RenderEnvironmentCaptureRequest,
) -> CubemapCaptureCamera {
    let axes = face.projection_axes();
    let origin = Vec3::from_array(request.position());
    let forward = Vec3::from_array(axes.forward);
    let image_up = -Vec3::from_array(axes.v);
    let perspective = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_2,
        1.0,
        request.near_plane(),
        request.far_plane(),
    );
    let clip_x_reflection = Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0));
    let camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(origin, origin + forward, image_up),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: std::f32::consts::FRAC_PI_2,
        z_near: request.near_plane(),
        z_far: request.far_plane(),
        aspect_ratio: 1.0,
        projection_override: Some(clip_x_reflection * perspective),
        hdr: true,
        msaa_samples: 1,
        ..ViewportCameraSnapshot::default()
    };

    CubemapCaptureCamera {
        camera,
        reverses_winding: true,
    }
}

pub fn cubemap_face_size_from_equirect_height(equirect_height: u32) -> u32 {
    equirect_height.saturating_add(1).saturating_div(2).max(1)
}

pub fn cubemap_scaled_uv_for_texel(x: u32, y: u32, face_size: u32) -> [Real; 2] {
    let face_size = face_size.max(1);
    let max_texel = face_size.saturating_sub(1);
    [
        scaled_axis_coord(x.min(max_texel), face_size),
        scaled_axis_coord(y.min(max_texel), face_size),
    ]
}

pub fn cubemap_direction_from_scaled_uv(face: CubemapFace, scaled_uv: [Real; 2]) -> [Real; 3] {
    let axes = FACE_UVN[face.index()];
    normalize_or_positive_z([
        axes[0][0] * scaled_uv[0] + axes[1][0] * scaled_uv[1] + axes[2][0],
        axes[0][1] * scaled_uv[0] + axes[1][1] * scaled_uv[1] + axes[2][1],
        axes[0][2] * scaled_uv[0] + axes[1][2] * scaled_uv[1] + axes[2][2],
    ])
}

pub(super) fn cubemap_side_space_direction(face: CubemapFace, direction: [Real; 3]) -> [Real; 3] {
    let direction = normalize_or_positive_z(direction);
    let axes = FACE_UVN[face.index()];
    [
        dot3(axes[0], direction),
        dot3(axes[1], direction),
        dot3(axes[2], direction),
    ]
}

pub fn cubemap_texel_direction(face: CubemapFace, x: u32, y: u32, face_size: u32) -> [Real; 3] {
    cubemap_direction_from_scaled_uv(face, cubemap_scaled_uv_for_texel(x, y, face_size))
}

pub fn cubemap_face_scaled_uv_from_direction(direction: [Real; 3]) -> (CubemapFace, [Real; 2]) {
    let direction = normalize_or_positive_z(direction);
    let abs_direction = [direction[0].abs(), direction[1].abs(), direction[2].abs()];
    let (face, major_axis) =
        if abs_direction[0] >= abs_direction[1] && abs_direction[0] >= abs_direction[2] {
            if direction[0] >= 0.0 {
                (CubemapFace::PositiveX, abs_direction[0])
            } else {
                (CubemapFace::NegativeX, abs_direction[0])
            }
        } else if abs_direction[1] >= abs_direction[2] {
            if direction[1] >= 0.0 {
                (CubemapFace::PositiveY, abs_direction[1])
            } else {
                (CubemapFace::NegativeY, abs_direction[1])
            }
        } else if direction[2] >= 0.0 {
            (CubemapFace::PositiveZ, abs_direction[2])
        } else {
            (CubemapFace::NegativeZ, abs_direction[2])
        };
    let face_direction = [
        direction[0] / major_axis.max(Real::EPSILON),
        direction[1] / major_axis.max(Real::EPSILON),
        direction[2] / major_axis.max(Real::EPSILON),
    ];
    let axes = FACE_UVN[face.index()];
    (
        face,
        [dot3(axes[0], face_direction), dot3(axes[1], face_direction)],
    )
}

pub fn equirect_uv_from_direction(direction: [Real; 3]) -> [Real; 2] {
    let direction = normalize_or_positive_z(direction);
    let phi = direction[0].atan2(direction[2]);
    let theta = direction[1].clamp(-1.0, 1.0).acos();
    [(CUBEMAP_PI + phi) / CUBEMAP_TAU, theta / CUBEMAP_PI]
}

pub fn cubemap_texel_solid_angle(x: u32, y: u32, face_size: u32) -> Real {
    let face_size = face_size.max(1);
    let scaled_uv = cubemap_scaled_uv_for_texel(x, y, face_size);
    cubemap_solid_angle_from_scaled_uv(scaled_uv, 1.0 / face_size as Real)
}

pub fn cubemap_solid_angle_from_scaled_uv(scaled_uv: [Real; 2], inv_face_size: Real) -> Real {
    let x0 = scaled_uv[0] - inv_face_size;
    let x1 = scaled_uv[0] + inv_face_size;
    let y0 = scaled_uv[1] - inv_face_size;
    let y1 = scaled_uv[1] + inv_face_size;
    area_element(x1, y1) - area_element(x0, y1) - area_element(x1, y0) + area_element(x0, y0)
}

fn scaled_axis_coord(texel: u32, face_size: u32) -> Real {
    ((texel as Real + 0.5) / face_size as Real) * 2.0 - 1.0
}

fn area_element(x: Real, y: Real) -> Real {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}

fn dot3(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize_or_positive_z(direction: [Real; 3]) -> [Real; 3] {
    let len_sq =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if len_sq <= Real::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    let inv_len = 1.0 / len_sq.sqrt();
    [
        direction[0] * inv_len,
        direction[1] * inv_len,
        direction[2] * inv_len,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderEnvironmentCaptureRequest, ViewProjectionMatrixPair,
    };
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
}
