use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use zircon_runtime::core::framework::render::{
    AdvancedLightingExtract, CapturedFrame, EnvironmentExtract, FallbackSkyboxKind,
    LightShadowSettings, PreviewEnvironmentExtract, RenderAmbientLightSnapshot,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    RendererCommon, ShaderQualityTier, ShadowPcfQuality, ShadowResolutionTier,
    ViewportCameraSnapshot, VolumetricFogSettings, DEFAULT_RENDER_LAYER_MASK,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::WgpuRenderFramework;

use super::{
    render_feature_descriptor, render_pass_executor_registrations, FEATURE_NAME,
    INTEGRATE_EXECUTOR, INTEGRATE_PASS, LIGHT_SCATTER_EXECUTOR, LIGHT_SCATTER_PASS,
    MEDIA_INJECT_EXECUTOR, MEDIA_INJECT_PASS,
};

#[cfg(windows)]
mod renderdoc_capture;

const VIEWPORT_SIZE: UVec2 = UVec2::new(192, 128);
const EXPECTED_HIGH_QUALITY_DISPATCHES: usize = 3;
const EXPECTED_HIGH_QUALITY_DISPATCH_GROUPS: usize = 44_400;
const EXPECTED_FRAME_UPLOAD_BYTES: u64 = 624;
const PNG_NAME: &str = "plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.png";
const REPORT_NAME: &str =
    "plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt";
const TEST_ASSET_MODULE_NAME: &str = "VolumetricFogProductAssetRuntime";
const TEST_ASSET_SERVICE_NAME: &str =
    "VolumetricFogProductAssetRuntime.Manager.ProjectAssetManager";

struct ProjectAssetTestRuntime {
    _runtime: CoreRuntime,
    access: ProjectAssetManagerAccess,
}

impl ProjectAssetTestRuntime {
    fn new(manager: Arc<ProjectAssetManager>) -> Self {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new(
                    TEST_ASSET_MODULE_NAME,
                    "volumetric fog product asset runtime",
                )
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        TEST_ASSET_MODULE_NAME,
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(
                            Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                as ServiceObject,
                        )
                    }),
                )),
            )
            .expect("volumetric product ProjectAssetManager service should register");
        runtime
            .activate_module(TEST_ASSET_MODULE_NAME)
            .expect("volumetric product ProjectAssetManager module should activate");
        let core = runtime.handle();
        let handle = manager_service_handle(&core, TEST_ASSET_SERVICE_NAME)
            .expect("volumetric product ProjectAssetManager handle should resolve");
        Self {
            _runtime: runtime,
            access: ProjectAssetManagerAccess::new(core, handle),
        }
    }

    fn access(&self) -> ProjectAssetManagerAccess {
        self.access.clone()
    }
}

