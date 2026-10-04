const EXPOSURE_RESOLVE_SHADER: &str = include_str!("../../../../shaders/exposure_resolve.wgsl");

#[test]
fn exposure_resolve_shader_declares_manual_and_histogram_output() {
    assert!(EXPOSURE_RESOLVE_SHADER.contains("@compute @workgroup_size(1, 1, 1)"));
    assert!(EXPOSURE_RESOLVE_SHADER.contains("fn histogram_average_ev100"));
    assert!(EXPOSURE_RESOLVE_SHADER.contains("fn adapt_ev100"));
    assert!(EXPOSURE_RESOLVE_SHADER.contains("current_exposure[0]"));
}
