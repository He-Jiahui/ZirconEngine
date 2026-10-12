use super::*;

#[test]
fn resident_probe_word_layout_matches_the_compute_storage_contract() {
    assert_eq!(
        std::mem::size_of::<GpuResidentProbeInput>(),
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize * std::mem::size_of::<u32>()
    );
    assert_eq!(GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET, 8);
}
