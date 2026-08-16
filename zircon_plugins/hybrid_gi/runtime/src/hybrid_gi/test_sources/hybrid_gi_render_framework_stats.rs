use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, CapturedFrame, DisplayMode,
    EnvironmentExtract, FallbackSkyboxKind, FrameHistoryInvalidationReason,
    PreviewEnvironmentExtract, ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality,
    RenderHybridGiRadianceCacheGpuStage, RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    RenderOverlayExtract, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
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
const GPU_READBACK_EVIDENCE_FRAME_LIMIT: usize =
    zircon_runtime::graphics::RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES
        * 32;
const GPU_READBACK_EVIDENCE_TIMEOUT: Duration = Duration::from_secs(15);
const GPU_READBACK_EVIDENCE_POLL_INTERVAL: Duration = Duration::from_millis(1);

#[path = "hybrid_gi_render_framework_stats/current_frame_post_uber_msaa.rs"]
mod current_frame_post_uber_msaa;
#[path = "hybrid_gi_render_framework_stats/debug_views.rs"]
mod debug_views;
#[path = "hybrid_gi_render_framework_stats/dynamic_light_matrix.rs"]
mod dynamic_light_matrix;
#[path = "hybrid_gi_render_framework_stats/localized_support_history.rs"]
mod localized_support_history;
#[path = "hybrid_gi_render_framework_stats/main_scene_hzb_trace.rs"]
mod main_scene_hzb_trace;
#[path = "hybrid_gi_render_framework_stats/radiance_cache_update.rs"]
mod radiance_cache_update;
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
    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();

    let first_stats = server.query_stats().unwrap();
    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(first_stats.last_hybrid_gi_active_probe_count, 0);
    assert_eq!(first_stats.last_hybrid_gi_requested_probe_count, 0);
    assert_eq!(first_stats.last_hybrid_gi_dirty_probe_count, 0);
    assert_eq!(
        first_stats.last_hybrid_gi_scene_card_count, 2,
        "expected public RenderFramework stats to expose scene-representation cards without direct renderer readback access"
    );
    assert_eq!(
        first_stats.last_hybrid_gi_surface_cache_resident_page_count, 1,
        "expected the HGI plugin runtime provider to project card-budgeted surface-cache residency through neutral RenderStats"
    );
    assert_eq!(
        first_stats.last_hybrid_gi_surface_cache_feedback_card_count, 1,
        "expected the over-budget second scene card to remain visible as plugin-owned surface-cache feedback"
    );
    assert_eq!(
        first_stats.last_hybrid_gi_global_sdf_object_count, 2,
        "expected both canonical runtime-extract meshes to reach the HGI runtime-prepare projection"
    );

    let mut last_stats = first_stats;
    let mut gpu_readback_stats = None;
    let readback_deadline = Instant::now() + GPU_READBACK_EVIDENCE_TIMEOUT;
    for _ in 0..GPU_READBACK_EVIDENCE_FRAME_LIMIT {
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
        last_stats = server.query_stats().unwrap();
        if last_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1
            && last_stats.last_hybrid_gi_probe_trace_tile_count >= 1
        {
            gpu_readback_stats = Some(last_stats.clone());
            break;
        }
        if Instant::now() >= readback_deadline {
            break;
        }
        thread::sleep(GPU_READBACK_EVIDENCE_POLL_INTERVAL);
    }
    let stats = gpu_readback_stats.unwrap_or_else(|| {
        panic!(
            "bounded follow-up frames must publish surface-cache depth and probe-trace GPU readback; depth_samples={}, trace_tiles={}, in_flight={}, completed={}, slot_reuse_rejections={}, global_sdf_objects={}, global_sdf_resident_pages={}, global_sdf_sampleable_pages={}, global_sdf_dirty_pages={}, global_sdf_dispatched_pages={}, global_sdf_uploaded_pages={}",
            last_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            last_stats.last_hybrid_gi_probe_trace_tile_count,
            last_stats.last_readback_in_flight_count,
            last_stats.last_readback_completed_count,
            last_stats.last_readback_slot_reuse_rejection_count,
            last_stats.last_hybrid_gi_global_sdf_object_count,
            last_stats.last_hybrid_gi_global_sdf_resident_page_count,
            last_stats.last_hybrid_gi_global_sdf_sampleable_page_count,
            last_stats.last_hybrid_gi_global_sdf_dirty_page_count,
            last_stats.last_hybrid_gi_global_sdf_dispatched_page_count,
            last_stats.last_hybrid_gi_global_sdf_uploaded_page_count,
        )
    });
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
    assert!(
        stats.last_hybrid_gi_radiance_cache_resident_probe_count >= 8,
        "expected HGI diagnostics to distinguish persistent radiance-cache residency from screen-probe output entries"
    );
    assert_eq!(
        stats.last_hybrid_gi_radiance_cache_truncated_demand_count, 0,
        "expected the small fixture to fit within the private radiance-cache capacity"
    );
    assert!(
        stats.last_hybrid_gi_radiance_cache_generation > 0,
        "expected persistent radiance-cache generation to be visible through public RenderStats"
    );
}

#[path = "hybrid_gi_render_framework_stats/product_wgpu.rs"]
mod product_wgpu;

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
        static_state: RenderMeshStaticState::new(true, 1, 1),
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
