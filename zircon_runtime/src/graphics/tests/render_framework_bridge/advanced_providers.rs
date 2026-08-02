use super::*;
use crate::core::framework::render::{
    RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
};

#[test]
fn headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities() {
    let server = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true),
        )
        .unwrap();
    let extract = flagship_extract();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert!(stats.capabilities.virtual_geometry_supported);
    assert!(stats.capabilities.hybrid_global_illumination_supported);
    assert!(!stats.capabilities.acceleration_structures_supported);
    assert!(!stats.capabilities.inline_ray_query);
    assert!(!stats.capabilities.ray_tracing_pipeline);
    assert!(stats
        .last_effective_features
        .contains(&"virtual_geometry".to_string()));
    assert!(stats
        .last_effective_features
        .contains(&"hybrid_gi".to_string()));
    assert_eq!(
        stats
            .advanced_provider_availability
            .virtual_geometry_provider_id
            .as_deref(),
        Some("test.virtual-geometry")
    );
    assert_eq!(
        stats
            .advanced_provider_availability
            .hybrid_gi_provider_id
            .as_deref(),
        Some("test.hybrid-gi")
    );
    assert_eq!(
        advanced_provider_report(&stats, AdvancedRenderFeature::VirtualGeometry).status,
        AdvancedProviderStatus::Ready
    );
    assert_eq!(
        advanced_provider_report(&stats, AdvancedRenderFeature::HybridGlobalIllumination).status,
        AdvancedProviderStatus::Ready
    );
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 5);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert_eq!(
        stats.last_hybrid_gi_payload_source,
        RenderHybridGiPayloadSource::SceneRepresentation
    );
    assert_eq!(stats.last_virtual_geometry_visible_cluster_count, 2);
    assert_eq!(stats.last_virtual_geometry_requested_page_count, 1);
    assert_eq!(stats.last_virtual_geometry_dirty_page_count, 1);
    assert_eq!(stats.last_virtual_geometry_page_table_entry_count, 0);
    assert_eq!(stats.last_virtual_geometry_resident_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_pending_request_count, 0);
    assert_eq!(stats.last_virtual_geometry_completed_page_count, 0);
    assert_eq!(
        stats.last_virtual_geometry_replaced_page_count, 0,
        "plugin-owned VG residency replacement pressure should not leak through runtime stats after the hard cutover"
    );
    assert_eq!(
        stats.last_virtual_geometry_indirect_draw_count, 2,
        "the no-RT flagship path should still report renderer-produced VG execution draws"
    );
    assert_eq!(stats.last_virtual_geometry_indirect_segment_count, 0);
    assert_eq!(stats.last_virtual_geometry_indirect_buffer_count, 0);
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_cache_entry_count, 0);
    assert_eq!(stats.last_hybrid_gi_resident_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_pending_update_count, 0);
    assert_eq!(stats.last_hybrid_gi_scheduled_trace_region_count, 0);
    assert_eq!(stats.last_hybrid_gi_scene_card_count, 0);
    let resolved = stats
        .last_hybrid_gi_resolved_settings
        .expect("enabled HybridGI provider must publish effective settings");
    assert_eq!(resolved.mode, RenderHybridGiMode::DynamicOnly);
    assert_eq!(resolved.profile, RenderHybridGiProfile::Custom);
    assert_eq!(resolved.quality, RenderHybridGiQuality::Medium);
    assert_eq!(
        (
            resolved.trace_budget,
            resolved.card_budget,
            resolved.voxel_budget
        ),
        (2, 1, 2)
    );
    assert_eq!(resolved.fallback_reason, None);
    assert_eq!(stats.last_hybrid_gi_surface_cache_resident_page_count, 0);
    assert_eq!(stats.last_hybrid_gi_surface_cache_dirty_page_count, 0);
    assert_eq!(stats.last_hybrid_gi_surface_cache_feedback_card_count, 0);
    assert_eq!(stats.last_hybrid_gi_surface_cache_capture_slot_count, 0);
    assert_eq!(stats.last_hybrid_gi_surface_cache_invalidated_page_count, 0);
    assert_eq!(stats.last_hybrid_gi_voxel_resident_clipmap_count, 0);
    assert_eq!(stats.last_hybrid_gi_voxel_dirty_clipmap_count, 0);
    assert_eq!(stats.last_hybrid_gi_voxel_invalidated_clipmap_count, 0);
}

