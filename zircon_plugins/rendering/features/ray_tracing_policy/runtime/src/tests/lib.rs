use super::*;
#[path = "../../../../test_support/frame_smoke.rs"]
mod frame_smoke;

#[test]
fn ray_tracing_policy_reports_backend_gates_and_keeps_a_real_product_frame() {
    let policy = RayTracingPolicyReport::from_backend(
        RayTracingPath::InlineQuery,
        RayTracingBackendCapabilities {
            acceleration_structures: true,
            inline_ray_query: false,
            ray_tracing_pipeline: false,
        },
    );
    assert!(!policy.supported);
    assert_eq!(
        policy.missing_gates,
        vec![RenderFeatureCapabilityRequirement::InlineRayQuery]
    );

    let (frame, stats) =
        frame_smoke::render_nonempty_product_frame([render_feature_descriptor()], []);
    assert_eq!((frame.width, frame.height), (96, 72));
    assert_eq!(frame.rgba.len(), 96 * 72 * 4);
    assert_eq!(
        RayTracingPolicyReport::from_backend(
            RayTracingPath::Pipeline,
            RayTracingBackendCapabilities {
                acceleration_structures: true,
                inline_ray_query: false,
                ray_tracing_pipeline: false,
            },
        )
        .missing_gates,
        vec![RenderFeatureCapabilityRequirement::RayTracingPipeline]
    );
    assert!(stats.last_mesh_draw_count > 0);
}

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
