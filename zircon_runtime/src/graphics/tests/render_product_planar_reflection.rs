use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::assets::{AlphaMode, MaterialAsset};
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, TextureAsset, TextureAssetDescriptor};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, EnvironmentExtract, PlanarReflectionProbeData,
    PlanarUpdateMode, PostProcessGraphResourceNames, PreviewEnvironmentExtract, ProjectionMode,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderImageColorSpace,
    RenderImageFallbackKind, RenderImageUsage, RenderLayerSet, RenderMeshSnapshot,
    RenderOverlayExtract, RenderQualityProfile, RenderSamplerDescriptor,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
    TextureMarker,
};
use crate::graphics::{
    planar_reflection_filter_compute_workload,
    planar_reflection_render_pass_executor_registrations, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassStage, WgpuRenderFramework, PLANAR_FILTER_EXECUTOR_ID,
    PLANAR_REFLECTION_TEXTURE_RESOURCE,
};
use crate::render_graph::QueueLane;

const PRODUCT_VIEWPORT_SIZE: UVec2 = UVec2::new(640, 360);
const CAPTURE_RESOLUTION: u32 = 256;
const FLOOR_CENTER_Y: f32 = -1.05;
const FLOOR_HEIGHT: f32 = 0.08;
const FLOOR_SURFACE_Y: f32 = FLOOR_CENTER_Y + FLOOR_HEIGHT * 0.5;
const PRODUCT_IMAGE_NAME: &str = "plan18_planar_mirror_floor_oblique_clip_filter_wgpu_20260712.png";
const PRODUCT_REPORT_NAME: &str =
    "plan18_planar_mirror_floor_oblique_clip_filter_wgpu_20260712.txt";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct FrameDifference {
    changed_pixel_count: u64,
    mean_absolute_rgb_error: f64,
    max_rgb_error: u8,
}

struct ProductRender {
    frame: CapturedFrame,
    stats: RenderStats,
    submit_cpu_micros: u128,
}

#[test]
fn render_product_planar_reflection_registered_but_empty_matches_baseline_exactly() {
    let baseline = render_mirror_floor(false, false);
    let registered_but_empty = render_mirror_floor(true, false);

    assert_eq!(baseline.frame.width, registered_but_empty.frame.width);
    assert_eq!(baseline.frame.height, registered_but_empty.frame.height);
    assert_eq!(
        baseline.frame.rgba, registered_but_empty.frame.rgba,
        "registering planar reflections must be byte-inert without an extracted probe"
    );
    assert!(!registered_but_empty
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == PLANAR_FILTER_EXECUTOR_ID));
}

#[test]
fn render_product_planar_reflection_changes_mirror_floor_through_camera_loop() {
    let baseline = render_mirror_floor(true, false);
    let planar = render_mirror_floor(true, true);
    let difference = frame_difference(&baseline.frame, &planar.frame);

    assert!(
        difference.changed_pixel_count > 2_000,
        "mirror capture should change a broad floor region: {difference:?}"
    );
    assert!(
        difference.mean_absolute_rgb_error > 0.35,
        "filtered planar reflection should differ visibly from the sky fallback: {difference:?}"
    );
    assert_eq!(baseline.stats.submitted_frames, 1);
    assert_eq!(planar.stats.submitted_frames, 1);
    assert_eq!(baseline.stats.last_camera_loop_submission_count, 1);
    assert_eq!(planar.stats.last_camera_loop_submission_count, 2);
}

#[test]
#[ignore = "exports the plan-18 planar-reflection WGPU product comparison and timing evidence"]
fn export_render_product_planar_reflection_mirror_floor_png() {
    let baseline = render_mirror_floor(true, false);
    let planar = render_mirror_floor(true, true);
    let difference = frame_difference(&baseline.frame, &planar.frame);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");

    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &baseline.frame, &planar.frame);
    let report_path = output_dir.join(PRODUCT_REPORT_NAME);
    fs::write(
        &report_path,
        product_report(&baseline, &planar, difference, &image_path),
    )
    .expect("planar-reflection product report should be writable");
    assert!(image_path.is_file());
    assert!(report_path.is_file());
}

