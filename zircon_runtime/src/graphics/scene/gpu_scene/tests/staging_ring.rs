use super::{
    GpuSceneStagingDestination, GpuSceneStagingRing, GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES,
};

#[test]
fn render_gpu_scene_staging_ring_selects_only_large_uploads() {
    assert!(!GpuSceneStagingRing::should_stage(
        GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES - 1
    ));
    assert!(GpuSceneStagingRing::should_stage(
        GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES
    ));
}

#[test]
fn render_gpu_scene_staging_ring_keeps_copy_offsets_in_one_frame_blob() {
    let mut ring = GpuSceneStagingRing::default();
    ring.begin_frame();
    ring.stage_pod_slice(GpuSceneStagingDestination::Primitive, 32, &[1u32; 4]);
    ring.stage_pod_slice(GpuSceneStagingDestination::Instance, 64, &[2u32; 4]);

    assert_eq!(ring.copies.len(), 2);
    assert_eq!(ring.copies[0].source_offset, 0);
    assert_eq!(ring.copies[1].source_offset, 16);
    assert_eq!(ring.bytes.len(), 32);
}