#[test]
#[ignore = "writes reviewed WGPU product evidence under docs/tests/runtime/render"]
fn export_volumetric_compiled_scene_window_light_shaft_perf_wgpu_png() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let asset_runtime = ProjectAssetTestRuntime::new(asset_manager.clone());
    let room_material = register_material(
        asset_manager.as_ref(),
        "res://materials/volumetric_window_room.zmaterial",
        "VolumetricWindowRoom",
        [0.18, 0.20, 0.23, 1.0],
        false,
        true,
    );
    let frame_material = register_material(
        asset_manager.as_ref(),
        "res://materials/volumetric_window_frame.zmaterial",
        "VolumetricWindowFrame",
        [0.11, 0.12, 0.13, 1.0],
        true,
        true,
    );

    let volumetric_framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [render_feature_descriptor()],
        render_pass_executor_registrations(),
        Vec::new(),
    )
    .expect("volumetric fog pluginized WGPU framework");
    let baseline_framework =
        WgpuRenderFramework::new(asset_runtime.access()).expect("baseline WGPU framework");

    let (baseline_frame, baseline_stats) = render_frame(
        &baseline_framework,
        "volumetric-window-baseline",
        window_light_shaft_extract(room_material, frame_material, false, true),
        false,
    );
    let (unshadowed_frame, _) = render_frame(
        &volumetric_framework,
        "volumetric-window-unshadowed-diagnostic",
        window_light_shaft_extract(room_material, frame_material, true, false),
        true,
    );
    let (volumetric_frame, volumetric_stats) = render_frame(
        &volumetric_framework,
        "volumetric-window-high",
        window_light_shaft_extract(room_material, frame_material, true, true),
        true,
    );

    assert_baseline_stats(&baseline_stats);
    assert_volumetric_stats(&volumetric_stats);
    let unshadowed_metrics = compare_frames(&baseline_frame, &unshadowed_frame);
    assert!(
        volumetric_scattering_gate_passed(unshadowed_metrics),
        "unshadowed volumetric reference should prove non-zero in-scattering; metrics={unshadowed_metrics:?}"
    );
    let metrics = compare_frames(&baseline_frame, &volumetric_frame);
    let product_gate_passed = volumetric_product_gate_passed(metrics);
    let output_dir = render_output_dir();
    fs::create_dir_all(&output_dir).expect("render evidence directory should be creatable");
    write_side_by_side_png(
        output_dir.join(PNG_NAME),
        &baseline_frame,
        &volumetric_frame,
    );
    fs::write(
        output_dir.join(REPORT_NAME),
        format_report(
            &baseline_stats,
            &volumetric_stats,
            unshadowed_metrics,
            metrics,
            product_gate_passed,
        ),
    )
    .expect("volumetric product report should be writable");
    assert!(
        product_gate_passed,
        "compiled-scene volumetric pass should produce a visible light-shaft/fog composite; metrics={metrics:?}"
    );
}

#[test]
#[cfg(windows)]
#[ignore = "captures the shadowed volumetric compiled scene through RenderDoc"]
fn capture_volumetric_compiled_scene_window_light_shaft_renderdoc() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let asset_runtime = ProjectAssetTestRuntime::new(asset_manager.clone());
    let room_material = register_material(
        asset_manager.as_ref(),
        "res://materials/volumetric_window_room_capture.zmaterial",
        "VolumetricWindowRoomCapture",
        [0.18, 0.20, 0.23, 1.0],
        false,
        true,
    );
    let frame_material = register_material(
        asset_manager.as_ref(),
        "res://materials/volumetric_window_frame_capture.zmaterial",
        "VolumetricWindowFrameCapture",
        [0.11, 0.12, 0.13, 1.0],
        true,
        true,
    );
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [render_feature_descriptor()],
        render_pass_executor_registrations(),
        Vec::new(),
    )
    .expect("volumetric fog RenderDoc framework");

    let capture_template =
        render_output_dir().join("plan18_af_m3_volumetric_media_dx12_renderdoc_20260716_capture");
    let ((_, stats), capture_path) =
        renderdoc_capture::capture_offscreen_frame(&capture_template, || {
            render_frame(
                &framework,
                "volumetric-window-renderdoc",
                window_light_shaft_extract(room_material, frame_material, true, true),
                // Keep both submissions in one offscreen capture so replay can compare
                // the history-prime frame with the measured product frame.
                true,
            )
        })
        .expect("RenderDoc should capture the offscreen volumetric frame");

    assert_volumetric_stats(&stats);
    assert!(
        capture_path.is_file(),
        "missing RenderDoc capture {capture_path:?}"
    );
    eprintln!("renderdoc_capture={}", capture_path.display());
}

fn volumetric_product_gate_passed(metrics: FrameComparisonMetrics) -> bool {
    volumetric_scattering_gate_passed(metrics)
        && metrics.window_light_shaft_sample_pixels > 0
        && metrics.shadow_control_sample_pixels > 0
        && metrics.window_light_shaft_brighter_pixels * 5 > metrics.window_light_shaft_sample_pixels
        && metrics.window_light_shaft_average_luma_delta > 1.5
        && metrics.window_light_shaft_average_luma_delta
            > metrics.shadow_control_average_luma_delta + 1.5
}

fn volumetric_scattering_gate_passed(metrics: FrameComparisonMetrics) -> bool {
    metrics.changed_pixels > 1_000
        && metrics.brighter_pixels > 350
        && metrics.color_shifted_pixels > 350
        && metrics.rgb_abs_delta > 18_000
}