#[test]
fn render_framework_publishes_effective_hybrid_gi_fallback_stats() {
    let server = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("hybrid-gi-fallback").with_hybrid_global_illumination(true),
        )
        .unwrap();
    let mut extract = flagship_extract();
    extract.geometry.virtual_geometry = None;
    let settings = extract
        .lighting
        .hybrid_global_illumination
        .as_mut()
        .expect("flagship fixture should request HybridGI");
    settings.mode = RenderHybridGiMode::BakedStaticDynamic;
    settings.profile = RenderHybridGiProfile::IndoorStatic;
    settings.trace_budget = 0;
    settings.card_budget = 0;
    settings.voxel_budget = 0;

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    let resolved = stats
        .last_hybrid_gi_resolved_settings
        .expect("enabled HybridGI provider must publish effective settings");

    assert_eq!(resolved.mode, RenderHybridGiMode::DynamicOnly);
    assert_eq!(resolved.profile, RenderHybridGiProfile::IndoorStatic);
    assert_eq!(resolved.quality, RenderHybridGiQuality::High);
    assert_eq!(
        (
            resolved.trace_budget,
            resolved.card_budget,
            resolved.voxel_budget,
        ),
        (64, 256, 64)
    );
    assert_eq!(
        resolved.fallback_reason,
        Some(RenderHybridGiFallbackReason::BakedLightingUnavailable)
    );
}

#[test]
fn render_framework_degrades_requested_advanced_features_without_runtime_providers() {
    let server = pluginized_wgpu_render_framework();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship-no-provider")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, flagship_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(!stats
        .last_effective_features
        .contains(&"virtual_geometry".to_string()));
    assert!(!stats
        .last_effective_features
        .contains(&"hybrid_gi".to_string()));
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::None
    );
    assert_eq!(
        stats.last_hybrid_gi_payload_source,
        RenderHybridGiPayloadSource::None
    );

    for feature in [
        AdvancedRenderFeature::VirtualGeometry,
        AdvancedRenderFeature::HybridGlobalIllumination,
    ] {
        let report = advanced_provider_report(&stats, feature);
        assert!(report.requested);
        assert_eq!(report.provider_id, None);
        assert_eq!(report.status, AdvancedProviderStatus::Degraded);
        assert!(report
            .degradations
            .iter()
            .any(|degradation| degradation.reason
                == AdvancedRenderDegradationReason::ProviderMissing));
    }
}

#[test]
fn render_framework_drops_stale_flagship_runtime_state_when_extract_removes_vg_and_hybrid_gi_payload(
) {
    let server = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true),
        )
        .unwrap();

    let active_extract = flagship_extract();
    server
        .submit_frame_extract(viewport, active_extract)
        .unwrap();
    let active_stats = server.query_stats().unwrap();
    assert_eq!(active_stats.last_virtual_geometry_page_table_entry_count, 0);
    assert_eq!(active_stats.last_hybrid_gi_cache_entry_count, 0);
    assert_eq!(active_stats.last_hybrid_gi_scene_card_count, 0);
    assert_eq!(
        active_stats.last_hybrid_gi_surface_cache_resident_page_count,
        0
    );

    server
        .submit_frame_extract(viewport, empty_flagship_extract())
        .unwrap();
    let cleared_stats = server.query_stats().unwrap();

    assert_eq!(cleared_stats.last_virtual_geometry_visible_cluster_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_requested_page_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_dirty_page_count, 0);
    assert_eq!(
        cleared_stats.last_virtual_geometry_page_table_entry_count,
        0
    );
    assert_eq!(cleared_stats.last_virtual_geometry_resident_page_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_pending_request_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_completed_page_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_replaced_page_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_cache_entry_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_resident_probe_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_pending_update_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_scheduled_trace_region_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_scene_card_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_resolved_settings, None);
    assert_eq!(
        cleared_stats.last_hybrid_gi_surface_cache_resident_page_count,
        0
    );
    assert_eq!(
        cleared_stats.last_hybrid_gi_surface_cache_dirty_page_count,
        0
    );
    assert_eq!(
        cleared_stats.last_hybrid_gi_surface_cache_feedback_card_count,
        0
    );
    assert_eq!(
        cleared_stats.last_hybrid_gi_surface_cache_capture_slot_count,
        0
    );
    assert_eq!(
        cleared_stats.last_hybrid_gi_surface_cache_invalidated_page_count,
        0
    );
    assert_eq!(cleared_stats.last_hybrid_gi_voxel_resident_clipmap_count, 0);
    assert_eq!(cleared_stats.last_hybrid_gi_voxel_dirty_clipmap_count, 0);
    assert_eq!(
        cleared_stats.last_hybrid_gi_voxel_invalidated_clipmap_count,
        0
    );
}
