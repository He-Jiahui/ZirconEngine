use super::*;

#[test]
fn policy_report_lists_missing_gates() {
    let report = RayTracingPolicyReport::from_backend(
        RayTracingPath::Pipeline,
        RayTracingBackendCapabilities {
            acceleration_structures: true,
            inline_ray_query: false,
            ray_tracing_pipeline: false,
        },
    );

    assert!(!report.supported);
    assert_eq!(
        report.missing_gates,
        vec![RenderFeatureCapabilityRequirement::RayTracingPipeline]
    );
}

#[test]
fn policy_feature_is_opt_in_and_capability_gated() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.extensions.render_features()[0].capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::AccelerationStructures,
            RenderFeatureCapabilityRequirement::InlineRayQuery,
            RenderFeatureCapabilityRequirement::RayTracingPipeline,
        ]
    );
}