#[test]
fn volumetric_product_metrics_distinguish_extinction_from_colored_in_scatter() {
    let baseline = comparison_frame(vec![120, 120, 120, 255, 120, 120, 120, 255]);
    let volumetric = comparison_frame(vec![90, 90, 90, 255, 115, 78, 54, 255]);

    let metrics = compare_frames(&baseline, &volumetric);

    assert_eq!(metrics.darker_pixels, 2);
    assert_eq!(metrics.color_shifted_pixels, 1);
}

#[test]
fn volumetric_product_gate_accepts_spatially_concentrated_window_light_shaft() {
    let baseline = synthetic_comparison_frame(64, 32, |_, _| [20, 20, 20, 255]);
    let volumetric = synthetic_comparison_frame(64, 32, |x, y| {
        if pixel_is_in_window_light_shaft(x, y, 64, 32) {
            [65, 45, 25, 255]
        } else {
            [26, 22, 20, 255]
        }
    });

    let metrics = compare_frames(&baseline, &volumetric);

    assert!(metrics.window_light_shaft_average_luma_delta > 10.0);
    assert!(
        metrics.window_light_shaft_average_luma_delta
            > metrics.shadow_control_average_luma_delta + 1.5
    );
    assert!(volumetric_product_gate_passed(metrics));
}

#[test]
fn volumetric_product_gate_rejects_uniform_full_frame_fog_change() {
    let baseline = synthetic_comparison_frame(64, 32, |_, _| [20, 20, 20, 255]);
    let volumetric = synthetic_comparison_frame(64, 32, |_, _| [60, 42, 24, 255]);

    let metrics = compare_frames(&baseline, &volumetric);

    assert!(metrics.changed_pixels > 1_000);
    assert!(metrics.color_shifted_pixels > 350);
    assert!(
        (metrics.window_light_shaft_average_luma_delta - metrics.shadow_control_average_luma_delta)
            .abs()
            < 0.01
    );
    assert!(!volumetric_product_gate_passed(metrics));
}

#[test]
fn volumetric_product_report_exposes_visibility_material_and_mesh_diagnostics() {
    let metrics = compare_frames(
        &comparison_frame(vec![0, 0, 0, 255, 0, 0, 0, 255]),
        &comparison_frame(vec![1, 2, 3, 255, 1, 2, 3, 255]),
    );
    let report = format_report(
        &RenderStats::default(),
        &RenderStats::default(),
        metrics,
        metrics,
        false,
    );

    for field in [
        "baseline_visibility_inputs=0",
        "baseline_materials_ready=0",
        "baseline_mesh_draws=0",
        "volumetric_visibility_inputs=0",
        "volumetric_materials_ready=0",
        "volumetric_mesh_draws=0",
    ] {
        assert!(report.contains(field), "missing report field `{field}`");
    }
}

fn comparison_frame(rgba: Vec<u8>) -> CapturedFrame {
    CapturedFrame::new(2, 1, rgba, 0)
}

fn synthetic_comparison_frame(
    width: u32,
    height: u32,
    pixel: impl Fn(u32, u32) -> [u8; 4],
) -> CapturedFrame {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&pixel(x, y));
        }
    }
    CapturedFrame::new(width, height, rgba, 0)
}

fn render_frame(
    framework: &WgpuRenderFramework,
    profile_name: &str,
    extract: RenderFrameExtract,
    prime_temporal_history: bool,
) -> (CapturedFrame, RenderStats) {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("volumetric product viewport");
    framework
        .set_quality_profile(viewport, quality_profile(profile_name))
        .expect("volumetric product quality profile");
    if prime_temporal_history {
        framework
            .submit_frame_extract(viewport, extract.clone())
            .expect("volumetric history-prime frame");
        framework
            .capture_frame(viewport)
            .expect("volumetric history-prime capture")
            .expect("volumetric history-prime frame should be capturable");
    }
    framework
        .submit_frame_extract(viewport, extract)
        .expect("volumetric product frame");
    let frame = framework
        .capture_frame(viewport)
        .expect("volumetric product capture")
        .expect("volumetric product frame should be capturable");
    let stats = framework.query_stats().expect("volumetric product stats");
    framework
        .destroy_viewport(viewport)
        .expect("volumetric product viewport destroy");
    (frame, stats)
}

