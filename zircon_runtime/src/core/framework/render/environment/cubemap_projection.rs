use crate::core::math::Real;

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
}
