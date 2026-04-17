use zircon_math::UVec2;

use crate::{
    CapturedFrame, FrameHistoryHandle, RenderPipelineHandle, RenderQualityProfile, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle,
};

#[test]
fn stable_render_handles_and_frame_types_are_constructible() {
    let viewport = RenderViewportHandle::new(7);
    let pipeline = RenderPipelineHandle::new(11);
    let descriptor = RenderViewportDescriptor::new(UVec2::new(320, 240));
    let profile =
        RenderQualityProfile::new("editor-high").with_pipeline_asset(RenderPipelineHandle::new(11));
    let frame = CapturedFrame::new(320, 240, vec![0; 320 * 240 * 4], 3);
    let stats = RenderStats::default();

    assert_eq!(viewport.raw(), 7);
    assert_eq!(pipeline.raw(), 11);
    assert_eq!(descriptor.size, UVec2::new(320, 240));
    assert_eq!(profile.name, "editor-high");
    assert_eq!(
        profile.pipeline_override,
        Some(RenderPipelineHandle::new(11))
    );
    assert!(profile.features.clustered_lighting);
    assert!(profile.features.screen_space_ambient_occlusion);
    assert!(profile.features.history_resolve);
    assert!(!profile.features.virtual_geometry);
    assert!(!profile.features.hybrid_global_illumination);
    assert!(profile.features.allow_async_compute);
    assert_eq!(frame.generation, 3);
    assert_eq!(stats.active_viewports, 0);
    assert_eq!(stats.last_frame_history, None);
    assert!(stats.last_effective_features.is_empty());
    assert_eq!(stats.last_async_compute_pass_count, 0);
    assert_eq!(stats.last_virtual_geometry_visible_cluster_count, 0);
    assert_eq!(stats.last_virtual_geometry_requested_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_dirty_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_page_table_entry_count, 0);
    assert_eq!(stats.last_virtual_geometry_resident_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_pending_request_count, 0);
    assert_eq!(stats.last_virtual_geometry_indirect_draw_count, 0);
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_cache_entry_count, 0);
    assert_eq!(stats.last_hybrid_gi_resident_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_pending_update_count, 0);
    assert_eq!(stats.last_hybrid_gi_scheduled_trace_region_count, 0);
    assert_eq!(stats.capabilities.backend_name, "");
    assert!(stats.capabilities.queue_classes.is_empty());
    assert!(!stats.capabilities.acceleration_structures_supported);
    assert!(!stats.capabilities.virtual_geometry_supported);
    assert!(!stats.capabilities.hybrid_global_illumination_supported);

    let history = FrameHistoryHandle::new(19);
    assert_eq!(history.raw(), 19);
}

#[test]
fn quality_profile_builder_methods_override_m4_feature_toggles() {
    let profile = RenderQualityProfile::new("forward-lite")
        .with_screen_space_ambient_occlusion(false)
        .with_clustered_lighting(false)
        .with_history_resolve(false)
        .with_virtual_geometry(true)
        .with_hybrid_global_illumination(true)
        .with_async_compute(false);

    assert!(!profile.features.clustered_lighting);
    assert!(!profile.features.screen_space_ambient_occlusion);
    assert!(!profile.features.history_resolve);
    assert!(profile.features.virtual_geometry);
    assert!(profile.features.hybrid_global_illumination);
    assert!(!profile.features.allow_async_compute);
}
