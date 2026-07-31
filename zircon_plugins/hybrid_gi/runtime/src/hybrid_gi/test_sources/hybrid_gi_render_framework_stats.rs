use std::fs;
use std::path::PathBuf;

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, CapturedFrame, DisplayMode,
    EnvironmentExtract, FallbackSkyboxKind, FrameHistoryInvalidationReason,
    PreviewEnvironmentExtract, ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality,
    RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState, RenderOverlayExtract,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager;

use super::hybrid_gi_scene_prepare_material_fixtures::{
    material_capture_test_assets, material_surface_response_test_assets,
    material_texture_capture_test_assets, model_handle,
};

const SCENE_REPRESENTATION_WGPU_PNG: &str =
    "plan18_hybrid_gi_voxel_miss_fallback_wgpu_20260707.png";
const SCENE_REPRESENTATION_WGPU_REPORT: &str =
    "plan18_hybrid_gi_voxel_miss_fallback_wgpu_20260707.txt";
const RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_PNG: &str =
    "plan18_hybrid_gi_runtime_trace_lighting_product_wgpu_20260707.png";
const RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_REPORT: &str =
    "plan18_hybrid_gi_runtime_trace_lighting_product_wgpu_20260707.txt";
const PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_PNG: &str =
    "plan18_hybrid_gi_product_composite_spatial_radiance_wgpu_20260710.png";
const PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_REPORT: &str =
    "plan18_hybrid_gi_product_composite_spatial_radiance_wgpu_20260710.txt";
const CURRENT_FRAME_POST_UBER_WGPU_PNG: &str =
    "plan18_hybrid_gi_current_frame_post_uber_wgpu_20260708.png";
const CURRENT_FRAME_POST_UBER_WGPU_REPORT: &str =
    "plan18_hybrid_gi_current_frame_post_uber_wgpu_20260708.txt";
const CURRENT_FRAME_POST_UBER_MSAA_WGPU_PNG: &str =
    "plan18_hybrid_gi_current_frame_post_uber_msaa_wgpu_20260708.png";
const CURRENT_FRAME_POST_UBER_MSAA_WGPU_REPORT: &str =
    "plan18_hybrid_gi_current_frame_post_uber_msaa_wgpu_20260708.txt";
const VOXEL_CONE_TRACE_WGPU_PNG: &str = "plan18_hybrid_gi_voxel_cone_trace_wgpu_20260708.png";
const VOXEL_CONE_TRACE_WGPU_REPORT: &str = "plan18_hybrid_gi_voxel_cone_trace_wgpu_20260708.txt";
const SURFACE_CACHE_RAY_MARCH_WGPU_PNG: &str =
    "plan18_hybrid_gi_surface_cache_ray_march_wgpu_20260708.png";
const SURFACE_CACHE_RAY_MARCH_WGPU_REPORT: &str =
    "plan18_hybrid_gi_surface_cache_ray_march_wgpu_20260708.txt";
const SURFACE_CACHE_RAY_DIRECTION_DISTRIBUTION_WGPU_PNG: &str =
    "plan18_hybrid_gi_quality_scaled_trace_rays_wgpu_20260711.png";
const SURFACE_CACHE_RAY_DIRECTION_DISTRIBUTION_WGPU_REPORT: &str =
    "plan18_hybrid_gi_quality_scaled_trace_rays_wgpu_20260711.txt";
const SURFACE_CACHE_HZB_TRACE_WGPU_PNG: &str =
    "plan18_hybrid_gi_surface_cache_hzb_trace_wgpu_20260710.png";
const SURFACE_CACHE_HZB_TRACE_WGPU_REPORT: &str =
    "plan18_hybrid_gi_surface_cache_hzb_trace_wgpu_20260710.txt";
const MAIN_SCENE_HZB_SURFACE_CACHE_TRACE_WGPU_PNG: &str =
    "plan18_hybrid_gi_main_scene_hzb_surface_cache_trace_wgpu_20260710.png";
const MAIN_SCENE_HZB_SURFACE_CACHE_TRACE_WGPU_REPORT: &str =
    "plan18_hybrid_gi_main_scene_hzb_surface_cache_trace_wgpu_20260710.txt";
const TEMPORAL_HISTORY_REJECTION_WGPU_PNG: &str =
    "plan18_hybrid_gi_temporal_history_rejection_wgpu_20260710.png";
