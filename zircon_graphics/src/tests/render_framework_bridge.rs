use std::sync::Arc;

use zircon_asset::pipeline::manager::ProjectAssetManager;
use zircon_framework::render::{
    FrameHistoryHandle, RenderFrameExtract, RenderFramework, RenderFrameworkError,
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderPipelineHandle,
    RenderQualityProfile, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle,
};
use zircon_math::{UVec2, Vec3};
use zircon_scene::world::World;

use crate::runtime::WgpuRenderFramework;

#[test]
fn render_framework_tracks_viewports_and_accepts_frame_extract_submission() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
fn render_framework_uses_default_forward_plus_pipeline_when_viewport_has_no_explicit_pipeline() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
fn render_framework_reuses_frame_history_handle_for_compatible_submissions() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
fn render_framework_rotates_frame_history_handle_when_pipeline_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
fn render_framework_rejects_unknown_pipeline_handles() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    let error = server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(999))
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownPipeline { pipeline: 999 }
    );
}

#[test]
fn render_framework_accepts_built_in_deferred_pipeline_handle() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
fn headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
        .submit_frame_extract(viewport, flagship_extract())
        .unwrap();
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
        .contains(&"global_illumination".to_string()));
    assert_eq!(stats.last_virtual_geometry_visible_cluster_count, 2);
    assert_eq!(stats.last_virtual_geometry_requested_page_count, 1);
    assert_eq!(stats.last_virtual_geometry_dirty_page_count, 1);
    assert_eq!(stats.last_virtual_geometry_page_table_entry_count, 2);
    assert_eq!(stats.last_virtual_geometry_resident_page_count, 2);
    assert_eq!(stats.last_virtual_geometry_pending_request_count, 0);
    assert_eq!(stats.last_virtual_geometry_completed_page_count, 1);
    assert_eq!(
        stats.last_virtual_geometry_replaced_page_count, 1,
        "expected render-framework stats to expose how many resident pages were explicitly recycled by the GPU uploader so residency-manager cascades can observe real replacement pressure"
    );
    assert!(
        stats.last_virtual_geometry_indirect_draw_count >= 1,
        "expected VG-enabled server submission to record at least one indirect raster draw"
    );
    assert_eq!(
        stats.last_virtual_geometry_indirect_segment_count,
        stats.last_virtual_geometry_indirect_draw_count,
        "expected prepare-owned VG segments to remain the authoritative indirect submission count until explicit GPU compaction changes that contract"
    );
    assert!(
        stats.last_virtual_geometry_indirect_buffer_count >= 1,
        "expected VG-enabled server submission to record at least one shared indirect args buffer"
    );
    assert!(
        stats.last_virtual_geometry_indirect_buffer_count
            <= stats.last_virtual_geometry_indirect_draw_count,
        "expected shared indirect buffer count to stay within indirect draw count"
    );
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 2);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 1);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 1);
    assert_eq!(stats.last_hybrid_gi_cache_entry_count, 1);
    assert_eq!(stats.last_hybrid_gi_resident_probe_count, 1);
    assert_eq!(stats.last_hybrid_gi_pending_update_count, 1);
    assert_eq!(stats.last_hybrid_gi_scheduled_trace_region_count, 1);
}

#[test]
fn render_framework_drops_stale_flagship_runtime_state_when_extract_removes_vg_and_hybrid_gi_payload(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
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
        .submit_frame_extract(viewport, flagship_extract())
        .unwrap();
    let active_stats = server.query_stats().unwrap();
    assert_eq!(active_stats.last_virtual_geometry_page_table_entry_count, 2);
    assert_eq!(active_stats.last_hybrid_gi_cache_entry_count, 1);

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
}

#[test]
fn render_framework_hybrid_gi_second_frame_resolve_reuses_gpu_completed_hierarchy_history() {
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
        warm_red > cool_red + 0.4,
        "expected the second-frame neutral resolve to keep more red indirect light when the previous frame's hierarchy-aware GPU completion was seeded by a warm ancestor-trace lineage; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.4,
        "expected the second-frame neutral resolve to keep more blue indirect light when the previous frame's hierarchy-aware GPU completion was seeded by a cool ancestor-trace lineage; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn render_hybrid_gi_history_capture(
    seed_rt_lighting_rgb: [u8; 3],
) -> zircon_framework::render::CapturedFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(160, 120);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("hybrid-gi-history")
                .with_hybrid_global_illumination(true)
                .with_virtual_geometry(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )
        .unwrap();

    server
        .submit_frame_extract(
            viewport,
            hybrid_gi_history_seed_extract(viewport_size, seed_rt_lighting_rgb),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, hybrid_gi_history_resolve_extract(viewport_size))
        .unwrap();

    server
        .capture_frame(viewport)
        .unwrap()
        .expect("expected captured second-frame hybrid GI output")
}

