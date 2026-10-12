use super::*;

#[test]
fn gpu_radiance_cache_consume_input_widens_all_eight_corner_weights() {
    let input = GpuRadianceCacheConsumeInput::new(
        &HybridGiPrepareRadianceCacheConsume {
            probe_id: 11,
            generation: 0xfedc_ba98_7654_3210,
            slots: [1, 2, 3, 4, 5, 6, 7, 8],
            weights_q16: [0, 1, 2, 3, 4, 5, 6, u16::MAX],
        },
        13,
    );

    assert_eq!(input.probe_id, 11);
    assert_eq!(input.generation_low, 0x7654_3210);
    assert_eq!(input.generation_high, 0xfedc_ba98);
    assert_eq!(input.resident_probe_index, 13);
    assert_eq!(input.slots, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(
        input.weights_q16,
        [0, 1, 2, 3, 4, 5, 6, u32::from(u16::MAX)]
    );
}