const TEMPORAL_HISTORY_REJECTION_WGPU_REPORT: &str =
    "plan18_hybrid_gi_temporal_history_rejection_wgpu_20260710.txt";
const LOCALIZED_SUPPORT_HISTORY_WGPU_PNG: &str =
    "plan18_hybrid_gi_localized_support_history_wgpu_20260710.png";
const LOCALIZED_SUPPORT_HISTORY_WGPU_REPORT: &str =
    "plan18_hybrid_gi_localized_support_history_wgpu_20260710.txt";
const DYNAMIC_LIGHT_MATRIX_WGPU_PNG: &str =
    "plan18_hybrid_gi_scene_representation_only_forward_deferred_wgpu_20260710.png";
const DYNAMIC_LIGHT_MATRIX_WGPU_REPORT: &str =
    "plan18_hybrid_gi_scene_representation_only_forward_deferred_wgpu_20260710.txt";
const SCENE_DEPTH_SOURCE_SAMPLING_WGPU_PNG: &str =
    "plan18_hybrid_gi_scene_depth_source_sampling_wgpu_20260707.png";
const SCENE_DEPTH_SOURCE_SAMPLING_WGPU_REPORT: &str =
    "plan18_hybrid_gi_scene_depth_source_sampling_wgpu_20260707.txt";

#[path = "hybrid_gi_render_framework_stats/current_frame_post_uber_msaa.rs"]
mod current_frame_post_uber_msaa;
#[path = "hybrid_gi_render_framework_stats/dynamic_light_matrix.rs"]
mod dynamic_light_matrix;
#[path = "hybrid_gi_render_framework_stats/localized_support_history.rs"]
mod localized_support_history;
#[path = "hybrid_gi_render_framework_stats/main_scene_hzb_trace.rs"]
mod main_scene_hzb_trace;
#[path = "hybrid_gi_render_framework_stats/surface_cache_hzb_trace.rs"]
mod surface_cache_hzb_trace;
#[path = "hybrid_gi_render_framework_stats/surface_cache_ray_direction_distribution.rs"]
mod surface_cache_ray_direction_distribution;
#[path = "hybrid_gi_render_framework_stats/surface_cache_ray_march.rs"]
mod surface_cache_ray_march;
#[path = "hybrid_gi_render_framework_stats/temporal_history.rs"]
mod temporal_history;
#[path = "hybrid_gi_render_framework_stats/voxel_cone_trace.rs"]
mod voxel_cone_trace;