fn quality_profile(name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(true)
        .with_shader_quality(ShaderQualityTier::High)
        .with_temporal_history(true)
        .with_screen_space_ambient_occlusion(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn window_light_shaft_extract(
    room_material: ResourceId,
    frame_material: ResourceId,
    volumetric_enabled: bool,
    directional_shadows: bool,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(73_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -4.4, 2.25),
                        Vec3::new(0.0, 0.55, 1.0),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    scene_mesh(
                        73_100,
                        Transform {
                            translation: Vec3::new(0.0, 0.0, -0.08),
                            scale: Vec3::new(4.8, 4.6, 0.08),
                            ..Transform::default()
                        },
                        room_material,
                    ),
                    scene_mesh(
                        73_101,
                        Transform {
                            translation: Vec3::new(-1.62, 0.95, 1.35),
                            scale: Vec3::new(0.95, 0.16, 1.45),
                            ..Transform::default()
                        },
                        frame_material,
                    ),
                    scene_mesh(
                        73_102,
                        Transform {
                            translation: Vec3::new(1.62, 0.95, 1.35),
                            scale: Vec3::new(0.95, 0.16, 1.45),
                            ..Transform::default()
                        },
                        frame_material,
                    ),
                    scene_mesh(
                        73_103,
                        Transform {
                            translation: Vec3::new(0.0, 0.95, 2.47),
                            scale: Vec3::new(0.72, 0.16, 0.34),
                            ..Transform::default()
                        },
                        frame_material,
                    ),
                    scene_mesh(
                        73_104,
                        Transform {
                            translation: Vec3::new(0.0, 0.95, 0.19),
                            scale: Vec3::new(0.72, 0.16, 0.28),
                            ..Transform::default()
                        },
                        frame_material,
                    ),
                ],
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 73_200,
                    light_id: 73_200,
                    layer_mask: default_render_layer_set(),
                    direction: Vec3::new(0.08, -0.95, -0.30).normalize(),
                    color: Vec3::new(1.0, 0.76, 0.48),
                    intensity: 7.0,
                    mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
                    shadow: directional_shadows.then_some(LightShadowSettings {
                        casts_shadow: true,
                        depth_bias: 0.0,
                        normal_bias: 0.0,
                        strength: 1.0,
                        resolution_preference: ShadowResolutionTier::T1024,
                        pcf_quality: ShadowPcfQuality::High,
                    }),
                }],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: vec![RenderAmbientLightSnapshot {
                    color: Vec3::new(0.10, 0.13, 0.18),
                    intensity: 0.08,
                    renderer_degraded: false,
                    degradation_reason: None,
                }],
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.008, 0.010, 0.016, 1.0),
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(VIEWPORT_SIZE);
    if volumetric_enabled {
        extract.lighting.advanced_lighting = AdvancedLightingExtract {
            volumetric: Some(VolumetricFogSettings {
                density: 0.035,
                albedo: Vec3::new(1.0, 0.78, 0.52),
                phase_g: 0.55,
                height_falloff: 0.025,
                scattering_intensity: 30.0,
                depth_distribution_exp: 2.0,
                temporal: true,
            }),
            volumetric_light_ids: vec![73_200],
            ..AdvancedLightingExtract::default()
        };
    }
    extract
}

fn register_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    name: &str,
    base_color: [f32; 4],
    cast_shadows: bool,
    receive_shadows: bool,
) -> ResourceId {
    let mut property_values = BTreeMap::new();
    property_values.insert("cast_shadows".to_string(), cast_shadows.into());
    property_values.insert("receive_shadows".to_string(), receive_shadows.into());
    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("volumetric product material insert");
    material_id
}

fn scene_mesh(node_id: u64, transform: Transform, material: ResourceId) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: RendererCommon {
            layer_mask: default_render_layer_set(),
            is_static: false,
            ..RendererCommon::default()
        },
    }
}

fn default_render_layer_set() -> RenderLayerSet {
    RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)
}

