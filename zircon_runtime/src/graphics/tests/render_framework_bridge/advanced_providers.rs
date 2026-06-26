use super::*;

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
        RenderHybridGiPayloadSource::Authored
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

#[test]
fn render_framework_ignores_legacy_hybrid_gi_history_while_feature_disabled() {
    let server = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("hybrid-disabled")
                .with_virtual_geometry(false)
                .with_hybrid_global_illumination(false),
        )
        .unwrap();

    let legacy_extract = hybrid_gi_history_seed_extract(UVec2::new(320, 240), [224, 112, 64]);
    server
        .submit_frame_extract(viewport, legacy_extract)
        .unwrap();
    let disabled_stats = server.query_stats().unwrap();
    assert_eq!(disabled_stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(disabled_stats.last_hybrid_gi_dirty_probe_count, 0);

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("hybrid-enabled-after-disabled")
                .with_virtual_geometry(false)
                .with_hybrid_global_illumination(true),
        )
        .unwrap();
    let legacy_extract = hybrid_gi_history_seed_extract(UVec2::new(320, 240), [224, 112, 64]);
    server
        .submit_frame_extract(viewport, legacy_extract)
        .unwrap();
    let enabled_stats = server.query_stats().unwrap();

    assert!(enabled_stats.last_hybrid_gi_requested_probe_count > 0);
    assert_eq!(
        enabled_stats.last_hybrid_gi_dirty_probe_count,
        enabled_stats.last_hybrid_gi_requested_probe_count,
        "expected old RenderHybridGiProbe fixtures submitted while Hybrid GI is disabled not to seed requested-probe history for the first enabled frame"
    );
}

#[test]
fn render_framework_hybrid_gi_second_frame_resolve_ignores_plugin_private_history() {
    let warm = render_hybrid_gi_history_capture([255, 72, 48]);
    let cool = render_hybrid_gi_history_capture([48, 96, 255]);

    let warm_red = average_region_channel(
        &warm.rgba,
        warm.width,
        warm.height,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let cool_red = average_region_channel(
        &cool.rgba,
        cool.width,
        cool.height,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let warm_blue = average_region_channel(
        &warm.rgba,
        warm.width,
        warm.height,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let cool_blue = average_region_channel(
        &cool.rgba,
        cool.width,
        cool.height,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        (warm_red - cool_red).abs() <= 0.4,
        "runtime neutral resolve should not consume plugin-private Hybrid GI completion history after the hard cutover; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (cool_blue - warm_blue).abs() <= 0.4,
        "runtime neutral resolve should not consume plugin-private Hybrid GI completion history after the hard cutover; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}