#[test]
fn render_framework_stats_expose_scene_representation_screen_probe_and_radiance_cache_counts() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(160, 120);
    let extract =
        scene_representation_extract(viewport_size, model, black_material, emissive_material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(
        stats.last_hybrid_gi_scene_card_count, 2,
        "expected public RenderFramework stats to expose scene-representation cards without direct renderer readback access"
    );
    assert_eq!(
        stats.last_hybrid_gi_surface_cache_resident_page_count, 1,
        "expected the HGI plugin runtime provider to project card-budgeted surface-cache residency through neutral RenderStats"
    );
    assert_eq!(
        stats.last_hybrid_gi_surface_cache_feedback_card_count, 1,
        "expected the over-budget second scene card to remain visible as plugin-owned surface-cache feedback"
    );
    assert!(stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert_eq!(
        stats.last_hybrid_gi_probe_trace_dispatch_group_count[0..2],
        [1, 1]
    );
    assert!(stats.last_hybrid_gi_probe_trace_dispatch_group_count[2] >= 1);
    assert_eq!(
        stats.last_hybrid_gi_scene_screen_probe_count, 2,
        "expected screen-probe placement from scene-representation budgets to cross only the public RenderFramework stats seam"
    );
    assert_eq!(
        stats.last_hybrid_gi_scene_radiance_cache_entry_count, 2,
        "expected one radiance-cache seed per screen probe without reopening renderer-private HGI frame internals"
    );
}

#[test]
#[ignore]
fn export_hybrid_gi_voxel_miss_fallback_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract =
        scene_representation_extract(viewport_size, model, black_material, emissive_material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();

    let first_stats = server.query_stats().unwrap();
    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(first_stats.last_hybrid_gi_scene_card_count, 2);
    assert!(first_stats.last_hybrid_gi_surface_cache_resident_page_count >= 1);
    assert!(first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(first_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert_eq!(
        first_stats.last_hybrid_gi_probe_trace_dispatch_group_count[0..2],
        [1, 1]
    );
    assert!(first_stats.last_hybrid_gi_probe_trace_dispatch_group_count[2] >= 1);
    assert!(first_stats.last_hybrid_gi_voxel_resident_clipmap_count >= 1);

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_hybrid_gi_cache_entry_count >= 1,
        "expected stateful runtime prepare collector GPU readback to feed provider cache entries"
    );

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("Wgpu scene-representation frame capture should be available");
    let metrics = frame_metrics(&frame);
    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(output_dir.join(SCENE_REPRESENTATION_WGPU_PNG), &frame);
    fs::write(
        output_dir.join(SCENE_REPRESENTATION_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\ngpu_scene_prepare_depth_trace_readback=surface_cache_depth_texture+gpu_probe_trace_tile_buffer\ngpu_probe_trace_tile_generation=generate_probe_trace_tiles_compute+indirect_args_readback\ngpu_probe_trace_tile_dispatch=trace_probe_tiles_compute+writes_probe_trace_lighting_buffer\ngpu_probe_trace_tile_surface_cache_sampling=trace_probe_tiles_compute+surface_cache_atlas_depth_texture_load\ngpu_probe_trace_tile_voxel_miss_fallback=trace_probe_tiles_compute+scene_prepare_voxel_cell_descriptor_radiance\nvalidated_surface_cache_depth_sample_count={}\nvalidated_surface_cache_texture_sampling_shader=trace_probe_tiles_shader_samples_surface_cache_atlas_and_depth_textures_exact_rgb\nvalidated_voxel_miss_fallback_shader=trace_probe_tiles_shader_uses_voxel_cell_descriptor_when_surface_cache_sample_is_invalid_exact_rgb\ngpu_scene_screen_probe_prepare_work_items=screen_probe_descriptors_to_transient_prepare_probes\nneutral_hybrid_gi_prepared_frame_sideband=provider_prepare_output_resident_screen_probes+probe_scene_data\nruntime_prepare_material_capture_context=collector_context_material_capture_seed+sample_texture_rgba_from_resource_streamer\nruntime_prepare_collector_execution=stateful_gpu_prepare_pending_readback_collected\nruntime_prepare_scene_prepare_reconstruction=deferred_pending_neutral_scene_prepare_to_internal_card_requests\nvalidated_provider_prepared_frame_resident_screen_probe_count=2\nvalidated_runtime_prepare_transient_screen_probe_count=2\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_scene_card_count={}\nlast_hybrid_gi_surface_cache_resident_page_count={}\nlast_hybrid_gi_surface_cache_feedback_card_count={}\nlast_hybrid_gi_surface_cache_depth_sample_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\nlast_hybrid_gi_voxel_resident_clipmap_count={}\n",
            SCENE_REPRESENTATION_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_scene_card_count,
            stats.last_hybrid_gi_surface_cache_resident_page_count,
            stats.last_hybrid_gi_surface_cache_feedback_card_count,
            stats.last_hybrid_gi_surface_cache_depth_sample_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblack Wgpu product frame; metrics={metrics:?}"
    );
}

#[test]
#[ignore]
fn export_hybrid_gi_runtime_trace_lighting_product_resolve_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract = scene_representation_extract_with_debug_view(
        viewport_size,
        model,
        black_material,
        emissive_material,
        RenderHybridGiDebugView::None,
    );

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();

    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first_stats = server.query_stats().unwrap();
    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(first_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(first_stats.last_hybrid_gi_scene_screen_probe_count >= 2);

    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let second_stats = server.query_stats().unwrap();
    assert!(
        second_stats.last_hybrid_gi_cache_entry_count >= 1,
        "expected first GPU trace lighting readback to become provider cache history"
    );

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    assert!(stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(stats.last_hybrid_gi_scene_screen_probe_count >= 2);
    assert!(stats.last_hybrid_gi_scene_radiance_cache_entry_count >= 2);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(
        output_dir.join(RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_PNG),
        &frame,
    );
    fs::write(
        output_dir.join(RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\nproduct_debug_view=none\nruntime_trace_lighting_readback=trace_probe_tiles_compute+writes_probe_trace_lighting_buffer\nruntime_trace_lighting_provider_history=completion_probe_trace_lighting_rgb_to_hybrid_gi_runtime_state\nruntime_trace_lighting_neutral_sideband=provider_resolve_runtime_probe_rt_lighting_rgb_to_render_hybrid_gi_prepared_frame\nruntime_trace_lighting_collector_rebuild=render_hybrid_gi_prepared_probe_rt_lighting_to_hybrid_gi_resolve_runtime\nruntime_trace_lighting_public_path=render_framework_prepare_runtime_submission_to_runtime_prepare_collector\nvalidated_provider_trace_lighting_sideband=provider_projects_probe_rt_lighting_history_into_neutral_prepared_frame_sideband\nvalidated_collector_trace_lighting_rebuild=neutral_prepared_frame_projects_to_gpu_prepare_inputs\nfirst_hybrid_gi_probe_trace_tile_count={}\nsecond_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\n",
            RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            first_stats.last_hybrid_gi_probe_trace_tile_count,
            second_stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
        ),
    )
    .unwrap();
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblack Wgpu product frame; metrics={metrics:?}"
    );
}

#[test]
#[ignore]
fn export_hybrid_gi_product_composite_spatial_radiance_wgpu_png() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let warm_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model.clone(),
        smooth_white.clone(),
        rough_white.clone(),
        RenderHybridGiDebugView::None,
        Vec3::new(1.0, 0.06, 0.03),
        false,
    );
    let cool_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model,
        smooth_white,
        rough_white,
        RenderHybridGiDebugView::None,
        Vec3::new(0.03, 0.08, 1.0),
        false,
    );
    let warm_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager.clone());
    let warm_viewport = warm_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    warm_server
        .set_quality_profile(warm_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract.clone())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract.clone())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract)
        .unwrap();
    let warm_stats = warm_server.query_stats().unwrap();
    let warm_frame = warm_server
        .capture_frame(warm_viewport)
        .unwrap()
        .expect("warm Wgpu product frame capture should be available");

    let cool_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let cool_viewport = cool_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    cool_server
        .set_quality_profile(cool_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract.clone())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract.clone())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract)
        .unwrap();
    let cool_stats = cool_server.query_stats().unwrap();
    let cool_frame = cool_server
        .capture_frame(cool_viewport)
        .unwrap()
        .expect("cool Wgpu product frame capture should be available");

    assert_eq!(warm_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(cool_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(warm_stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(cool_stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(warm_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(cool_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(warm_stats.last_hybrid_gi_scene_screen_probe_count >= 2);
    assert!(cool_stats.last_hybrid_gi_scene_screen_probe_count >= 2);

    let warm_metrics = frame_metrics(&warm_frame);
    let cool_metrics = frame_metrics(&cool_frame);
    let warm_red =
        average_region_channel(&warm_frame.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red =
        average_region_channel(&cool_frame.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue =
        average_region_channel(&warm_frame.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue =
        average_region_channel(&cool_frame.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_metrics.visible_pixels > 0 && cool_metrics.visible_pixels > 0,
        "expected nonblank Wgpu product frames; warm={warm_metrics:?}, cool={cool_metrics:?}"
    );
    assert!(
        warm_red > cool_red + 0.25,
        "expected warm scene direct-light seed to survive HGI product composite with preview direct lighting disabled; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.25,
        "expected cool scene direct-light seed to survive HGI product composite with preview direct lighting disabled; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_PNG),
        &warm_frame,
        &cool_frame,
    );
    fs::write(
        output_dir.join(PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_REPORT),
        format!(
            "png={}\nleft=warm_spatial_radiance\nright=cool_spatial_radiance\nwidth={}\nheight={}\nwarm_generation={}\ncool_generation={}\nwarm_visible_pixels={}\ncool_visible_pixels={}\nwarm_min_luma={:.2}\nwarm_max_luma={:.2}\ncool_min_luma={:.2}\ncool_max_luma={:.2}\nwarm_center_red={:.2}\ncool_center_red={:.2}\nwarm_minus_cool_red={:.2}\nwarm_center_blue={:.2}\ncool_center_blue={:.2}\ncool_minus_warm_blue={:.2}\nproduct_debug_view=none\nproduct_preview_direct_lighting=disabled_for_product_gi_isolation\ngpu_probe_trace_tile_radiance=trace_probe_tiles_compute_preserves_spatially_lit_surface_cache_radiance\ncompletion_scene_light_seed_scope=synthetic_legacy_fallback_only\nproduct_composite_source=scene_prepare_spatial_direct_radiance_to_surface_cache_trace_to_global_illumination\nlumen_reference=CompositeTraces_ScreenProbeRadianceCurrentFrame_to_FinalCompose_DiffuseIndirect\nwarm_hybrid_gi_graph_executed_pass_count={}\ncool_hybrid_gi_graph_executed_pass_count={}\nwarm_hybrid_gi_cache_entry_count={}\ncool_hybrid_gi_cache_entry_count={}\nwarm_hybrid_gi_probe_trace_tile_count={}\ncool_hybrid_gi_probe_trace_tile_count={}\nwarm_hybrid_gi_scene_screen_probe_count={}\ncool_hybrid_gi_scene_screen_probe_count={}\nwarm_hybrid_gi_scene_radiance_cache_entry_count={}\ncool_hybrid_gi_scene_radiance_cache_entry_count={}\nwarm_hybrid_gi_surface_cache_resident_page_count={}\ncool_hybrid_gi_surface_cache_resident_page_count={}\nwarm_hybrid_gi_voxel_resident_clipmap_count={}\ncool_hybrid_gi_voxel_resident_clipmap_count={}\n",
            PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_PNG,
            warm_frame.width + 1 + cool_frame.width,
            warm_frame.height,
            warm_frame.generation,
            cool_frame.generation,
            warm_metrics.visible_pixels,
            cool_metrics.visible_pixels,
            warm_metrics.min_luma,
            warm_metrics.max_luma,
            cool_metrics.min_luma,
            cool_metrics.max_luma,
            warm_red,
            cool_red,
            warm_red - cool_red,
            warm_blue,
            cool_blue,
            cool_blue - warm_blue,
            warm_stats.last_hybrid_gi_graph_executed_pass_count,
            cool_stats.last_hybrid_gi_graph_executed_pass_count,
            warm_stats.last_hybrid_gi_cache_entry_count,
            cool_stats.last_hybrid_gi_cache_entry_count,
            warm_stats.last_hybrid_gi_probe_trace_tile_count,
            cool_stats.last_hybrid_gi_probe_trace_tile_count,
            warm_stats.last_hybrid_gi_scene_screen_probe_count,
            cool_stats.last_hybrid_gi_scene_screen_probe_count,
            warm_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            cool_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            warm_stats.last_hybrid_gi_surface_cache_resident_page_count,
            cool_stats.last_hybrid_gi_surface_cache_resident_page_count,
            warm_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            cool_stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
    write_side_by_side_png(
        output_dir.join(CURRENT_FRAME_POST_UBER_WGPU_PNG),
        &warm_frame,
        &cool_frame,
    );
    fs::write(
        output_dir.join(CURRENT_FRAME_POST_UBER_WGPU_REPORT),
        format!(
            "png={}\nleft=warm_current_frame_hgi\nright=cool_current_frame_hgi\nwidth={}\nheight={}\nwarm_generation={}\ncool_generation={}\nwarm_visible_pixels={}\ncool_visible_pixels={}\nwarm_min_luma={:.2}\nwarm_max_luma={:.2}\ncool_min_luma={:.2}\ncool_max_luma={:.2}\nwarm_center_red={:.2}\ncool_center_red={:.2}\nwarm_minus_cool_red={:.2}\nwarm_center_blue={:.2}\ncool_center_blue={:.2}\ncool_minus_warm_blue={:.2}\ncurrent_frame_post_uber_input=hybrid-gi-lighting_graph_resource\ncurrent_frame_post_uber_binding=post_uber_history_global_illumination_slot_reused_for_current_frame_when_available\ncurrent_frame_fallback=history-global-illumination_when_hybrid_gi_lighting_missing\nrender_graph_route=hybrid-gi-resolve_write_texture_hybrid-gi-lighting_to_post.uber_read_texture\nstack_activation=PostProcessStackDescriptor_with_hybrid_gi_lighting_input\nshader_branch=params.hybrid_gi_counts.w_current_frame_source\nlumen_reference=CompositeTraces_ScreenProbeRadianceCurrentFrame_to_FinalCompose_DiffuseIndirect\nwarm_hybrid_gi_graph_executed_pass_count={}\ncool_hybrid_gi_graph_executed_pass_count={}\nwarm_hybrid_gi_cache_entry_count={}\ncool_hybrid_gi_cache_entry_count={}\nwarm_hybrid_gi_probe_trace_tile_count={}\ncool_hybrid_gi_probe_trace_tile_count={}\nwarm_hybrid_gi_scene_screen_probe_count={}\ncool_hybrid_gi_scene_screen_probe_count={}\nwarm_hybrid_gi_scene_radiance_cache_entry_count={}\ncool_hybrid_gi_scene_radiance_cache_entry_count={}\nwarm_hybrid_gi_surface_cache_resident_page_count={}\ncool_hybrid_gi_surface_cache_resident_page_count={}\nwarm_hybrid_gi_voxel_resident_clipmap_count={}\ncool_hybrid_gi_voxel_resident_clipmap_count={}\n",
            CURRENT_FRAME_POST_UBER_WGPU_PNG,
            warm_frame.width + 1 + cool_frame.width,
            warm_frame.height,
            warm_frame.generation,
            cool_frame.generation,
            warm_metrics.visible_pixels,
            cool_metrics.visible_pixels,
            warm_metrics.min_luma,
            warm_metrics.max_luma,
            cool_metrics.min_luma,
            cool_metrics.max_luma,
            warm_red,
            cool_red,
            warm_red - cool_red,
            warm_blue,
            cool_blue,
            cool_blue - warm_blue,
            warm_stats.last_hybrid_gi_graph_executed_pass_count,
            cool_stats.last_hybrid_gi_graph_executed_pass_count,
            warm_stats.last_hybrid_gi_cache_entry_count,
            cool_stats.last_hybrid_gi_cache_entry_count,
            warm_stats.last_hybrid_gi_probe_trace_tile_count,
            cool_stats.last_hybrid_gi_probe_trace_tile_count,
            warm_stats.last_hybrid_gi_scene_screen_probe_count,
            cool_stats.last_hybrid_gi_scene_screen_probe_count,
            warm_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            cool_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            warm_stats.last_hybrid_gi_surface_cache_resident_page_count,
            cool_stats.last_hybrid_gi_surface_cache_resident_page_count,
            warm_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            cool_stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
}

#[test]
#[ignore]
fn export_hybrid_gi_scene_depth_source_sampling_wgpu_png() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let near_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model.clone(),
        smooth_white.clone(),
        rough_white.clone(),
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.0, 0.0, -24.0),
        Vec3::new(3.0, 0.0, 0.0),
    );
    let far_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model,
        smooth_white,
        rough_white,
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.0, 0.0, 24.0),
        Vec3::new(3.0, 0.0, 0.0),
    );

    let near_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager.clone());
    let near_viewport = near_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    near_server
        .set_quality_profile(near_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract.clone())
        .unwrap();
    let near_first_stats = near_server.query_stats().unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract.clone())
        .unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract)
        .unwrap();
    let near_stats = near_server.query_stats().unwrap();
    let near_frame = near_server
        .capture_frame(near_viewport)
        .unwrap()
        .expect("near-depth Wgpu product frame capture should be available");

    let far_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let far_viewport = far_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    far_server
        .set_quality_profile(far_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract.clone())
        .unwrap();
    let far_first_stats = far_server.query_stats().unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract.clone())
        .unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract)
        .unwrap();
    let far_stats = far_server.query_stats().unwrap();
    let far_frame = far_server
        .capture_frame(far_viewport)
        .unwrap()
        .expect("far-depth Wgpu product frame capture should be available");

    assert!(near_first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(far_first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(near_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(far_stats.last_hybrid_gi_probe_trace_tile_count >= 1);

    let near_metrics = frame_metrics(&near_frame);
    let far_metrics = frame_metrics(&far_frame);
    assert!(
        near_metrics.visible_pixels > 0 && far_metrics.visible_pixels > 0,
        "expected nonblank depth-source Wgpu product frames; near={near_metrics:?}, far={far_metrics:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(SCENE_DEPTH_SOURCE_SAMPLING_WGPU_PNG),
        &near_frame,
        &far_frame,
    );
    fs::write(
        output_dir.join(SCENE_DEPTH_SOURCE_SAMPLING_WGPU_REPORT),
        format!(
            "png={}\nleft=near_scene_depth_source\nright=far_scene_depth_source\nwidth={}\nheight={}\nnear_generation={}\nfar_generation={}\nnear_visible_pixels={}\nfar_visible_pixels={}\nnear_min_luma={:.2}\nnear_max_luma={:.2}\nfar_min_luma={:.2}\nfar_max_luma={:.2}\nscene_depth_source_sampling=collect_inputs_scene_prepare_card_bounds_to_surface_cache_depth_source_rgba\nscene_depth_source_precedence=surface_cache_depth_source_samples_preferred_over_bounds_fallback\nwgpu_depth_upload=scene_prepare_surface_cache_depth_texture_upload_and_readback\ntrace_depth_consumer=trace_probe_tiles_compute_surface_cache_depth_texture_load\ndirect_dsrt_scene_depth_texture=hybrid_gi_scene_prepare_graph_executor_texture_depth_load_to_hybrid_gi_scene_buffer\nlumen_reference=ScreenProbeGather_surface_cache_trace_depth_then_composite_indirect\nnear_first_hybrid_gi_surface_cache_depth_sample_count={}\nfar_first_hybrid_gi_surface_cache_depth_sample_count={}\nnear_last_hybrid_gi_graph_executed_pass_count={}\nfar_last_hybrid_gi_graph_executed_pass_count={}\nnear_last_hybrid_gi_surface_cache_depth_sample_count={}\nfar_last_hybrid_gi_surface_cache_depth_sample_count={}\nnear_last_hybrid_gi_probe_trace_tile_count={}\nfar_last_hybrid_gi_probe_trace_tile_count={}\nnear_last_hybrid_gi_probe_trace_dispatch_group_count={:?}\nfar_last_hybrid_gi_probe_trace_dispatch_group_count={:?}\nnear_last_hybrid_gi_scene_screen_probe_count={}\nfar_last_hybrid_gi_scene_screen_probe_count={}\nnear_last_hybrid_gi_surface_cache_resident_page_count={}\nfar_last_hybrid_gi_surface_cache_resident_page_count={}\n",
            SCENE_DEPTH_SOURCE_SAMPLING_WGPU_PNG,
            near_frame.width + 1 + far_frame.width,
            near_frame.height,
            near_frame.generation,
            far_frame.generation,
            near_metrics.visible_pixels,
            far_metrics.visible_pixels,
            near_metrics.min_luma,
            near_metrics.max_luma,
            far_metrics.min_luma,
            far_metrics.max_luma,
            near_first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            far_first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            near_stats.last_hybrid_gi_graph_executed_pass_count,
            far_stats.last_hybrid_gi_graph_executed_pass_count,
            near_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            far_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            near_stats.last_hybrid_gi_probe_trace_tile_count,
            far_stats.last_hybrid_gi_probe_trace_tile_count,
            near_stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            far_stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            near_stats.last_hybrid_gi_scene_screen_probe_count,
            far_stats.last_hybrid_gi_scene_screen_probe_count,
            near_stats.last_hybrid_gi_surface_cache_resident_page_count,
            far_stats.last_hybrid_gi_surface_cache_resident_page_count,
        ),
    )
    .unwrap();
}

fn scene_representation_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    first_material: ResourceHandle<MaterialMarker>,
    second_material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    scene_representation_extract_with_debug_view(
        viewport_size,
        model,
        first_material,
        second_material,
        RenderHybridGiDebugView::SurfaceCache,
    )
}

fn scene_representation_extract_with_debug_view(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    first_material: ResourceHandle<MaterialMarker>,
    second_material: ResourceHandle<MaterialMarker>,
    debug_view: RenderHybridGiDebugView,
) -> RenderFrameExtract {
    scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model,
        first_material,
        second_material,
        debug_view,
        Vec3::ONE,
        true,
    )
}

fn scene_representation_extract_with_debug_view_and_key_light(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    first_material: ResourceHandle<MaterialMarker>,
    second_material: ResourceHandle<MaterialMarker>,
    debug_view: RenderHybridGiDebugView,
    key_light_color: Vec3,
    preview_lighting_enabled: bool,
) -> RenderFrameExtract {
    scene_representation_extract_with_card_positions(
        viewport_size,
        model,
        first_material,
        second_material,
        debug_view,
        key_light_color,
        preview_lighting_enabled,
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
    )
}

fn scene_representation_extract_with_card_positions(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    first_material: ResourceHandle<MaterialMarker>,
    second_material: ResourceHandle<MaterialMarker>,
    debug_view: RenderHybridGiDebugView,
    key_light_color: Vec3,
    preview_lighting_enabled: bool,
    first_card_translation: Vec3,
    second_card_translation: Vec3,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 6.0,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                mesh(
                    11,
                    model.clone(),
                    first_material,
                    first_card_translation,
                    2.0,
                ),
                mesh(22, model, second_material, second_card_translation, 1.0),
            ],
            directional_lights: vec![directional_key_light(key_light_color)],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: EnvironmentExtract::disabled(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: preview_lighting_enabled,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        mode: Default::default(),
        profile: Default::default(),
        quality: RenderHybridGiQuality::High,
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 1,
        debug_view,
    });
    extract
}

fn directional_key_light(color: Vec3) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 900,
        light_id: 900,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        direction: Vec3::new(-0.35, -0.65, -1.0).normalize_or_zero(),
        color,
        intensity: 4.0,
        mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    }
}

