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

/// 为跨面采样反求规范面和局部坐标；边缘过滤与捕获朝向都依赖同一套六面轴约定。
/// 输入应为可用的方向向量，退化方向按 +Z 处理。
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
#[path = "tests/cubemap_projection.rs"]
mod tests;
