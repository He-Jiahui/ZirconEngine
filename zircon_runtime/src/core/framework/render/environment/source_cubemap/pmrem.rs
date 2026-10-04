use super::super::ibl_bake_recipe::CANONICAL_IBL_BAKE_RECIPE;
use super::sampling::{normalize_or_positive_z, sample_source_cubemap_trilinear};
use super::{
    source_cubemap_face_mip_outputs, source_cubemap_mip_size,
    source_cubemap_roughness_from_pmrem_mip, CubemapFaceMipOutput, SourceCubemapPrefilterQuality,
};
use crate::core::framework::render::environment::{cubemap_texel_direction, CubemapFace};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;

trait PmremFaceExecutor {
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync;
}

struct SerialPmremFaceExecutor;

impl PmremFaceExecutor for SerialPmremFaceExecutor {
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync,
    {
        for output in faces {
            filter_face(output.face, &mut *output.texels);
        }
    }
}

struct ParallelPmremFaceExecutor<'a, E>(&'a E);

impl<E> PmremFaceExecutor for ParallelPmremFaceExecutor<'_, E>
where
    E: ParallelSliceExecutor,
{
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync,
    {
        self.0.parallel_for(faces, 1, |chunk| {
            for output in chunk {
                filter_face(output.face, &mut *output.texels);
            }
        });
    }
}

/// 根据源金字塔构造独立目标布局的预滤反射链，供不同粗糙度的 PBR 采样。
/// 这是导入或重建阶段的 CPU 路径，运行时 GPU 烘焙使用自己的生产者身份。
pub(super) fn prefilter_pmrem_mips_from_source(
    pmrem_texels: &mut [[Real; 4]],
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    source_mips: &[[Real; 4]],
    source_face_size: u32,
    source_mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
) {
    prefilter_pmrem_mips_from_source_with_face_executor(
        pmrem_texels,
        pmrem_face_size,
        pmrem_mip_count,
        source_mips,
        source_face_size,
        source_mip_count,
        quality,
        &SerialPmremFaceExecutor,
    );
}

pub(super) fn prefilter_pmrem_mips_from_source_with_parallel_executor<E>(
    pmrem_texels: &mut [[Real; 4]],
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    source_mips: &[[Real; 4]],
    source_face_size: u32,
    source_mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
) where
    E: ParallelSliceExecutor,
{
    prefilter_pmrem_mips_from_source_with_face_executor(
        pmrem_texels,
        pmrem_face_size,
        pmrem_mip_count,
        source_mips,
        source_face_size,
        source_mip_count,
        quality,
        &ParallelPmremFaceExecutor(parallel_executor),
    );
}

fn prefilter_pmrem_mips_from_source_with_face_executor(
    pmrem_texels: &mut [[Real; 4]],
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    source_mips: &[[Real; 4]],
    source_face_size: u32,
    source_mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
    face_executor: &impl PmremFaceExecutor,
) {
    for mip in 0..pmrem_mip_count.max(1) {
        let mip_size = source_cubemap_mip_size(pmrem_face_size, mip);
        let filter = GgxRadianceFilter::new(mip, pmrem_mip_count, mip_size, quality);
        let filter_face = |face, face_texels: &mut [[Real; 4]]| {
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let direction = cubemap_texel_direction(face, x, y, mip_size);
                    face_texels[y as usize * mip_size as usize + x as usize] =
                        ggx_prefilter_direction(
                            source_mips,
                            source_face_size,
                            source_mip_count,
                            direction,
                            filter,
                        );
                }
            }
        };
        let mut outputs =
            source_cubemap_face_mip_outputs(pmrem_texels, pmrem_face_size, pmrem_mip_count, mip);
        face_executor.filter_faces(&mut outputs, &filter_face);
    }
}

#[derive(Clone, Copy, Debug)]
struct GgxRadianceFilter {
    roughness: Real,
    destination_face_size: u32,
    sample_count: u32,
}

impl GgxRadianceFilter {
    fn new(
        mip: u32,
        mip_count: u32,
        destination_face_size: u32,
        quality: SourceCubemapPrefilterQuality,
    ) -> Self {
        let roughness = source_cubemap_roughness_from_pmrem_mip(mip, mip_count);
        Self {
            roughness,
            destination_face_size,
            sample_count: ggx_sample_count_for_mip(mip, mip_count, quality),
        }
    }
}

fn ggx_prefilter_direction(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [Real; 3],
    filter: GgxRadianceFilter,
) -> [Real; 4] {
    let direction = normalize_or_positive_z(direction);
    let footprint_lod = source_footprint_lod(face_size, mip_count, filter.destination_face_size);
    if filter.roughness <= Real::EPSILON {
        return sample_source_cubemap_trilinear(
            texels,
            face_size,
            mip_count,
            direction,
            footprint_lod,
        );
    }

    if filter.roughness >= CANONICAL_IBL_BAKE_RECIPE.full_roughness_cosine_threshold() {
        return cosine_prefilter_direction(
            texels,
            face_size,
            mip_count,
            direction,
            filter.sample_count,
        );
    }

    let basis = tangent_basis(direction);
    let mut color = [0.0; 4];
    let mut weight_sum = 0.0;
    for sample_index in 0..filter.sample_count {
        let xi = hammersley(sample_index, filter.sample_count);
        let half_vector = tangent_to_world(importance_sample_ggx(xi, filter.roughness), basis);
        let light_direction = normalize_or_positive_z(sub3(
            scale3(half_vector, 2.0 * dot3(direction, half_vector)),
            direction,
        ));
        let no_l = dot3(direction, light_direction).max(0.0);
        if no_l <= 0.0 {
            continue;
        }

        let no_h = dot3(direction, half_vector).max(0.0);
        let pdf = ggx_light_direction_pdf(no_h, filter.roughness).max(0.000001);
        let source_lod = source_lod_for_pdf(face_size, mip_count, pdf, filter.sample_count);
        let radiance = sample_source_cubemap_trilinear(
            texels,
            face_size,
            mip_count,
            light_direction,
            source_lod,
        );
        for channel in 0..4 {
            color[channel] += radiance[channel] * no_l;
        }
        weight_sum += no_l;
    }

    normalize_weighted_color(color, weight_sum, || {
        sample_source_cubemap_trilinear(texels, face_size, mip_count, direction, 0.0)
    })
}