fn assert_baseline_stats(stats: &RenderStats) {
    assert_eq!(stats.last_volumetric_fog_compute_dispatch_count, 0);
    assert_eq!(stats.last_volumetric_fog_compute_dispatch_group_count, 0);
    assert_eq!(stats.last_volumetric_fog_uploaded_bytes, 0);
    for id in [
        MEDIA_INJECT_EXECUTOR,
        LIGHT_SCATTER_EXECUTOR,
        INTEGRATE_EXECUTOR,
    ] {
        assert!(
            !stats
                .last_graph_executed_executor_ids
                .contains(&id.to_string()),
            "baseline graph should not execute `{id}`"
        );
    }
}

fn assert_volumetric_stats(stats: &RenderStats) {
    assert!(
        stats
            .last_effective_features
            .contains(&FEATURE_NAME.to_string()),
        "volumetric feature should be effective; features={:?}",
        stats.last_effective_features
    );
    for pass in [MEDIA_INJECT_PASS, LIGHT_SCATTER_PASS, INTEGRATE_PASS] {
        assert!(
            stats.last_graph_executed_passes.contains(&pass.to_string()),
            "volumetric graph should execute `{pass}`; passes={:?}",
            stats.last_graph_executed_passes
        );
    }
    for id in [
        MEDIA_INJECT_EXECUTOR,
        LIGHT_SCATTER_EXECUTOR,
        INTEGRATE_EXECUTOR,
    ] {
        assert!(
            stats
                .last_graph_executed_executor_ids
                .contains(&id.to_string()),
            "volumetric graph should execute `{id}`; executors={:?}",
            stats.last_graph_executed_executor_ids
        );
    }
    assert_eq!(
        stats.last_volumetric_fog_compute_dispatch_count,
        EXPECTED_HIGH_QUALITY_DISPATCHES
    );
    assert_eq!(
        stats.last_volumetric_fog_compute_dispatch_group_count,
        EXPECTED_HIGH_QUALITY_DISPATCH_GROUPS
    );
    assert_eq!(
        stats.last_volumetric_fog_uploaded_bytes,
        EXPECTED_FRAME_UPLOAD_BYTES
    );
    assert!(stats.last_graph_compute_matched_workload_count >= 3);
    assert_eq!(stats.last_graph_compute_missing_dispatch_count, 0);
    assert_eq!(stats.last_graph_compute_workload_mismatch_count, 0);
    assert_eq!(stats.last_graph_compute_unexpected_dispatch_count, 0);
    assert!(stats.last_light_grid_reported);
    assert_eq!(stats.last_light_grid_light_count, 1);
    assert!(stats.last_light_grid_non_empty_cluster_count > 0);
    assert_eq!(
        stats.last_light_grid_peak_lights_per_cluster, 1,
        "directional light should cover every populated cluster"
    );
    assert!(stats.last_shadow_execution_report.shadow_pass_executed);
    assert!(stats.last_shadow_execution_report.shadow_atlas_write_count > 0);
    assert_eq!(
        stats
            .last_shadow_execution_report
            .directional_light_ready_count,
        1
    );
}

#[derive(Clone, Copy, Debug)]
struct FrameComparisonMetrics {
    changed_pixels: usize,
    brighter_pixels: usize,
    darker_pixels: usize,
    color_shifted_pixels: usize,
    rgb_abs_delta: u64,
    baseline_visible_pixels: usize,
    volumetric_visible_pixels: usize,
    baseline_max_luma: f32,
    volumetric_max_luma: f32,
    window_light_shaft_sample_pixels: usize,
    window_light_shaft_brighter_pixels: usize,
    window_light_shaft_average_luma_delta: f32,
    shadow_control_sample_pixels: usize,
    shadow_control_average_luma_delta: f32,
}

