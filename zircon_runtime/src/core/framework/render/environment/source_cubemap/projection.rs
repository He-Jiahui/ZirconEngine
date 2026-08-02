use super::{
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE, SourceCubemapMipChain,
    SourceCubemapPmremLayout, SourceCubemapPrefilterQuality,
    build_source_cubemap_from_source_mips_with_pmrem_layout,
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor, mipmap,
    source_cubemap_face_mip_offset, source_cubemap_mip_count, source_cubemap_sample_count,
};
use crate::core::framework::render::environment::{
    CubemapFace, cubemap_face_size_from_equirect_height, cubemap_texel_direction,
    equirect_uv_from_direction,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;

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
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_equirect(face_size, sample_equirect);
    let source_mips = mipmap::source_cubemap_mips_from_base(&source_storage, face_size, mip_count);
    build_source_cubemap_from_source_mips_with_pmrem_layout(
        face_size,
        mip_count,
        source_mips,
        pmrem_layout,
        quality,
    )
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
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_equirect_with_parallel_executor(
            face_size,
            parallel_executor,
            sample_equirect,
        );
    let source_mips = mipmap::source_cubemap_mips_from_base_with_parallel_executor(
        &source_storage,
        face_size,
        mip_count,
        parallel_executor,
    );
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor(
        face_size,
        mip_count,
        source_mips,
        pmrem_layout,
        quality,
        parallel_executor,
    )
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
    struct FilteredFace {
        face: CubemapFace,
        texels: Vec<[Real; 4]>,
    }

    let face_size = face_size.max(1);
    let mip_count = source_cubemap_mip_count(face_size);
    let mut filtered_faces = CubemapFace::ALL
        .into_iter()
        .map(|face| FilteredFace {
            face,
            texels: Vec::new(),
        })
        .collect::<Vec<_>>();
    parallel_executor.parallel_for(&mut filtered_faces, 1, |faces| {
        for filtered_face in faces {
            let mut texels = vec![[0.0; 4]; face_size as usize * face_size as usize];
            for y in 0..face_size {
                for x in 0..face_size {
                    let direction = cubemap_texel_direction(filtered_face.face, x, y, face_size);
                    let uv = equirect_uv_from_direction(direction);
                    texels[y as usize * face_size as usize + x as usize] =
                        sample_equirect(uv[0], uv[1]);
                }
            }
            filtered_face.texels = texels;
        }
    });

    let mut source_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    for filtered_face in filtered_faces {
        let base_offset =
            source_cubemap_face_mip_offset(face_size, mip_count, filtered_face.face, 0);
        source_storage[base_offset..base_offset + filtered_face.texels.len()]
            .copy_from_slice(&filtered_face.texels);
    }
    (face_size, mip_count, source_storage)
}