fn empty_flagship_extract() -> RenderFrameExtract {
    let world = World::new();
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract
}

fn hybrid_gi_history_seed_extract(
    viewport_size: UVec2,
    seed_rt_lighting_rgb: [u8; 3],
) -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            hybrid_gi_probe(mesh, 100, true, Vec3::new(-0.85, 0.0, 0.0), 96),
            hybrid_gi_probe_with_parent(250, 100, mesh, false, Vec3::new(-0.25, 0.0, 0.0), 96),
            hybrid_gi_probe_with_parent(300, 250, mesh, false, Vec3::ZERO, 128),
        ],
        trace_regions: vec![
            hybrid_gi_trace_region(mesh, 40, Vec3::ZERO, 0.2),
            hybrid_gi_trace_region_with_rt_lighting(
                mesh,
                50,
                Vec3::new(-0.85, 0.0, 0.0),
                0.95,
                seed_rt_lighting_rgb,
            ),
        ],
    });
    extract
}

fn hybrid_gi_history_resolve_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![
            hybrid_gi_probe(mesh, 100, true, Vec3::new(-0.85, 0.0, 0.0), 96),
            hybrid_gi_probe_with_parent(250, 100, mesh, false, Vec3::new(-0.25, 0.0, 0.0), 96),
            hybrid_gi_probe_with_parent(300, 250, mesh, false, Vec3::ZERO, 128),
        ],
        trace_regions: Vec::new(),
    });
    extract
}

fn flagship_extract() -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            virtual_geometry_cluster(mesh, 15, 150, 1, None, Vec3::new(100.0, 0.0, 0.0), 9.0),
            virtual_geometry_cluster(mesh, 30, 300, 0, None, Vec3::ZERO, 8.0),
            virtual_geometry_cluster(mesh, 20, 200, 1, None, Vec3::new(0.1, 0.0, 0.0), 5.0),
            virtual_geometry_cluster(mesh, 10, 100, 2, None, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        pages: vec![
            virtual_geometry_page(100, false),
            virtual_geometry_page(150, false),
            virtual_geometry_page(200, true),
            virtual_geometry_page(300, false),
            virtual_geometry_page(500, true),
        ],
    });
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_gi_probe(mesh, 30, false, Vec3::ZERO, 128),
            hybrid_gi_probe(mesh, 20, true, Vec3::new(0.1, 0.0, 0.0), 64),
            hybrid_gi_probe(mesh, 10, false, Vec3::new(100.0, 0.0, 0.0), 32),
        ],
        trace_regions: vec![
            hybrid_gi_trace_region(mesh, 40, Vec3::ZERO, 8.0),
            hybrid_gi_trace_region(mesh, 50, Vec3::new(0.1, 0.0, 0.0), 5.0),
            hybrid_gi_trace_region(mesh, 60, Vec3::new(100.0, 0.0, 0.0), 10.0),
        ],
    });
    extract
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    parent_cluster_id: Option<u32>,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level,
        parent_cluster_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn hybrid_gi_probe(
    entity: u64,
    probe_id: u32,
    resident: bool,
    position: Vec3,
    ray_budget: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity,
        probe_id,
        position,
        radius: 0.5,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn hybrid_gi_probe_with_parent(
    probe_id: u32,
    parent_probe_id: u32,
    entity: u64,
    resident: bool,
    position: Vec3,
    ray_budget: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity,
        probe_id,
        position,
        radius: 0.8,
        parent_probe_id: Some(parent_probe_id),
        resident,
        ray_budget,
    }
}

fn hybrid_gi_trace_region(
    entity: u64,
    region_id: u32,
    bounds_center: Vec3,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity,
        region_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_coverage,
        rt_lighting_rgb: [0, 0, 0],
    }
}

fn hybrid_gi_trace_region_with_rt_lighting(
    entity: u64,
    region_id: u32,
    bounds_center: Vec3,
    screen_coverage: f32,
    rt_lighting_rgb: [u8; 3],
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity,
        region_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_coverage,
        rt_lighting_rgb,
    }
}

fn average_region_channel(
    rgba: &[u8],
    width: u32,
    height: u32,
    channel: usize,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let start_x = ((width as f32) * min_x).floor() as u32;
    let end_x = ((width as f32) * max_x).ceil() as u32;
    let start_y = ((height as f32) * min_y).floor() as u32;
    let end_y = ((height as f32) * max_y).ceil() as u32;
    let mut total = 0.0f32;
    let mut count = 0.0f32;

    for y in start_y.min(height)..end_y.min(height) {
        for x in start_x.min(width)..end_x.min(width) {
            let index = ((y * width + x) as usize) * 4;
            total += rgba[index + channel] as f32;
            count += 1.0;
        }
    }

    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}