fn cosine_prefilter_direction(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [Real; 3],
    sample_count: u32,
) -> [Real; 4] {
    let basis = tangent_basis(direction);
    let mut color = [0.0; 4];
    for sample_index in 0..sample_count {
        let local_direction = cosine_sample_hemisphere(hammersley(sample_index, sample_count));
        let pdf = (local_direction[2] / std::f32::consts::PI).max(0.000001);
        let source_lod = source_lod_for_pdf(face_size, mip_count, pdf, sample_count);
        let radiance = sample_source_cubemap_trilinear(
            texels,
            face_size,
            mip_count,
            tangent_to_world(local_direction, basis),
            source_lod,
        );
        for channel in 0..4 {
            color[channel] += radiance[channel];
        }
    }

    let inv_sample_count = 1.0 / sample_count.max(1) as Real;
    for channel in &mut color {
        *channel *= inv_sample_count;
    }
    color
}

fn importance_sample_ggx(xi: [Real; 2], roughness: Real) -> [Real; 3] {
    let alpha = (roughness * roughness).max(0.0001);
    let alpha2 = alpha * alpha;
    let e_y = (xi[1] * 0.995).clamp(0.0, 0.99999);
    let phi = std::f32::consts::TAU * xi[0];
    let cos_theta = ((1.0 - e_y) / (1.0 + (alpha2 - 1.0) * e_y).max(0.0001)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    [sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta]
}

fn distribution_ggx(no_h: Real, roughness: Real) -> Real {
    let alpha = (roughness * roughness).max(0.0001);
    let alpha2 = alpha * alpha;
    let clamped_no_h = no_h.clamp(0.0, 1.0);
    let no_h_squared = clamped_no_h * clamped_no_h;
    let denominator = (1.0 - no_h_squared) + no_h_squared * alpha2;
    alpha2 / (std::f32::consts::PI * denominator * denominator)
}

fn ggx_light_direction_pdf(no_h: Real, roughness: Real) -> Real {
    distribution_ggx(no_h, roughness) * 0.25
}

fn source_footprint_lod(
    source_face_size: u32,
    source_mip_count: u32,
    destination_face_size: u32,
) -> Real {
    let source_max_mip = source_mip_count.max(1).saturating_sub(1) as Real;
    ((source_face_size.max(1) as Real / destination_face_size.max(1) as Real).log2())
        .clamp(0.0, source_max_mip)
}

fn source_lod_for_pdf(
    source_face_size: u32,
    source_mip_count: u32,
    pdf: Real,
    sample_count: u32,
) -> Real {
    let source_face_size = source_face_size.max(1) as Real;
    let texel_solid_angle = 4.0 * std::f32::consts::PI
        / (6.0 * source_face_size * source_face_size)
        * CANONICAL_IBL_BAKE_RECIPE.fis_solid_angle_texel_scale();
    let sample_solid_angle = 1.0 / (sample_count.max(1) as Real * pdf);
    let lod = 0.5 * (sample_solid_angle / texel_solid_angle).max(1.0).log2();
    lod.clamp(0.0, source_mip_count.max(1).saturating_sub(1) as Real)
}

fn ggx_sample_count_for_mip(
    mip: u32,
    mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
) -> u32 {
    let roughness = source_cubemap_roughness_from_pmrem_mip(mip, mip_count);
    let normal_sample_count = CANONICAL_IBL_BAKE_RECIPE.pmrem_sample_count(roughness, mip);
    match quality {
        SourceCubemapPrefilterQuality::Fast => (normal_sample_count / 2).max(16),
        SourceCubemapPrefilterQuality::Normal => normal_sample_count,
        SourceCubemapPrefilterQuality::High => normal_sample_count * 2,
    }
}

fn cosine_sample_hemisphere(xi: [Real; 2]) -> [Real; 3] {
    let radius = xi[0].sqrt();
    let phi = std::f32::consts::TAU * xi[1];
    [
        phi.cos() * radius,
        phi.sin() * radius,
        (1.0 - xi[0]).max(0.0).sqrt(),
    ]
}

fn tangent_basis(direction: [Real; 3]) -> [[Real; 3]; 3] {
    let normal = normalize_or_positive_z(direction);
    let up = if normal[2].abs() > 0.999 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let tangent = normalize_or_positive_z(cross3(up, normal));
    let bitangent = cross3(normal, tangent);
    [tangent, bitangent, normal]
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

fn normalize_weighted_color<F>(color: [Real; 4], weight_sum: Real, fallback: F) -> [Real; 4]
where
    F: FnOnce() -> [Real; 4],
{
    if weight_sum <= Real::EPSILON {
        return fallback();
    }
    let inv_weight = 1.0 / weight_sum;
    [
        color[0] * inv_weight,
        color[1] * inv_weight,
        color[2] * inv_weight,
        color[3] * inv_weight,
    ]
}

fn scale3(value: [Real; 3], scale: Real) -> [Real; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn sub3(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
#[path = "tests/pmrem.rs"]
mod tests;