fn render_mirror_floor(registered: bool, with_probe: bool) -> ProductRender {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let target = register_capture_target(&asset_manager);
    let floor = register_material(
        &asset_manager,
        "planar-floor",
        [0.72, 0.76, 0.8, 1.0],
        [0.0; 3],
        1.0,
        0.04,
    );
    let red = register_material(
        &asset_manager,
        "planar-red",
        [0.9, 0.06, 0.04, 1.0],
        [1.3, 0.025, 0.012],
        0.05,
        0.32,
    );
    let cyan = register_material(
        &asset_manager,
        "planar-cyan",
        [0.03, 0.72, 0.92, 1.0],
        [0.015, 0.85, 1.15],
        0.65,
        0.2,
    );
    let gold = register_material(
        &asset_manager,
        "planar-gold",
        [0.95, 0.57, 0.08, 1.0],
        [0.5, 0.18, 0.015],
        0.9,
        0.22,
    );

    let framework = if registered {
        WgpuRenderFramework::new_with_plugin_render_features(
            asset_manager,
            [planar_render_feature_descriptor()],
            planar_reflection_render_pass_executor_registrations(),
            Vec::new(),
        )
    } else {
        WgpuRenderFramework::new(asset_manager)
    }
    .expect("WGPU framework should initialize for the planar product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_VIEWPORT_SIZE))
        .expect("planar product viewport should be created");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("planar product profile should be accepted");

    let start = Instant::now();
    framework
        .submit_frame_extract(
            viewport,
            mirror_floor_extract(with_probe, target, floor, red, cyan, gold),
        )
        .expect("planar product frame should submit");
    let submit_cpu_micros = start.elapsed().as_micros();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();
    ProductRender {
        frame,
        stats,
        submit_cpu_micros,
    }
}

fn mirror_floor_extract(
    with_probe: bool,
    target: ResourceHandle<TextureMarker>,
    floor: ResourceHandle<MaterialMarker>,
    red: ResourceHandle<MaterialMarker>,
    cyan: ResourceHandle<MaterialMarker>,
    gold: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let environment = EnvironmentExtract::procedural_default();
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(
            Vec3::new(5.2, 3.9, 6.8),
            Vec3::new(0.0, -0.2, 0.0),
            Vec3::Y,
        ),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 52.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 80.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(PRODUCT_VIEWPORT_SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                product_mesh(
                    18_500,
                    Vec3::new(0.0, FLOOR_CENTER_Y, 0.0),
                    Vec3::new(4.8, FLOOR_HEIGHT, 4.2),
                    floor,
                ),
                product_mesh(
                    18_501,
                    Vec3::new(-1.55, 0.18, -0.1),
                    Vec3::new(1.25, 2.35, 1.25),
                    red,
                ),
                product_mesh(
                    18_502,
                    Vec3::new(0.05, 0.42, -1.05),
                    Vec3::new(1.35, 2.75, 1.35),
                    cyan,
                ),
                product_mesh(
                    18_503,
                    Vec3::new(1.55, -0.05, 0.55),
                    Vec3::splat(1.35),
                    gold,
                ),
            ],
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 18_510,
                light_id: 18_510,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                direction: Vec3::new(-0.45, -0.8, -0.3).normalize_or_zero(),
                color: Vec3::new(1.0, 0.92, 0.8),
                intensity: 2.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            }],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..Default::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            true,
            Vec4::new(0.018, 0.022, 0.03, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(if with_probe { 18_502 } else { 18_501 }),
        snapshot,
    )
    .with_viewport_size(PRODUCT_VIEWPORT_SIZE);
    if with_probe {
        extract.lighting.advanced_lighting.planar_probes = vec![PlanarReflectionProbeData {
            probe_id: 18_520,
            plane_transform: Mat4::from_translation(Vec3::new(0.0, FLOOR_SURFACE_Y, 0.0)),
            local_reference_position: Vec3::ZERO,
            bounds_min: Vec3::new(-4.8, -0.3, -4.2),
            bounds_max: Vec3::new(4.8, 0.3, 4.2),
            resolution: CAPTURE_RESOLUTION,
            update: PlanarUpdateMode::EveryFrame,
            capture_target: Some(target),
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        }];
    }
    extract
}

fn product_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    material: ResourceHandle<MaterialMarker>,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            translation,
            scale,
            ..Default::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn register_material(
    asset_manager: &ProjectAssetManager,
    label: &str,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse(&format!("res://materials/{label}.zmaterial")).unwrap();
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some(label.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive,
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri),
            material,
        )
        .unwrap();
    ResourceHandle::new(id)
}

