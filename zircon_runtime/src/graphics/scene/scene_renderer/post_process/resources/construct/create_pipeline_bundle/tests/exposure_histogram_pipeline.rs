const EXPOSURE_HISTOGRAM_SHADER: &str = include_str!("../../../../shaders/exposure_histogram.wgsl");

#[test]
fn exposure_histogram_shader_declares_compute_entry_and_atomic_bins() {
    assert!(EXPOSURE_HISTOGRAM_SHADER.contains("@compute @workgroup_size(16, 16, 1)"));
    assert!(EXPOSURE_HISTOGRAM_SHADER.contains("fn cs_main"));
    assert!(EXPOSURE_HISTOGRAM_SHADER.contains("array<atomic<u32>, 64>"));
    assert!(EXPOSURE_HISTOGRAM_SHADER.contains("atomicAdd(&exposure_histogram"));
}
