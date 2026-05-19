use crate::core::framework::render::{
    RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderCapabilitySummary, RenderFramework,
    RenderFrameworkError, RenderQualityProfile, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, SolariRuntimeStatus,
};
use crate::core::math::UVec2;
use crate::scene::world::World;

use super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework, pluginized_wgpu_render_framework_with_advanced_providers,
    pluginized_wgpu_render_framework_with_solari_provider,
};

#[test]
fn render_product_solari_is_not_requested_by_default_or_advanced_profiles() {
    let server = pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Ready);
    server.override_capabilities_for_tests(solari_capabilities());
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, default_extract())
        .unwrap();
    let default_stats = server.query_stats().unwrap();
    assert_eq!(
        default_stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::NotRequested
    );

    let advanced = pluginized_wgpu_render_framework_with_advanced_providers();
    advanced.override_capabilities_for_tests(solari_capabilities());
    let viewport = advanced
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    advanced
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("advanced-no-solari")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true),
        )
        .unwrap();
    advanced
        .submit_frame_extract(viewport, default_extract())
        .unwrap();
    let advanced_stats = advanced.query_stats().unwrap();
    assert_eq!(
        advanced_stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::NotRequested
    );
}

#[test]
fn render_product_solari_reports_provider_missing_with_full_caps() {
    let server = pluginized_wgpu_render_framework();
    server.override_capabilities_for_tests(solari_capabilities());
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            solari_quality_profile("solari-provider-missing", true),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, default_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(stats.last_solari_runtime_report.requested);
    assert_eq!(
        stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::ProviderMissing
    );
    assert_eq!(
        stats.last_solari_runtime_report.degradation_reason_labels(),
        vec!["provider-missing"]
    );
}

#[test]
fn render_product_solari_reports_experimental_gate_before_enabling() {
    let server = pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Ready);
    server.override_capabilities_for_tests(solari_capabilities());
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, solari_quality_profile("solari-disabled", false))
        .unwrap();

    server
        .submit_frame_extract(viewport, default_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_solari_runtime_report.provider_id.as_deref(),
        Some("test.solari")
    );
    assert_eq!(
        stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::ExperimentalDisabled
    );
}

#[test]
fn render_product_solari_reports_unavailable_provider_with_full_caps_and_gate() {
    let server =
        pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Unavailable);
    server.override_capabilities_for_tests(solari_capabilities());
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, solari_quality_profile("solari-unavailable", true))
        .unwrap();

    server
        .submit_frame_extract(viewport, default_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_solari_runtime_report.status,
        SolariRuntimeStatus::Unavailable
    );
    assert_eq!(
        stats.last_solari_runtime_report.degradation_reason_labels(),
        vec!["provider-unavailable"]
    );
}

#[test]
fn render_product_solari_quality_profile_requires_bevy_solari_caps() {
    let server = pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Ready);
    server.override_capabilities_for_tests(RenderCapabilitySummary {
        backend_name: "solari-missing-caps".to_string(),
        supports_fxaa: true,
        ..RenderCapabilitySummary::default()
    });
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    let error = server
        .set_quality_profile(
            viewport,
            solari_quality_profile("solari-missing-caps", true),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        RenderFrameworkError::CapabilityMismatch { .. }
    ));
    if let RenderFrameworkError::CapabilityMismatch { missing, .. } = error {
        assert!(missing.contains(&RenderCapabilityMismatchDetail::new(
            RenderCapabilityKind::InlineRayQuery
        )));
        assert!(missing.contains(&RenderCapabilityMismatchDetail::new(
            RenderCapabilityKind::BufferBindingArray
        )));
        assert!(missing.contains(&RenderCapabilityMismatchDetail::new(
            RenderCapabilityKind::PartiallyBoundBindingArray
        )));
    }
}

pub(super) fn solari_quality_profile(
    name: &str,
    experimental_enabled: bool,
) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_solari(true)
        .with_solari_experimental_enabled(experimental_enabled)
}

pub(super) fn solari_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "solari-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        acceleration_structures_supported: true,
        inline_ray_query: true,
        supports_buffer_binding_array: true,
        supports_texture_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        supports_partially_bound_binding_array: true,
        virtual_geometry_supported: true,
        hybrid_global_illumination_supported: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        ..RenderCapabilitySummary::default()
    }
}

fn default_extract() -> crate::core::framework::render::RenderFrameExtract {
    let mut extract = crate::core::framework::render::RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(900),
        World::new().to_render_snapshot(),
    );
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract
}