fn compare_frames(baseline: &CapturedFrame, volumetric: &CapturedFrame) -> FrameComparisonMetrics {
    assert_eq!(
        (baseline.width, baseline.height),
        (volumetric.width, volumetric.height)
    );
    let mut metrics = FrameComparisonMetrics {
        changed_pixels: 0,
        brighter_pixels: 0,
        darker_pixels: 0,
        color_shifted_pixels: 0,
        rgb_abs_delta: 0,
        baseline_visible_pixels: 0,
        volumetric_visible_pixels: 0,
        baseline_max_luma: 0.0,
        volumetric_max_luma: 0.0,
        window_light_shaft_sample_pixels: 0,
        window_light_shaft_brighter_pixels: 0,
        window_light_shaft_average_luma_delta: 0.0,
        shadow_control_sample_pixels: 0,
        shadow_control_average_luma_delta: 0.0,
    };
    let mut window_light_shaft_luma_delta = 0.0;
    let mut shadow_control_luma_delta = 0.0;
    for (pixel_index, (baseline_pixel, volumetric_pixel)) in baseline
        .rgba
        .chunks_exact(4)
        .zip(volumetric.rgba.chunks_exact(4))
        .enumerate()
    {
        let baseline_luma = rgb_luma(baseline_pixel);
        let volumetric_luma = rgb_luma(volumetric_pixel);
        let luma_delta = volumetric_luma - baseline_luma;
        let x = pixel_index as u32 % baseline.width;
        let y = pixel_index as u32 / baseline.width;
        let pixel_delta = baseline_pixel[..3]
            .iter()
            .zip(&volumetric_pixel[..3])
            .map(|(lhs, rhs)| (*lhs as i16 - *rhs as i16).unsigned_abs() as u64)
            .sum::<u64>();
        metrics.rgb_abs_delta += pixel_delta;
        metrics.changed_pixels += usize::from(pixel_delta > 3);
        metrics.brighter_pixels += usize::from(volumetric_luma > baseline_luma + 1.5);
        metrics.darker_pixels += usize::from(baseline_luma > volumetric_luma + 1.5);
        metrics.color_shifted_pixels +=
            usize::from(chromaticity_distance(baseline_pixel, volumetric_pixel) > 0.035);
        metrics.baseline_visible_pixels += usize::from(baseline_luma > 4.0);
        metrics.volumetric_visible_pixels += usize::from(volumetric_luma > 4.0);
        metrics.baseline_max_luma = metrics.baseline_max_luma.max(baseline_luma);
        metrics.volumetric_max_luma = metrics.volumetric_max_luma.max(volumetric_luma);
        if pixel_is_in_window_light_shaft(x, y, baseline.width, baseline.height) {
            metrics.window_light_shaft_sample_pixels += 1;
            metrics.window_light_shaft_brighter_pixels += usize::from(luma_delta > 1.5);
            window_light_shaft_luma_delta += luma_delta;
        }
        if pixel_is_in_shadow_control(x, y, baseline.width, baseline.height) {
            metrics.shadow_control_sample_pixels += 1;
            shadow_control_luma_delta += luma_delta;
        }
    }
    if metrics.window_light_shaft_sample_pixels > 0 {
        metrics.window_light_shaft_average_luma_delta =
            window_light_shaft_luma_delta / metrics.window_light_shaft_sample_pixels as f32;
    }
    if metrics.shadow_control_sample_pixels > 0 {
        metrics.shadow_control_average_luma_delta =
            shadow_control_luma_delta / metrics.shadow_control_sample_pixels as f32;
    }
    metrics
}

fn pixel_is_in_window_light_shaft(x: u32, y: u32, width: u32, height: u32) -> bool {
    let normalized_x = (x as f32 + 0.5) / width as f32;
    let normalized_y = (y as f32 + 0.5) / height as f32;
    if !(0.55..=0.95).contains(&normalized_y) {
        return false;
    }
    let shaft_progress = (normalized_y - 0.55) / 0.40;
    let shaft_half_width = 0.08 + shaft_progress * 0.28;
    (normalized_x - 0.5).abs() <= shaft_half_width
}

fn pixel_is_in_shadow_control(x: u32, y: u32, width: u32, height: u32) -> bool {
    let normalized_x = (x as f32 + 0.5) / width as f32;
    let normalized_y = (y as f32 + 0.5) / height as f32;
    (0.55..=0.80).contains(&normalized_y) && (normalized_x <= 0.20 || normalized_x >= 0.80)
}

fn rgb_luma(pixel: &[u8]) -> f32 {
    pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722
}

