use super::*;

#[derive(Default)]
struct CountingParallelSliceExecutor(std::sync::atomic::AtomicUsize);

impl CountingParallelSliceExecutor {
    fn call_count(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl crate::core::framework::tasks::ParallelSliceExecutor for CountingParallelSliceExecutor {
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
fn source_cubemap_face_size_clamps_equirect_height_to_power_of_two() {
    assert_eq!(source_cubemap_face_size_from_equirect_height(512), 256);
    assert_eq!(source_cubemap_face_size_from_equirect_height(32), 64);
    assert_eq!(source_cubemap_face_size_from_equirect_height(4096), 1024);
}

#[test]
fn source_and_pmrem_mip_layouts_are_independent() {
    let source_face_size = 512;
    let cubemap =
        build_source_cubemap_from_equirect(source_face_size, |_, _| [0.25, 0.5, 0.75, 1.0]);

    assert_eq!(cubemap.source_face_size(), source_face_size);
    assert_eq!(
        cubemap.source_mip_count(),
        source_cubemap_mip_count(source_face_size)
    );
    assert_eq!(cubemap.pmrem_face_size(), SOURCE_CUBEMAP_PMREM_FACE_SIZE);
    assert_eq!(cubemap.pmrem_mip_count(), SOURCE_CUBEMAP_PMREM_MIP_COUNT);
    assert_eq!(
        cubemap.source_texels().len(),
        source_cubemap_sample_count(cubemap.source_face_size(), cubemap.source_mip_count())
    );
    assert_eq!(
        cubemap.pmrem_texels().len(),
        source_cubemap_sample_count(cubemap.pmrem_face_size(), cubemap.pmrem_mip_count())
    );
}

#[test]
fn source_cubemap_constant_equirect_preserves_all_mips() {
    let cubemap = build_source_cubemap_from_equirect(4, |_, _| [0.25, 0.5, 0.75, 1.0]);

    assert_eq!(cubemap.source_face_size(), 4);
    assert_eq!(cubemap.source_mip_count(), 3);
    assert_eq!(cubemap.pmrem_face_size(), SOURCE_CUBEMAP_PMREM_FACE_SIZE);
    assert_eq!(cubemap.pmrem_mip_count(), SOURCE_CUBEMAP_PMREM_MIP_COUNT);
    for texel in cubemap.source_texels() {
        assert_vec4_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
    for texel in cubemap.pmrem_texels() {
        assert_vec4_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
}

#[test]
fn source_cubemap_parallel_executor_matches_serial_chain() {
    let serial = SourceCubemapMipChain::from_equirect_with_pmrem_layout(
        64,
        64,
        source_cubemap_mip_count(64),
        SourceCubemapPrefilterQuality::Fast,
        |u, v| [u, v, u * v, 1.0],
    );
    let executor = CountingParallelSliceExecutor::default();
    let parallel = SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_parallel_executor(
        64,
        64,
        source_cubemap_mip_count(64),
        SourceCubemapPrefilterQuality::Fast,
        &executor,
        |u, v| [u, v, u * v, 1.0],
    );

    assert_eq!(parallel, serial);
    assert!(
        executor.call_count() >= source_cubemap_mip_count(64) as usize,
        "each PMREM mip must dispatch its independent cube-face work through the caller executor"
    );
}

#[test]
fn source_cubemap_samples_equirect_uv_from_cube_face_direction() {
    let cubemap = build_source_cubemap_from_equirect(3, |u, v| [u, v, 0.0, 1.0]);

    assert_vec4_close(
        source_texel_at(&cubemap, CubemapFace::PositiveZ, 0, 1, 1),
        [0.5, 0.5, 0.0, 1.0],
    );
    assert_vec4_close(
        source_texel_at(&cubemap, CubemapFace::PositiveX, 0, 1, 1),
        [0.75, 0.5, 0.0, 1.0],
    );
}

fn source_texel_at(
    cubemap: &SourceCubemapMipChain,
    face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [Real; 4] {
    let mip_level = mip_level.min(cubemap.source_mip_count().saturating_sub(1));
    let mip_size = source_cubemap_mip_size(cubemap.source_face_size(), mip_level);
    assert!(x < mip_size && y < mip_size, "test texel must be in bounds");
    let offset = source_cubemap_face_mip_offset(
        cubemap.source_face_size(),
        cubemap.source_mip_count(),
        face,
        mip_level,
    );
    cubemap.source_texels()[offset + y as usize * mip_size as usize + x as usize]
}
