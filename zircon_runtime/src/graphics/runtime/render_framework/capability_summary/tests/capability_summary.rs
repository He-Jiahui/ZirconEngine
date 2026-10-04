use crate::rhi::RenderBackendCaps;

use super::capability_summary;

#[test]
fn capability_summary_reports_taa_when_offscreen_postprocess_is_available() {
    let with_offscreen =
        capability_summary(&RenderBackendCaps::new("taa-capable").with_offscreen_support(true));
    let without_offscreen =
        capability_summary(&RenderBackendCaps::new("taa-disabled").with_offscreen_support(false));

    assert!(with_offscreen.supports_taa);
    assert!(!without_offscreen.supports_taa);
}

#[test]
fn capability_summary_preserves_the_gpu_timestamp_gate() {
    let caps = RenderBackendCaps::new("timestamps").with_gpu_timestamp(true);

    assert!(capability_summary(&caps).supports_gpu_timestamp);
}

#[test]
fn capability_summary_preserves_optional_compute_and_observation_gates() {
    let caps = RenderBackendCaps::new("optional-gates")
        .with_subgroup(true)
        .with_pipeline_statistics_query(true);
    let summary = capability_summary(&caps);

    assert!(summary.supports_subgroup);
    assert!(summary.supports_pipeline_statistics_query);
}
