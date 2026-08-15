use super::{
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter, mipmap,
    source_cubemap_face_mip_offset, source_cubemap_face_mip_outputs, source_cubemap_mip_count,
    source_cubemap_sample_count, SourceCubemapBuildTiming, SourceCubemapMipChain,
    SourceCubemapPmremLayout, SourceCubemapPrefilterQuality, WorkItemCountingParallelSliceExecutor,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE,
};
use crate::core::framework::render::environment::{
    cubemap_face_size_from_equirect_height, cubemap_texel_direction, equirect_uv_from_direction,
    CubemapFace,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;
use std::time::Instant;

impl SourceCubemapMipChain {
    /// Projects an equirectangular source and bakes PMREM directly at an
    /// independent destination layout without first producing the default PMREM.
    pub fn from_equirect_with_pmrem_layout<F>(
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        quality: SourceCubemapPrefilterQuality,
        sample_equirect: F,
    ) -> Self
    where
        F: FnMut(Real, Real) -> [Real; 4],
    {
        build_source_cubemap_from_equirect_with_pmrem_layout(
            source_face_size,
            SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
            quality,
            sample_equirect,
        )
    }

    /// Builds a source cubemap and attributes its CPU bake phases separately.
    pub fn from_equirect_with_pmrem_layout_and_timing<F>(
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        quality: SourceCubemapPrefilterQuality,
        sample_equirect: F,
    ) -> (Self, SourceCubemapBuildTiming)
    where
        F: FnMut(Real, Real) -> [Real; 4],
    {
        build_source_cubemap_from_equirect_with_pmrem_layout_and_timing(
            source_face_size,
            SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
            quality,
            sample_equirect,
        )
    }

    /// Parallel equirectangular projection and PMREM construction.
    ///
    /// `sample_equirect` can be invoked concurrently for independent cube faces;
    /// callers must provide a thread-safe, side-effect-free sampler.
    pub fn from_equirect_with_parallel_executor<E, F>(
        source_face_size: u32,
        parallel_executor: &E,
        sample_equirect: F,
    ) -> Self
    where
        E: ParallelSliceExecutor,
        F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
    {
        build_source_cubemap_from_equirect_with_parallel_executor(
            source_face_size,
            parallel_executor,
            sample_equirect,
        )
    }

    /// Parallel equirectangular projection and PMREM construction with an
    /// explicit reflection result layout.
    ///
    /// `sample_equirect` can be invoked concurrently for independent cube faces;
    /// callers must provide a thread-safe, side-effect-free sampler.
    pub fn from_equirect_with_pmrem_layout_and_parallel_executor<E, F>(
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        quality: SourceCubemapPrefilterQuality,
        parallel_executor: &E,
        sample_equirect: F,
    ) -> Self
    where
        E: ParallelSliceExecutor,
        F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
    {
        build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor(
            source_face_size,
            SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
            quality,
            parallel_executor,
            sample_equirect,
        )
    }

    /// Builds a source cubemap through the caller executor and attributes the
    /// projection, mip, PMREM, and SH9 CPU phases separately.
    pub fn from_equirect_with_pmrem_layout_and_parallel_executor_and_timing<E, F>(
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        quality: SourceCubemapPrefilterQuality,
        parallel_executor: &E,
        sample_equirect: F,
    ) -> (Self, SourceCubemapBuildTiming)
    where
        E: ParallelSliceExecutor,
        F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
    {
        build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor_and_timing(
            source_face_size,
            SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
            quality,
            parallel_executor,
            sample_equirect,
        )
    }
}

pub fn source_cubemap_face_size_from_equirect_height(equirect_height: u32) -> u32 {
    cubemap_face_size_from_equirect_height(equirect_height)
        .next_power_of_two()
        .clamp(SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_MAX_FACE_SIZE)
}

pub fn build_source_cubemap_from_equirect<F>(
    face_size: u32,
    sample_equirect: F,
) -> SourceCubemapMipChain
where
    F: FnMut(Real, Real) -> [Real; 4],
{
    build_source_cubemap_from_equirect_with_pmrem_layout(
        face_size,
        SourceCubemapPmremLayout::default(),
        SourceCubemapPrefilterQuality::Normal,
        sample_equirect,
    )
}

fn build_source_cubemap_from_equirect_with_parallel_executor<E, F>(
    face_size: u32,
    parallel_executor: &E,
    sample_equirect: F,
) -> SourceCubemapMipChain
where
    E: ParallelSliceExecutor,
    F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
{
    build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor(
        face_size,
        SourceCubemapPmremLayout::default(),
        SourceCubemapPrefilterQuality::Normal,
        parallel_executor,
        sample_equirect,
    )
}

fn build_source_cubemap_from_equirect_with_pmrem_layout<F>(
    face_size: u32,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
    sample_equirect: F,
) -> SourceCubemapMipChain
where
    F: FnMut(Real, Real) -> [Real; 4],
{
    build_source_cubemap_from_equirect_with_pmrem_layout_and_timing(
        face_size,
        pmrem_layout,
        quality,
        sample_equirect,
    )
    .0
}

fn build_source_cubemap_from_equirect_with_pmrem_layout_and_timing<F>(
    face_size: u32,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
    sample_equirect: F,
) -> (SourceCubemapMipChain, SourceCubemapBuildTiming)
where
    F: FnMut(Real, Real) -> [Real; 4],
{
    let projection_started = Instant::now();
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_equirect(face_size, sample_equirect);
    let equirect_projection = projection_started.elapsed();
    let mips_started = Instant::now();
    let source_mips = mipmap::source_cubemap_mips_from_base(&source_storage, face_size, mip_count);
    let source_mip_build = mips_started.elapsed();
    let (cubemap, mut timing) =
        build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
            face_size,
            mip_count,
            source_mips,
            pmrem_layout,
            |pmrem_texels,
             pmrem_face_size,
             pmrem_mip_count,
             source_texels,
             source_face_size,
             source_mip_count| {
                super::pmrem::prefilter_pmrem_mips_from_source(
                    pmrem_texels,
                    pmrem_face_size,
                    pmrem_mip_count,
                    source_texels,
                    source_face_size,
                    source_mip_count,
                    quality,
                );
            },
        );
    timing.equirect_projection = equirect_projection;
    timing.source_mip_build = source_mip_build;
    (cubemap, timing)
}

