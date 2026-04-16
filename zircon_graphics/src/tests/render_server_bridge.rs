use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_math::UVec2;
use zircon_render_server::{
    FrameHistoryHandle, RenderPipelineHandle, RenderQualityProfile, RenderServer,
    RenderServerError, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

use crate::runtime::WgpuRenderServer;

#[test]
fn render_server_tracks_viewports_and_accepts_frame_extract_submission() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, RenderQualityProfile::new("editor"))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(viewport, RenderViewportHandle::new(1));
    assert_eq!(stats.active_viewports, 1);
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
    assert_eq!(stats.capabilities.backend_name, "wgpu");
    assert!(!stats.capabilities.supports_surface);
    assert!(stats.capabilities.supports_offscreen);
    assert!(!stats.capabilities.acceleration_structures_supported);
}

#[test]
fn render_server_uses_default_forward_plus_pipeline_when_viewport_has_no_explicit_pipeline() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(1)),
        "submit should fall back to the default Forward+ pipeline asset"
    );
}

#[test]
fn render_server_reuses_frame_history_handle_for_compatible_submissions() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let first = server.query_stats().unwrap().last_frame_history;

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let second = server.query_stats().unwrap().last_frame_history;

    assert_eq!(first, second);
    assert_eq!(second, Some(FrameHistoryHandle::new(1)));
}

#[test]
fn headless_wgpu_server_falls_back_async_compute_passes_to_graphics() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(!stats.capabilities.supports_async_compute);
    assert_eq!(stats.last_async_compute_pass_count, 0);
    assert!(
        stats
            .last_effective_features
            .contains(&"clustered_lighting".to_string()),
        "clustered lighting should stay enabled while queue execution falls back to graphics"
    );
}

#[test]
fn render_server_rotates_frame_history_handle_when_pipeline_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let forward_history = server.query_stats().unwrap().last_frame_history;

    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let deferred_history = server.query_stats().unwrap().last_frame_history;

    assert_ne!(forward_history, deferred_history);
    assert_eq!(forward_history, Some(FrameHistoryHandle::new(1)));
    assert_eq!(deferred_history, Some(FrameHistoryHandle::new(2)));
}

#[test]
fn quality_profile_can_disable_ssao_clustered_and_history_features() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let before = server.query_stats().unwrap().last_frame_history;

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("forward-lite")
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_ne!(before, stats.last_frame_history);
    assert!(!stats
        .last_effective_features
        .contains(&"screen_space_ambient_occlusion".to_string()));
    assert!(!stats
        .last_effective_features
        .contains(&"clustered_lighting".to_string()));
    assert!(!stats
        .last_effective_features
        .contains(&"history_resolve".to_string()));
}

#[test]
fn render_server_rejects_unknown_pipeline_handles() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    let error = server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(999))
        .unwrap_err();

    assert_eq!(error, RenderServerError::UnknownPipeline { pipeline: 999 });
}

#[test]
fn render_server_accepts_built_in_deferred_pipeline_handle() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(2)),
        "submit should honor the built-in deferred pipeline asset"
    );
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
}

#[test]
fn quality_profile_can_override_the_default_pipeline_asset() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("deferred-quality")
                .with_pipeline_asset(RenderPipelineHandle::new(2)),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(2)),
        "quality profile pipeline override should become the viewport default when no explicit pipeline is set"
    );
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
}

#[test]
fn headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
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
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(!stats.capabilities.virtual_geometry_supported);
    assert!(!stats.capabilities.hybrid_global_illumination_supported);
    assert!(!stats
        .last_effective_features
        .contains(&"virtual_geometry".to_string()));
    assert!(!stats
        .last_effective_features
        .contains(&"global_illumination".to_string()));
    assert_eq!(stats.last_virtual_geometry_visible_cluster_count, 0);
    assert_eq!(stats.last_virtual_geometry_requested_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_dirty_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_page_table_entry_count, 0);
    assert_eq!(stats.last_virtual_geometry_resident_page_count, 0);
    assert_eq!(stats.last_virtual_geometry_pending_request_count, 0);
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_cache_entry_count, 0);
    assert_eq!(stats.last_hybrid_gi_resident_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_pending_update_count, 0);
    assert_eq!(stats.last_hybrid_gi_scheduled_trace_region_count, 0);
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
