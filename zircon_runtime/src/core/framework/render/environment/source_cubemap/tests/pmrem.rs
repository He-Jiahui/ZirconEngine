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
fn ggx_distribution_preserves_the_valid_canonical_low_roughness_peak() {
    let roughness = super::super::source_cubemap_roughness_from_pmrem_mip(1, 8);
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let expected = 1.0 / (std::f32::consts::PI * alpha_squared);
    let actual = distribution_ggx(1.0, roughness);

    assert!(
        (actual - expected).abs() <= expected * 0.00001,
        "PMREM D_GGX must retain the canonical mip1 peak: actual={actual} expected={expected}"
    );
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
    let mut texels =
        vec![[-1.0 as Real; 4]; super::super::source_cubemap_sample_count(face_size, mip_count)];
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
    let mut texels =
        vec![[-1.0 as Real; 4]; super::super::source_cubemap_sample_count(face_size, mip_count)];
    let mut outputs =
        super::super::source_cubemap_face_mip_outputs(&mut texels, face_size, mip_count, u32::MAX);

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