fn chromaticity_distance(lhs: &[u8], rhs: &[u8]) -> f32 {
    let lhs_sum = lhs[..3].iter().map(|value| *value as f32).sum::<f32>();
    let rhs_sum = rhs[..3].iter().map(|value| *value as f32).sum::<f32>();
    if lhs_sum <= 1.0 || rhs_sum <= 1.0 {
        return 0.0;
    }
    lhs[..3]
        .iter()
        .zip(&rhs[..3])
        .map(|(lhs, rhs)| (*lhs as f32 / lhs_sum - *rhs as f32 / rhs_sum).abs())
        .sum()
}

fn write_side_by_side_png(path: PathBuf, left: &CapturedFrame, right: &CapturedFrame) {
    assert_eq!(left.height, right.height);
    let output_width = left.width + 1 + right.width;
    let mut rgba = vec![0u8; (output_width * left.height * 4) as usize];
    for y in 0..left.height {
        let output_row = (y * output_width * 4) as usize;
        let left_row = (y * left.width * 4) as usize;
        let left_len = (left.width * 4) as usize;
        rgba[output_row..output_row + left_len]
            .copy_from_slice(&left.rgba[left_row..left_row + left_len]);

        let separator = output_row + left_len;
        rgba[separator..separator + 4].copy_from_slice(&[255, 255, 255, 255]);

        let right_row = (y * right.width * 4) as usize;
        let right_len = (right.width * 4) as usize;
        let right_start = separator + 4;
        rgba[right_start..right_start + right_len]
            .copy_from_slice(&right.rgba[right_row..right_row + right_len]);
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, left.height, rgba)
        .expect("side-by-side volumetric product payload");
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("volumetric product PNG should be writable");
}

