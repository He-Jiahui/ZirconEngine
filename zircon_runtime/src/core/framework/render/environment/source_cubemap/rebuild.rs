use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;

use super::{
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter, pmrem,
    SourceCubemapBuildTiming, SourceCubemapMipChain, SourceCubemapPmremLayout,
    SourceCubemapPrefilterQuality, WorkItemCountingParallelSliceExecutor,
};

/// Rebuilds derived PMREM/SH9 from an immutable staged source pyramid.
///
/// This is the derived-only cache-miss path: callers retain the source layout
/// while selecting a new PMREM layout without repeating HDR projection.
pub fn rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
) -> (SourceCubemapMipChain, SourceCubemapBuildTiming) {
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
        face_size,
        mip_count,
        source_texels,
        SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
        |pmrem_texels,
         pmrem_face_size,
         pmrem_mip_count,
         source_texels,
         source_face_size,
         source_mip_count| {
            pmrem::prefilter_pmrem_mips_from_source(
                pmrem_texels,
                pmrem_face_size,
                pmrem_mip_count,
                source_texels,
                source_face_size,
                source_mip_count,
                quality,
            );
        },
    )
}

pub fn rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing<
    E,
>(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
) -> (SourceCubemapMipChain, SourceCubemapBuildTiming)
where
    E: ParallelSliceExecutor,
{
    let pmrem_executor = WorkItemCountingParallelSliceExecutor::new(parallel_executor);
    let (cubemap, mut timing) =
        build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
            face_size,
            mip_count,
            source_texels,
            SourceCubemapPmremLayout::new(pmrem_face_size, pmrem_mip_count),
            |pmrem_texels,
             pmrem_face_size,
             pmrem_mip_count,
             source_texels,
             source_face_size,
             source_mip_count| {
                pmrem::prefilter_pmrem_mips_from_source_with_parallel_executor(
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
    timing.pmrem_build_parallel_work_items = pmrem_executor.submitted_work_items();
    (cubemap, timing)
}
