use super::*;

#[derive(Default)]
struct CountingParallelSliceExecutor {
    call_count: std::sync::atomic::AtomicUsize,
    submitted_work_items: std::sync::atomic::AtomicUsize,
}

impl CountingParallelSliceExecutor {
    fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn submitted_work_items(&self) -> u64 {
        self.submitted_work_items
            .load(std::sync::atomic::Ordering::Relaxed) as u64
    }
}

impl crate::core::framework::tasks::ParallelSliceExecutor for CountingParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        let chunk_size = chunk_size.max(1);
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.submitted_work_items.fetch_add(
            items.len().div_ceil(chunk_size),
            std::sync::atomic::Ordering::Relaxed,
        );
        for chunk in items.chunks_mut(chunk_size) {
            task(chunk);
        }
    }
}

#[test]
fn timed_equirect_builds_preserve_output_and_phase_accounting_for_both_paths() {
    let (serial, serial_timing) = SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_timing(
        8,
        2,
        2,
        SourceCubemapPrefilterQuality::Fast,
        |u, v| [u, v, u * v, 1.0],
    );
    let executor = CountingParallelSliceExecutor::default();
    let (parallel, parallel_timing) =
        SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_parallel_executor_and_timing(
            8,
            2,
            2,
            SourceCubemapPrefilterQuality::Fast,
            &executor,
            |u, v| [u, v, u * v, 1.0],
        );

    assert_eq!(parallel, serial);
    assert_eq!(serial.source_face_size(), 8);
    assert_eq!(serial.source_mip_count(), source_cubemap_mip_count(8));
    assert_eq!(serial.pmrem_face_size(), 2);
    assert_eq!(serial.pmrem_mip_count(), 2);
    assert!(
        executor.call_count() >= 3,
        "timed parallel construction must use the supplied executor for projection and PMREM"
    );
    assert_eq!(
        serial_timing.equirect_projection_parallel_work_items(),
        0,
        "serial construction must not report caller-executor work"
    );
    assert_eq!(serial_timing.source_mip_build_parallel_work_items(), 0);
    assert_eq!(serial_timing.pmrem_build_parallel_work_items(), 0);
    assert_eq!(
        parallel_timing
            .equirect_projection_parallel_work_items()
            .saturating_add(parallel_timing.source_mip_build_parallel_work_items())
            .saturating_add(parallel_timing.pmrem_build_parallel_work_items()),
        executor.submitted_work_items(),
        "each caller-executor submission must have exactly one build-phase owner"
    );
    for timing in [serial_timing, parallel_timing] {
        assert_eq!(
            timing.total(),
            timing
                .equirect_projection()
                .saturating_add(timing.source_mip_build())
                .saturating_add(timing.pmrem_build())
                .saturating_add(timing.sh9_build()),
            "phase timing must retain a non-overlapping accounting contract"
        );
    }
}

#[test]
fn timed_parallel_pmrem_rebuild_attributes_only_pmrem_submissions() {
    let source_face_size = 8;
    let source_mip_count = source_cubemap_mip_count(source_face_size);
    let source_texels = vec![
        [0.25 as Real, 0.5 as Real, 0.75 as Real, 1.0 as Real];
        source_cubemap_sample_count(source_face_size, source_mip_count)
    ];
    let executor = CountingParallelSliceExecutor::default();
    let (_cubemap, timing) =
        rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing(
            source_face_size,
            source_mip_count,
            source_texels,
            4,
            source_cubemap_mip_count(4),
            SourceCubemapPrefilterQuality::Fast,
            &executor,
        );

    assert_eq!(timing.equirect_projection_parallel_work_items(), 0);
    assert_eq!(timing.source_mip_build_parallel_work_items(), 0);
    assert!(timing.pmrem_build_parallel_work_items() > 0);
    assert_eq!(
        timing.pmrem_build_parallel_work_items(),
        executor.submitted_work_items(),
        "derived-only rebuilds must assign every submitted chunk to PMREM"
    );
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
    assert_eq!(
        executor.call_count(),
        source_cubemap_mip_count(64) as usize * 2,
        "the caller executor must receive one equirect projection, each source mip after mip zero, and each PMREM mip"
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
