use crate::core::framework::render::environment::{
    cubemap_texel_direction, cubemap_texel_solid_angle, CubemapFace,
};
use crate::core::math::Real;

use super::sampling::{mip_texel, normalize_or_positive_z};
use super::{
    source_cubemap_irradiance_mip_level, source_cubemap_mip_size, SourceCubemapMipChain,
    SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
};

pub type SourceCubemapIrradianceSh9 = [[Real; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];

impl SourceCubemapMipChain {
    pub(in crate::core::framework::render::environment) fn source_irradiance_sh9_from_source_texels(
        source_texels: &[[Real; 4]],
        source_face_size: u32,
        source_mip_count: u32,
    ) -> SourceCubemapIrradianceSh9 {
        let source_face_size = source_face_size.max(1);
        let source_mip_count =
            source_mip_count.clamp(1, super::source_cubemap_mip_count(source_face_size));
        assert_eq!(
            source_texels.len(),
            super::source_cubemap_sample_count(source_face_size, source_mip_count),
            "source cubemap texel count must match its source layout"
        );
        source_cubemap_irradiance_sh9_from_texels(
            source_texels,
            source_face_size,
            source_mip_count,
            source_cubemap_irradiance_mip_level(source_face_size, source_mip_count),
        )
    }
}

pub fn source_cubemap_evaluate_irradiance_sh9(
    coefficients: &SourceCubemapIrradianceSh9,
    normal: [Real; 3],
) -> [Real; 3] {
    let basis = sh9_basis_y_up(normalize_or_positive_z(normal));
    let mut irradiance = [0.0; 3];
    for coefficient_index in 0..SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT {
        irradiance[0] += coefficients[coefficient_index][0] * basis[coefficient_index];
        irradiance[1] += coefficients[coefficient_index][1] * basis[coefficient_index];
        irradiance[2] += coefficients[coefficient_index][2] * basis[coefficient_index];
    }
    [
        irradiance[0].max(0.0),
        irradiance[1].max(0.0),
        irradiance[2].max(0.0),
    ]
}

pub(super) fn source_cubemap_irradiance_sh9_from_texels(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> SourceCubemapIrradianceSh9 {
    let mip_level = mip_level.min(mip_count.max(1).saturating_sub(1));
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let mut coefficients = [[0.0; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];
    let mut solid_angle_sum = 0.0;

    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_texel_direction(face, x, y, mip_size);
                let solid_angle = cubemap_texel_solid_angle(x, y, mip_size);
                let texel = mip_texel(texels, face_size, mip_count, face, mip_level, x, y);
                let basis = sh9_basis_y_up(direction);
                solid_angle_sum += solid_angle;
                for coefficient_index in 0..SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT {
                    let weighted_basis = basis[coefficient_index] * solid_angle;
                    coefficients[coefficient_index][0] += texel[0] * weighted_basis;
                    coefficients[coefficient_index][1] += texel[1] * weighted_basis;
                    coefficients[coefficient_index][2] += texel[2] * weighted_basis;
                }
            }
        }
    }

    let normalization = std::f32::consts::TAU * 2.0 / solid_angle_sum.max(Real::EPSILON);
    for (coefficient_index, coefficient) in coefficients.iter_mut().enumerate() {
        let band_scale = sh9_cosine_lobe_scale(coefficient_index);
        coefficient[0] *= normalization * band_scale;
        coefficient[1] *= normalization * band_scale;
        coefficient[2] *= normalization * band_scale;
    }

    coefficients
}

fn sh9_basis_y_up(direction: [Real; 3]) -> [Real; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT] {
    // This is Zircon's serialized Y-up basis order. It is intentionally not
    // cmft's legacy coefficient ordering; the payload codec and WGSL evaluator
    // must remain in lockstep with this order.
    let x = direction[0];
    let y = direction[1];
    let z = direction[2];
    [
        0.282_094_8,
        0.488_602_52 * z,
        0.488_602_52 * y,
        0.488_602_52 * x,
        1.092_548_5 * x * z,
        1.092_548_5 * z * y,
        0.315_391_57 * (3.0 * y * y - 1.0),
        1.092_548_5 * x * y,
        0.546_274_24 * (x * x - z * z),
    ]
}

fn sh9_cosine_lobe_scale(coefficient_index: usize) -> Real {
    match coefficient_index {
        0 => 1.0,
        1..=3 => 2.0 / 3.0,
        _ => 0.25,
    }
}