fn register_capture_target(asset_manager: &ProjectAssetManager) -> ResourceHandle<TextureMarker> {
    let uri = AssetUri::parse("res://generated/planar-capture.texture").unwrap();
    let id = ResourceId::from_locator(&uri);
    let descriptor = TextureAssetDescriptor {
        format: "rgba8unorm".to_string(),
        color_space: RenderImageColorSpace::Linear,
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![
            RenderImageUsage::RenderTarget,
            RenderImageUsage::Sampled,
            RenderImageUsage::CopySrc,
        ],
        fallback: RenderImageFallbackKind::MissingImage,
        ..Default::default()
    };
    let asset = TextureAsset::new_rgba8(
        uri.clone(),
        CAPTURE_RESOLUTION,
        CAPTURE_RESOLUTION,
        vec![0; (CAPTURE_RESOLUTION * CAPTURE_RESOLUTION * 4) as usize],
    )
    .with_descriptor(descriptor);
    asset_manager
        .assets::<TextureAsset>()
        .insert(ResourceRecord::new(id, ResourceKind::Texture, uri), asset)
        .unwrap();
    ResourceHandle::new(id)
}

fn planar_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "planar_reflections",
        vec!["view".to_string(), "advanced_lighting".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            PLANAR_FILTER_EXECUTOR_ID,
            QueueLane::AsyncCompute,
        )
        .with_executor_id(PLANAR_FILTER_EXECUTOR_ID)
        .with_compute_workload(planar_reflection_filter_compute_workload())
        .with_side_effects()
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_storage_external_texture(PLANAR_REFLECTION_TEXTURE_RESOURCE)],
    )
    .when_advanced_lighting_planar_capture_enabled()
}

fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("advanced-lighting-planar-product")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(true)
}

fn frame_difference(baseline: &CapturedFrame, planar: &CapturedFrame) -> FrameDifference {
    assert_eq!(
        (baseline.width, baseline.height, baseline.rgba.len()),
        (planar.width, planar.height, planar.rgba.len())
    );
    let mut difference = FrameDifference::default();
    let mut absolute_error_sum = 0_u64;
    for (baseline_pixel, planar_pixel) in baseline
        .rgba
        .chunks_exact(4)
        .zip(planar.rgba.chunks_exact(4))
    {
        let mut changed = false;
        for channel in 0..3 {
            let error = baseline_pixel[channel].abs_diff(planar_pixel[channel]);
            changed |= error != 0;
            absolute_error_sum += u64::from(error);
            difference.max_rgb_error = difference.max_rgb_error.max(error);
        }
        difference.changed_pixel_count += u64::from(changed);
    }
    difference.mean_absolute_rgb_error = absolute_error_sum as f64
        / (u64::from(baseline.width) * u64::from(baseline.height) * 3).max(1) as f64;
    difference
}

fn write_side_by_side_png(path: &Path, baseline: &CapturedFrame, planar: &CapturedFrame) {
    const SEPARATOR_WIDTH: u32 = 6;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(
        baseline.width * 2 + SEPARATOR_WIDTH,
        baseline.height,
        |x, y| {
            if x < baseline.width {
                captured_pixel(baseline, x, y)
            } else if x < baseline.width + SEPARATOR_WIDTH {
                Rgba([235, 239, 245, 255])
            } else {
                captured_pixel(planar, x - baseline.width - SEPARATOR_WIDTH, y)
            }
        },
    );
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn captured_pixel(frame: &CapturedFrame, x: u32, y: u32) -> Rgba<u8> {
    let index = ((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4) as usize;
    Rgba([
        frame.rgba[index],
        frame.rgba[index + 1],
        frame.rgba[index + 2],
        frame.rgba[index + 3],
    ])
}

fn product_report(
    baseline: &ProductRender,
    planar: &ProductRender,
    difference: FrameDifference,
    image_path: &Path,
) -> String {
    format!(
        "Plan 18 AF-M4 planar-reflection WGPU product evidence\nimage={}\nviewport={}x{}\ncapture_resolution={}\nchanged_pixel_count={}\nmean_absolute_rgb_error={:.4}\nmax_rgb_error={}\nbaseline_submit_cpu_micros={}\nplanar_submit_cpu_micros={}\nbaseline_submitted_frames={}\nplanar_submitted_frames={}\nbaseline_camera_loop_submissions={}\nplanar_camera_loop_submissions={}\nterminal_graph_profile_cpu_micros={}\nterminal_passes={:?}\n",
        image_path.display(), PRODUCT_VIEWPORT_SIZE.x, PRODUCT_VIEWPORT_SIZE.y, CAPTURE_RESOLUTION,
        difference.changed_pixel_count, difference.mean_absolute_rgb_error, difference.max_rgb_error,
        baseline.submit_cpu_micros, planar.submit_cpu_micros,
        baseline.stats.submitted_frames, planar.stats.submitted_frames,
        baseline.stats.last_camera_loop_submission_count,
        planar.stats.last_camera_loop_submission_count,
        planar.stats.last_graph_execution_profile_report.total_cpu_elapsed_micros(),
        planar.stats.last_graph_executed_passes,
    )
}

fn render_product_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