fn mesh(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    translation: Vec3,
    uniform_scale: f32,
) -> RenderMeshSnapshot {
    let transform = Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale));
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    }
}

fn hybrid_gi_only_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("hgi-scene-representation-stats")
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(true)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(true)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_async_compute(false)
}

struct TempProjectCleanup(PathBuf);

impl Drop for TempProjectCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FrameMetrics {
    visible_pixels: usize,
    min_luma: f32,
    max_luma: f32,
}

fn frame_metrics(frame: &CapturedFrame) -> FrameMetrics {
    let mut visible_pixels = 0_usize;
    let mut min_luma = f32::INFINITY;
    let mut max_luma = f32::NEG_INFINITY;

    for pixel in frame.rgba.chunks_exact(4) {
        if pixel[3] == 0 {
            continue;
        }
        visible_pixels += 1;
        let luma = 0.2126 * f32::from(pixel[0])
            + 0.7152 * f32::from(pixel[1])
            + 0.0722 * f32::from(pixel[2]);
        min_luma = min_luma.min(luma);
        max_luma = max_luma.max(luma);
    }

    if visible_pixels == 0 {
        min_luma = 0.0;
        max_luma = 0.0;
    }

    FrameMetrics {
        visible_pixels,
        min_luma,
        max_luma,
    }
}

