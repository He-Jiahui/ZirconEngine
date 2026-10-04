use super::half_resolution_transparency_supported;

#[test]
fn half_resolution_transparency_requires_a_single_sample_graph() {
    assert!(half_resolution_transparency_supported(1));
    assert!(!half_resolution_transparency_supported(2));
    assert!(!half_resolution_transparency_supported(4));
}