fn format_report(
    baseline_stats: &RenderStats,
    volumetric_stats: &RenderStats,
    unshadowed_metrics: FrameComparisonMetrics,
    metrics: FrameComparisonMetrics,
    product_gate_passed: bool,
) -> String {
    let status = if product_gate_passed {
        "render_plan18_af_m3_volumetric_compiled_scene_window_light_shaft_perf_wgpu_passed"
    } else {
        "render_plan18_af_m3_volumetric_compiled_scene_window_light_shaft_perf_wgpu_diagnostic_failed"
    };
    format!(
        concat!(
            "status={}\n",
            "artifact={}\n",
            "layout=baseline_left|one_pixel_separator|volumetric_high_temporal_right\n",
            "viewport={}x{}\n",
            "reference=dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp\n",
            "scene=shadowed_directional_light_through_window_frame_into_foggy_room\n",
            "unshadowed_changed_pixels={}\n",
            "unshadowed_brighter_pixels={}\n",
            "unshadowed_color_shifted_pixels={}\n",
            "unshadowed_rgb_abs_delta={}\n",
            "baseline_visibility_inputs={}\n",
            "baseline_visibility_frustum_culled={}\n",
            "baseline_visibility_visible={}\n",
            "baseline_materials={}\n",
            "baseline_materials_ready={}\n",
            "baseline_material_fallbacks={}\n",
            "baseline_mesh_draws={}\n",
            "baseline_mesh_opaque_draws={}\n",
            "baseline_mesh_commands={}\n",
            "baseline_volumetric_dispatches={}\n",
            "baseline_volumetric_dispatch_groups={}\n",
            "baseline_volumetric_uploaded_bytes={}\n",
            "volumetric_visibility_inputs={}\n",
            "volumetric_visibility_frustum_culled={}\n",
            "volumetric_visibility_visible={}\n",
            "volumetric_materials={}\n",
            "volumetric_materials_ready={}\n",
            "volumetric_material_fallbacks={}\n",
            "volumetric_mesh_draws={}\n",
            "volumetric_mesh_opaque_draws={}\n",
            "volumetric_mesh_commands={}\n",
            "volumetric_dispatches={}\n",
            "volumetric_dispatch_groups={}\n",
            "volumetric_uploaded_bytes={}\n",
            "matched_compute_workloads={}\n",
            "missing_compute_dispatches={}\n",
            "mismatched_compute_workloads={}\n",
            "unexpected_compute_dispatches={}\n",
            "light_grid_lights={}\n",
            "light_grid_non_empty_clusters={}\n",
            "light_grid_peak_lights_per_cluster={}\n",
            "shadow_pass_count={}\n",
            "shadow_atlas_write_count={}\n",
            "shadow_caster_draw_count={}\n",
            "shadow_directional_light_ready_count={}\n",
            "changed_pixels={}\n",
            "brighter_pixels={}\n",
            "darker_pixels={}\n",
            "color_shifted_pixels={}\n",
            "rgb_abs_delta={}\n",
            "baseline_visible_pixels={}\n",
            "volumetric_visible_pixels={}\n",
            "baseline_max_luma={:.3}\n",
            "volumetric_max_luma={:.3}\n",
            "window_light_shaft_sample_pixels={}\n",
            "window_light_shaft_brighter_pixels={}\n",
            "window_light_shaft_average_luma_delta={:.3}\n",
            "shadow_control_sample_pixels={}\n",
            "shadow_control_average_luma_delta={:.3}\n",
            "window_light_shaft_shadow_contrast_luma={:.3}\n",
        ),
        status,
        PNG_NAME,
        VIEWPORT_SIZE.x,
        VIEWPORT_SIZE.y,
        unshadowed_metrics.changed_pixels,
        unshadowed_metrics.brighter_pixels,
        unshadowed_metrics.color_shifted_pixels,
        unshadowed_metrics.rgb_abs_delta,
        baseline_stats.last_visibility_input_count,
        baseline_stats.last_visibility_frustum_culled_count,
        baseline_stats.last_visibility_visible_count,
        baseline_stats.last_material_count,
        baseline_stats.last_material_ready_count,
        baseline_stats.last_material_fallback_count,
        baseline_stats.last_mesh_draw_count,
        baseline_stats.last_mesh_opaque_draw_count,
        baseline_stats.last_mesh_command_count,
        baseline_stats.last_volumetric_fog_compute_dispatch_count,
        baseline_stats.last_volumetric_fog_compute_dispatch_group_count,
        baseline_stats.last_volumetric_fog_uploaded_bytes,
        volumetric_stats.last_visibility_input_count,
        volumetric_stats.last_visibility_frustum_culled_count,
        volumetric_stats.last_visibility_visible_count,
        volumetric_stats.last_material_count,
        volumetric_stats.last_material_ready_count,
        volumetric_stats.last_material_fallback_count,
        volumetric_stats.last_mesh_draw_count,
        volumetric_stats.last_mesh_opaque_draw_count,
        volumetric_stats.last_mesh_command_count,
        volumetric_stats.last_volumetric_fog_compute_dispatch_count,
        volumetric_stats.last_volumetric_fog_compute_dispatch_group_count,
        volumetric_stats.last_volumetric_fog_uploaded_bytes,
        volumetric_stats.last_graph_compute_matched_workload_count,
        volumetric_stats.last_graph_compute_missing_dispatch_count,
        volumetric_stats.last_graph_compute_workload_mismatch_count,
        volumetric_stats.last_graph_compute_unexpected_dispatch_count,
        volumetric_stats.last_light_grid_light_count,
        volumetric_stats.last_light_grid_non_empty_cluster_count,
        volumetric_stats.last_light_grid_peak_lights_per_cluster,
        volumetric_stats
            .last_shadow_execution_report
            .shadow_pass_count,
        volumetric_stats
            .last_shadow_execution_report
            .shadow_atlas_write_count,
        volumetric_stats
            .last_shadow_execution_report
            .caster_draw_count,
        volumetric_stats
            .last_shadow_execution_report
            .directional_light_ready_count,
        metrics.changed_pixels,
        metrics.brighter_pixels,
        metrics.darker_pixels,
        metrics.color_shifted_pixels,
        metrics.rgb_abs_delta,
        metrics.baseline_visible_pixels,
        metrics.volumetric_visible_pixels,
        metrics.baseline_max_luma,
        metrics.volumetric_max_luma,
        metrics.window_light_shaft_sample_pixels,
        metrics.window_light_shaft_brighter_pixels,
        metrics.window_light_shaft_average_luma_delta,
        metrics.shadow_control_sample_pixels,
        metrics.shadow_control_average_luma_delta,
        metrics.window_light_shaft_average_luma_delta - metrics.shadow_control_average_luma_delta,
    )
}

fn render_output_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(5)
        .expect("volumetric plugin manifest should be nested under repository root")
        .join("docs/tests/runtime/render")
}
