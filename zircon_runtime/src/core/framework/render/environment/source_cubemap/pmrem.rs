use super::super::ibl_bake_recipe::CANONICAL_IBL_BAKE_RECIPE;
use super::{
    normalize_or_positive_z, sample_source_cubemap_trilinear, source_cubemap_face_mip_outputs,
    source_cubemap_mip_size, source_cubemap_roughness_from_pmrem_mip, CubemapFaceMipOutput,
    SourceCubemapPrefilterQuality,
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
    let denominator = (no_h * no_h * (alpha2 - 1.0) + 1.0).max(0.0001);
    alpha2 / (std::f32::consts::PI * denominator * denominator).max(0.000001)
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
mod tests {
    use super::{
        distribution_ggx, ggx_light_direction_pdf, ggx_sample_count_for_mip,
        prefilter_pmrem_mips_from_source, prefilter_pmrem_mips_from_source_with_parallel_executor,
        source_lod_for_pdf, SourceCubemapPrefilterQuality,
    };
    use crate::core::framework::tasks::ParallelSliceExecutor;
    use crate::core::math::Real;

    #[derive(Default)]
    struct CountingParallelSliceExecutor(std::sync::atomic::AtomicUsize);

    impl ParallelSliceExecutor for CountingParallelSliceExecutor {
        fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
        where
            T: Send,
            F: Fn(&mut [T]) + Send + Sync,
        {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            for chunk in items.chunks_mut(chunk_size.max(1)) {
                task(chunk);
            }
        }
    }

    #[test]
    fn ggx_light_direction_pdf_matches_unreal_v_equals_n_reduction() {
        let no_h = 0.5;
        let roughness = 0.45;
        let expected = distribution_ggx(no_h, roughness) * 0.25;

        assert!((ggx_light_direction_pdf(no_h, roughness) - expected).abs() <= 0.000001);
    }

    #[test]
    fn filtered_importance_source_lod_does_not_apply_destination_footprint_floor() {
        let lod = source_lod_for_pdf(512, 10, 1024.0, 64);

        assert!(
            lod.abs() <= 0.000001,
            "UE PDF-selected source LOD should remain at mip 0, got {lod}"
        );
    }

    #[test]
    fn ggx_prefilter_quality_matches_gpu_normal_and_scaled_quality_budgets() {
        assert_eq!(
            ggx_sample_count_for_mip(1, 8, SourceCubemapPrefilterQuality::Normal),
            32
        );
        assert_eq!(
            ggx_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::Fast),
            32
        );
        assert_eq!(
            ggx_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::Normal),
            64
        );
        assert_eq!(
            ggx_sample_count_for_mip(7, 8, SourceCubemapPrefilterQuality::Normal),
            128
        );
        assert_eq!(
            ggx_sample_count_for_mip(3, 8, SourceCubemapPrefilterQuality::High),
            128
        );
        assert_eq!(
            ggx_sample_count_for_mip(7, 8, SourceCubemapPrefilterQuality::High),
            256
        );
    }

    #[test]
    fn pmrem_face_mip_outputs_write_directly_to_each_face_final_storage() {
        let face_size = 4;
        let mip_count = super::super::source_cubemap_mip_count(face_size);
        let mip = 1;
        let mip_size = super::super::source_cubemap_mip_size(face_size, mip);
        let mut texels = vec![
            [-1.0 as Real; 4];
            super::super::source_cubemap_sample_count(face_size, mip_count)
        ];
        let mut outputs =
            super::super::source_cubemap_face_mip_outputs(&mut texels, face_size, mip_count, mip);

        assert_eq!(outputs.len(), super::super::SOURCE_CUBEMAP_FACE_COUNT);
        for output in &mut outputs {
            output.texels.fill([output.face.index() as Real; 4]);
        }
        drop(outputs);

        for face in super::CubemapFace::ALL {
            let mip_offset =
                super::super::source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
            let mip_len = mip_size as usize * mip_size as usize;
            assert_eq!(
                &texels[mip_offset..mip_offset + mip_len],
                vec![[face.index() as Real; 4]; mip_len].as_slice(),
            );
            assert_eq!(
                texels[super::super::source_cubemap_face_mip_offset(face_size, mip_count, face, 0,)],
                [-1.0; 4],
            );
        }
    }

    #[test]
    fn pmrem_face_mip_outputs_clamp_mip_before_borrowing_final_storage() {
        let face_size = 4;
        let mip_count = super::super::source_cubemap_mip_count(face_size);
        let last_mip = mip_count - 1;
        let mut texels = vec![
            [-1.0 as Real; 4];
            super::super::source_cubemap_sample_count(face_size, mip_count)
        ];
        let mut outputs = super::super::source_cubemap_face_mip_outputs(
            &mut texels,
            face_size,
            mip_count,
            u32::MAX,
        );

        for output in &mut outputs {
            output.texels.fill([output.face.index() as Real; 4]);
        }
        drop(outputs);

        for face in super::CubemapFace::ALL {
            let last_mip_offset =
                super::super::source_cubemap_face_mip_offset(face_size, mip_count, face, last_mip);
            assert_eq!(texels[last_mip_offset], [face.index() as Real; 4]);
            assert_eq!(
                texels[super::super::source_cubemap_face_mip_offset(face_size, mip_count, face, 0,)],
                [-1.0; 4],
            );
        }
    }

    #[test]
    fn parallel_pmrem_prefilter_dispatches_each_mip_and_matches_serial_output() {
        let face_size = 8;
        let mip_count = super::super::source_cubemap_mip_count(face_size);
        let source_texels = vec![
            [0.25 as Real, 0.5 as Real, 0.75 as Real, 1.0 as Real];
            super::super::source_cubemap_sample_count(face_size, mip_count)
        ];
        let mut serial =
            vec![[0.0; 4]; super::super::source_cubemap_sample_count(face_size, mip_count)];
        let mut parallel = serial.clone();
        prefilter_pmrem_mips_from_source(
            &mut serial,
            face_size,
            mip_count,
            &source_texels,
            face_size,
            mip_count,
            SourceCubemapPrefilterQuality::Fast,
        );

        let executor = CountingParallelSliceExecutor::default();
        prefilter_pmrem_mips_from_source_with_parallel_executor(
            &mut parallel,
            face_size,
            mip_count,
            &source_texels,
            face_size,
            mip_count,
            SourceCubemapPrefilterQuality::Fast,
            &executor,
        );

        assert_eq!(parallel, serial);
        assert_eq!(
            executor.0.load(std::sync::atomic::Ordering::Relaxed),
            mip_count as usize,
            "each PMREM mip must dispatch its independent faces through the supplied executor"
        );
    }
}
