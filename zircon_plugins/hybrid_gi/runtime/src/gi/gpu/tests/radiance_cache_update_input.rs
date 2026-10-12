use super::*;

#[test]
fn gpu_radiance_cache_update_input_preserves_generation_and_sample_bits() {
    let input = GpuRadianceCacheUpdateInput::from(&HybridGiPrepareRadianceCacheUpdate {
        slot: 7,
        generation: 0x0123_4567_89ab_cdef,
        radiance_rgb: [10, 20, 30],
        confidence_q8: 40,
        reuse_committed_radiance: true,
    });

    assert_eq!(input.slot, 7);
    assert_eq!(input.generation_low, 0x89ab_cdef);
    assert_eq!(input.generation_high, 0x0123_4567);
    assert_eq!(input.radiance_confidence.to_le_bytes(), [10, 20, 30, 40]);
    assert_eq!(input.reuse_committed_radiance, 1);
}