fn average_region_channel(
    rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let start_x = ((width as f32) * x_min.clamp(0.0, 1.0)).floor() as usize;
    let end_x = ((width as f32) * x_max.clamp(0.0, 1.0)).ceil() as usize;
    let start_y = ((height as f32) * y_min.clamp(0.0, 1.0)).floor() as usize;
    let end_y = ((height as f32) * y_max.clamp(0.0, 1.0)).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in start_y.min(height)..end_y.min(height).max(start_y.min(height) + 1) {
        for x in start_x.min(width)..end_x.min(width).max(start_x.min(width) + 1) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }
    total / count as f32
}

fn write_png(path: PathBuf, frame: &CapturedFrame) {
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("captured frame rgba payload should match its dimensions");
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn write_side_by_side_png(path: PathBuf, left: &CapturedFrame, right: &CapturedFrame) {
    assert_eq!(left.height, right.height);
    assert_eq!(left.rgba.len(), (left.width * left.height * 4) as usize);
    assert_eq!(right.rgba.len(), (right.width * right.height * 4) as usize);

    let output_width = left.width + 1 + right.width;
    let mut rgba = vec![0_u8; (output_width * left.height * 4) as usize];
    for y in 0..left.height as usize {
        let output_row = y * output_width as usize * 4;
        let left_row = y * left.width as usize * 4;
        let right_row = y * right.width as usize * 4;
        let left_len = left.width as usize * 4;
        let right_len = right.width as usize * 4;
        rgba[output_row..output_row + left_len]
            .copy_from_slice(&left.rgba[left_row..left_row + left_len]);
        let separator = output_row + left_len;
        rgba[separator..separator + 4].copy_from_slice(&[255, 255, 255, 255]);
        let right_start = separator + 4;
        rgba[right_start..right_start + right_len]
            .copy_from_slice(&right.rgba[right_row..right_row + right_len]);
    }

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, left.height, rgba)
        .expect("side-by-side rgba payload should match its dimensions");
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
