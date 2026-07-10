use super::{
    normalize_or_positive_z, sample_source_cubemap_trilinear, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, source_cubemap_roughness_from_pmrem_mip, tangent_basis,
    SourceCubemapPrefilterQuality,
};
use crate::core::framework::render::environment::{cubemap_texel_direction, CubemapFace};
use crate::core::math::Real;

const CMFT_FILTER_THRESHOLD: Real = 0.00001;
const CMFT_GLOSS_SCALE: Real = 10.0;
const CMFT_GLOSS_BIAS: Real = 3.0;
pub(super) fn prefilter_pmrem_mips_from_source(
    texels: &mut [[Real; 4]],
    source_mips: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
) {
    if mip_count <= 1 {
        return;
    }

    for mip in 1..mip_count {
        let mip_size = source_cubemap_mip_size(face_size, mip);
        let filter = CmftRadianceFilter::new(mip, mip_count, face_size, quality);
        for face in CubemapFace::ALL {
            let dest_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let direction = cubemap_texel_direction(face, x, y, mip_size);
                    texels[dest_offset + y as usize * mip_size as usize + x as usize] =
                        cmft_prefilter_direction(
                            source_mips,
                            face_size,
                            mip_count,
                            direction,
                            filter,
                        );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CmftRadianceFilter {
    roughness: Real,
    specular_power: Real,
    cos_angle: Real,
    sample_count: u32,
}

impl CmftRadianceFilter {
    fn new(
        mip: u32,
        mip_count: u32,
        source_face_size: u32,
        quality: SourceCubemapPrefilterQuality,
    ) -> Self {
        let mip_size = source_cubemap_mip_size(source_face_size, mip);
        let mip_size_f = mip_size.max(1) as Real;
        let specular_power =
            cmft_apply_blinn_brdf(cmft_specular_power_for_mip(mip as Real, mip_count as Real));
        let min_angle = 1.0_f32.atan2(mip_size_f);
        let max_angle = std::f32::consts::FRAC_PI_2;
        let filter_angle =
            cmft_cosine_power_filter_angle(specular_power).clamp(min_angle, max_angle);
        let roughness = source_cubemap_roughness_from_pmrem_mip(mip, mip_count);

        Self {
            roughness,
            specular_power,
            cos_angle: filter_angle.cos().max(0.0),
            sample_count: cmft_sample_count_for_mip(mip, mip_count, quality),
        }
    }
}

fn cmft_prefilter_direction(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [Real; 3],
    filter: CmftRadianceFilter,
) -> [Real; 4] {
    if filter.roughness <= Real::EPSILON {
        return sample_source_cubemap_trilinear(texels, face_size, mip_count, direction, 0.0);
    }

    let direction = normalize_or_positive_z(direction);
    let basis = tangent_basis(direction);
    let mut color = [0.0; 4];
    let mut weight_sum = 0.0;

    for sample_index in 0..filter.sample_count {
        let xi = hammersley(sample_index, filter.sample_count);
        let local_direction = uniform_sample_cone(xi, filter.cos_angle);
        let sample_direction = tangent_to_world(local_direction, basis);
        let dp = dot3(direction, sample_direction).max(0.0);
        if dp < filter.cos_angle {
            continue;
        }

        // cmft radianceFilter/processFilterArea integrates pow(dot, power) over cubemap texel
        // solid angles. Hammersley cone samples provide the same solid-angle domain without
        // scanning every source texel for each destination texel.
        let weight = dp.powf(filter.specular_power);
        if weight <= Real::EPSILON {
            continue;
        }

        let radiance =
            sample_source_cubemap_trilinear(texels, face_size, mip_count, sample_direction, 0.0);
        color[0] += radiance[0] * weight;
        color[1] += radiance[1] * weight;
        color[2] += radiance[2] * weight;
        color[3] += radiance[3] * weight;
        weight_sum += weight;
    }

    if weight_sum <= Real::EPSILON {
        return sample_source_cubemap_trilinear(texels, face_size, mip_count, direction, 0.0);
    }

    let inv_weight = 1.0 / weight_sum;
    [
        color[0] * inv_weight,
        color[1] * inv_weight,
        color[2] * inv_weight,
        color[3] * inv_weight,
    ]
}

fn cmft_specular_power_for_mip(mip: Real, mip_count: Real) -> Real {
    let glossiness = (1.0 - mip / (mip_count - (1.0 + 0.0000001))).max(0.0);
    2.0_f32.powf(CMFT_GLOSS_SCALE * glossiness + CMFT_GLOSS_BIAS)
}

fn cmft_apply_blinn_brdf(specular_power: Real) -> Real {
    specular_power / 4.0 + 1.0
}

fn cmft_cosine_power_filter_angle(specular_power: Real) -> Real {
    CMFT_FILTER_THRESHOLD
        .powf(1.0 / specular_power.max(Real::EPSILON))
        .acos()
}

fn cmft_sample_count_for_mip(
    mip: u32,
    mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
) -> u32 {
    let terminal_mip = mip.saturating_add(2) >= mip_count.max(1);
    match (quality, terminal_mip) {
        (SourceCubemapPrefilterQuality::Fast, false) => 32,
        (SourceCubemapPrefilterQuality::Fast, true) => 64,
        (SourceCubemapPrefilterQuality::Normal, false) => 64,
        (SourceCubemapPrefilterQuality::Normal, true) => 128,
        (SourceCubemapPrefilterQuality::High, false) => 128,
        (SourceCubemapPrefilterQuality::High, true) => 256,
    }
}

fn uniform_sample_cone(xi: [Real; 2], cos_angle: Real) -> [Real; 3] {
    let cos_theta = 1.0 - xi[1].clamp(0.0, 1.0) * (1.0 - cos_angle);
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let phi = std::f32::consts::TAU * xi[0];
    [phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta]
}

fn tangent_to_world(local: [Real; 3], basis: [[Real; 3]; 3]) -> [Real; 3] {
    normalize_or_positive_z([
        basis[0][0] * local[0] + basis[1][0] * local[1] + basis[2][0] * local[2],
        basis[0][1] * local[0] + basis[1][1] * local[1] + basis[2][1] * local[2],
        basis[0][2] * local[0] + basis[1][2] * local[1] + basis[2][2] * local[2],
    ])
}

fn hammersley(index: u32, sample_count: u32) -> [Real; 2] {
    [
        (index as Real + 0.5) / sample_count.max(1) as Real,
        radical_inverse_vdc(index),
    ]
}

fn radical_inverse_vdc(mut bits: u32) -> Real {
    bits = bits.rotate_right(16);
    bits = ((bits & 0x5555_5555) << 1) | ((bits & 0xAAAA_AAAA) >> 1);
    bits = ((bits & 0x3333_3333) << 2) | ((bits & 0xCCCC_CCCC) >> 2);
    bits = ((bits & 0x0F0F_0F0F) << 4) | ((bits & 0xF0F0_F0F0) >> 4);
    bits = ((bits & 0x00FF_00FF) << 8) | ((bits & 0xFF00_FF00) >> 8);
    bits as Real * 2.328_306_4e-10
}

fn dot3(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::{cmft_sample_count_for_mip, SourceCubemapPrefilterQuality};

    #[test]
    fn cmft_prefilter_quality_matches_planned_mid_and_terminal_mip_budgets() {
        assert_eq!(
            cmft_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::Fast),
            32
        );
        assert_eq!(
            cmft_sample_count_for_mip(6, 8, SourceCubemapPrefilterQuality::Fast),
            64
        );
        assert_eq!(
            cmft_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::Normal),
            64
        );
        assert_eq!(
            cmft_sample_count_for_mip(7, 8, SourceCubemapPrefilterQuality::Normal),
            128
        );
        assert_eq!(
            cmft_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::High),
            128
        );
        assert_eq!(
            cmft_sample_count_for_mip(7, 8, SourceCubemapPrefilterQuality::High),
            256
        );
    }
}