fn build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor<E, F>(
    face_size: u32,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
    sample_equirect: F,
) -> SourceCubemapMipChain
where
    E: ParallelSliceExecutor,
    F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
{
    build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor_and_timing(
        face_size,
        pmrem_layout,
        quality,
        parallel_executor,
        sample_equirect,
    )
    .0
}

fn build_source_cubemap_from_equirect_with_pmrem_layout_and_parallel_executor_and_timing<E, F>(
    face_size: u32,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
    sample_equirect: F,
) -> (SourceCubemapMipChain, SourceCubemapBuildTiming)
where
    E: ParallelSliceExecutor,
    F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
{
    let projection_executor = WorkItemCountingParallelSliceExecutor::new(parallel_executor);
    let projection_started = Instant::now();
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_equirect_with_parallel_executor(
            face_size,
            &projection_executor,
            sample_equirect,
        );
    let equirect_projection = projection_started.elapsed();
    let equirect_projection_parallel_work_items = projection_executor.submitted_work_items();

    let source_mip_executor = WorkItemCountingParallelSliceExecutor::new(parallel_executor);
    let mips_started = Instant::now();
    let source_mips = mipmap::source_cubemap_mips_from_base_with_parallel_executor(
        &source_storage,
        face_size,
        mip_count,
        &source_mip_executor,
    );
    let source_mip_build = mips_started.elapsed();
    let source_mip_build_parallel_work_items = source_mip_executor.submitted_work_items();

    let pmrem_executor = WorkItemCountingParallelSliceExecutor::new(parallel_executor);
    let (cubemap, mut timing) =
        build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
            face_size,
            mip_count,
            source_mips,
            pmrem_layout,
            |pmrem_texels,
             pmrem_face_size,
             pmrem_mip_count,
             source_texels,
             source_face_size,
             source_mip_count| {
                super::pmrem::prefilter_pmrem_mips_from_source_with_parallel_executor(
                    pmrem_texels,
                    pmrem_face_size,
                    pmrem_mip_count,
                    source_texels,
                    source_face_size,
                    source_mip_count,
                    quality,
                    &pmrem_executor,
                );
            },
        );
    timing.equirect_projection = equirect_projection;
    timing.source_mip_build = source_mip_build;
    timing.equirect_projection_parallel_work_items = equirect_projection_parallel_work_items;
    timing.source_mip_build_parallel_work_items = source_mip_build_parallel_work_items;
    timing.pmrem_build_parallel_work_items = pmrem_executor.submitted_work_items();
    (cubemap, timing)
}

fn source_cubemap_base_from_equirect<F>(
    face_size: u32,
    mut sample_equirect: F,
) -> (u32, u32, Vec<[Real; 4]>)
where
    F: FnMut(Real, Real) -> [Real; 4],
{
    let face_size = face_size.max(1);
    let mip_count = source_cubemap_mip_count(face_size);
    let mut source_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];

    for face in CubemapFace::ALL {
        let base_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        for y in 0..face_size {
            for x in 0..face_size {
                let direction = cubemap_texel_direction(face, x, y, face_size);
                let uv = equirect_uv_from_direction(direction);
                source_storage[base_offset + y as usize * face_size as usize + x as usize] =
                    sample_equirect(uv[0], uv[1]);
            }
        }
    }

    (face_size, mip_count, source_storage)
}

fn source_cubemap_base_from_equirect_with_parallel_executor<E, F>(
    face_size: u32,
    parallel_executor: &E,
    sample_equirect: F,
) -> (u32, u32, Vec<[Real; 4]>)
where
    E: ParallelSliceExecutor,
    F: Fn(Real, Real) -> [Real; 4] + Send + Sync,
{
    let face_size = face_size.max(1);
    let mip_count = source_cubemap_mip_count(face_size);
    let mut source_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    let mut outputs = source_cubemap_face_mip_outputs(&mut source_storage, face_size, mip_count, 0);
    parallel_executor.parallel_for(&mut outputs, 1, |outputs| {
        for output in outputs {
            for y in 0..face_size {
                for x in 0..face_size {
                    let direction = cubemap_texel_direction(output.face, x, y, face_size);
                    let uv = equirect_uv_from_direction(direction);
                    output.texels[y as usize * face_size as usize + x as usize] =
                        sample_equirect(uv[0], uv[1]);
                }
            }
        }
    });
    (face_size, mip_count, source_storage)
}
