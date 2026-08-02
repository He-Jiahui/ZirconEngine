use std::{fs, path::PathBuf, sync::Arc};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::AssetUri;
use crate::core::framework::render::{
    CameraRenderDescriptor, CapturedFrame, DisplayMode, GeometryExtract, ProjectionMode,
    RenderCameraClear, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMaterialLightingModel, RenderMeshSnapshot, RenderMeshStaticState, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::WgpuRenderFramework;

use super::super::render_product_submit::{
    material_with_import_note, snapshot_with_projection_for_mesh_cache_tests,
};
use super::register_material_asset_revision;

const STATUS: &str = "render_plan08_three_shading_models_forward_deferred_parity_wgpu_passed_light_grid_fallback_renderdoc_deferred";
const PRODUCT_READBACK_PNG_STATUS: &str = "render_plan08_three_shading_models_forward_deferred_product_readback_png_passed_renderdoc_deferred";
const FORWARD_PIPELINE_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(1);
const DEFERRED_PIPELINE_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(2);

#[test]
fn render_product_three_shading_models_forward_deferred_parity() {
    assert!(!STATUS.is_empty());

    let forward = capture_three_shading_models(
        FORWARD_PIPELINE_HANDLE,
        "forward-plus",
        9_201,
        viewport_size(),
    );
    let deferred =
        capture_three_shading_models(DEFERRED_PIPELINE_HANDLE, "deferred", 9_202, viewport_size());

    assert_three_shading_model_product_stats(&forward.stats, &deferred.stats);
    assert_three_shading_model_frames(&forward.frame, &deferred.frame);
}

#[test]
#[ignore = "manual product PNG export for Plan 08 Forward/Deferred parity"]
fn export_three_shading_models_forward_deferred_product_png() {
    assert!(!PRODUCT_READBACK_PNG_STATUS.is_empty());

    let output_size = UVec2::new(320, 240);
    let forward = capture_three_shading_models(
        FORWARD_PIPELINE_HANDLE,
        "forward-plus-product-png",
        9_203,
        output_size,
    );
    let deferred = capture_three_shading_models(
        DEFERRED_PIPELINE_HANDLE,
        "deferred-product-png",
        9_204,
        output_size,
    );

    assert_three_shading_model_product_stats(&forward.stats, &deferred.stats);
    assert_three_shading_model_frames(&forward.frame, &deferred.frame);

    let output_path = render_test_output_dir()
        .join("runtime_render_plan08_three_shading_models_forward_deferred_product_20260703.png");
    save_side_by_side_product_frames(&forward.frame, &deferred.frame, &output_path);
}

struct ShadingModelCapture {
    frame: CapturedFrame,
    stats: RenderStats,
}

fn capture_three_shading_models(
    pipeline: RenderPipelineHandle,
    profile_name: &str,
    world: u64,
    output_size: UVec2,
) -> ShadingModelCapture {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    for case in shading_model_cases() {
        register_shading_model_material(&asset_manager, &case);
    }

    let framework = WgpuRenderFramework::new_for_test(asset_manager).expect("WGPU framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(output_size))
        .expect("viewport");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("set product pipeline");
    framework
        .set_quality_profile(viewport, parity_quality_profile(profile_name))
        .expect("set quality profile");
    framework
        .submit_frame_extract(viewport, three_shading_models_extract(world, output_size))
        .expect("submit three-model product extract");
    let frame = framework
        .capture_frame(viewport)
        .expect("capture call")
        .expect("captured frame");
    let stats = framework.query_stats().expect("render stats");
    framework
        .destroy_viewport(viewport)
        .expect("destroy viewport");

    ShadingModelCapture { frame, stats }
}

fn register_shading_model_material(
    asset_manager: &ProjectAssetManager,
    case: &ShadingModelParityCase,
) {
    let mut material = material_with_import_note();
    material.name = Some(format!("Plan08{}Parity", case.name));
    material.base_color = case.base_color;
    material.metallic = 0.0;
    material.roughness = 1.0;
    material.emissive = [0.0, 0.0, 0.0];
    material.validation_diagnostics.clear();
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String(case.lighting_model.as_token()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));

    register_material_asset_revision(
        asset_manager,
        case.material_id(),
        case.material_uri(),
        case.source_hash,
        material,
    );
}

fn three_shading_models_extract(world: u64, output_size: UVec2) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Perspective,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(output_size);

    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(camera.projection_mode),
    );
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(9_200), camera);
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    extract.view.select_camera_descriptor(descriptor);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        shading_model_cases()
            .iter()
            .map(shading_model_mesh)
            .collect::<Vec<_>>(),
    );
    extract
}

fn shading_model_mesh(case: &ShadingModelParityCase) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: case.node_id,
        stable_instance_key: case.node_id << 16,
        transform_revision: 1,
        transform: Transform {
            translation: Vec3::new(case.translation_x, 0.0, 0.0),
            scale: Vec3::splat(0.52),
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(case.material_id()),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}

fn parity_quality_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_clustered_lighting(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

#[derive(Clone)]
struct ShadingModelParityCase {
    name: &'static str,
    locator: &'static str,
    source_hash: &'static str,
    lighting_model: RenderMaterialLightingModel,
    base_color: [f32; 4],
    node_id: u64,
    translation_x: f32,
}

impl ShadingModelParityCase {
    fn material_uri(&self) -> AssetUri {
        AssetUri::parse(self.locator).expect("material URI")
    }

    fn material_id(&self) -> ResourceId {
        ResourceId::from_locator(&self.material_uri())
    }
}

fn shading_model_cases() -> [ShadingModelParityCase; 3] {
    [
        ShadingModelParityCase {
            name: "Pbr",
            locator: "res://materials/plan08-pbr-parity.zmaterial",
            source_hash: "plan08-pbr-parity-v1",
            lighting_model: RenderMaterialLightingModel::Pbr,
            base_color: [0.9, 0.15, 0.12, 1.0],
            node_id: 9_211,
            translation_x: -1.15,
        },
        ShadingModelParityCase {
            name: "BlinnPhong",
            locator: "res://materials/plan08-blinn-phong-parity.zmaterial",
            source_hash: "plan08-blinn-phong-parity-v1",
            lighting_model: RenderMaterialLightingModel::BlinnPhong,
            base_color: [0.12, 0.85, 0.18, 1.0],
            node_id: 9_212,
            translation_x: 0.0,
        },
        ShadingModelParityCase {
            name: "Unlit",
            locator: "res://materials/plan08-unlit-parity.zmaterial",
            source_hash: "plan08-unlit-parity-v1",
            lighting_model: RenderMaterialLightingModel::Unlit,
            base_color: [0.1, 0.28, 0.95, 1.0],
            node_id: 9_213,
            translation_x: 1.15,
        },
    ]
}

fn viewport_size() -> UVec2 {
    UVec2::new(160, 120)
}

fn render_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render");
    fs::create_dir_all(&output_dir).expect("create render test output directory");
    output_dir
}

fn assert_three_shading_model_product_stats(forward: &RenderStats, deferred: &RenderStats) {
    assert_eq!(forward.last_mesh_opaque_draw_count, 3);
    assert_eq!(deferred.last_mesh_opaque_draw_count, 3);
    assert_executed(forward, "mesh.opaque", "forward product submit");
    assert_executed(deferred, "deferred.gbuffer", "deferred product submit");
    assert_executed(deferred, "lighting.deferred", "deferred product submit");
}

fn assert_three_shading_model_frames(forward: &CapturedFrame, deferred: &CapturedFrame) {
    assert_dominant_color_visible(forward, DominantChannel::Red, "forward PBR swatch");
    assert_dominant_color_visible(
        forward,
        DominantChannel::Green,
        "forward Blinn-Phong swatch",
    );
    assert_dominant_color_visible(forward, DominantChannel::Blue, "forward Unlit swatch");
    assert_dominant_color_visible(deferred, DominantChannel::Red, "deferred PBR swatch");
    assert_dominant_color_visible(
        deferred,
        DominantChannel::Green,
        "deferred Blinn-Phong swatch",
    );
    assert_dominant_color_visible(deferred, DominantChannel::Blue, "deferred Unlit swatch");
    assert_rgba_frames_nearly_equal(deferred, forward, 4, 192);
}

fn save_side_by_side_product_frames(
    forward: &CapturedFrame,
    deferred: &CapturedFrame,
    output_path: &PathBuf,
) {
    assert_eq!(forward.width, deferred.width, "capture width mismatch");
    assert_eq!(forward.height, deferred.height, "capture height mismatch");

    const SEPARATOR_RGBA: [u8; 4] = [255, 255, 255, 255];
    let output_width = forward.width + deferred.width + 1;
    let output_height = forward.height;
    let row_bytes = (forward.width as usize) * 4;
    let mut rgba = Vec::with_capacity((output_width * output_height * 4) as usize);

    for row in 0..forward.height as usize {
        let row_start = row * row_bytes;
        let row_end = row_start + row_bytes;
        rgba.extend_from_slice(&forward.rgba[row_start..row_end]);
        rgba.extend_from_slice(&SEPARATOR_RGBA);
        rgba.extend_from_slice(&deferred.rgba[row_start..row_end]);
    }

    ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, output_height, rgba)
        .expect("combined product frame should match image dimensions")
        .save_with_format(output_path, ImageFormat::Png)
        .expect("write Plan 08 product readback PNG");
}

fn assert_executed(stats: &RenderStats, executor_id: &str, label: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "{label} should execute `{executor_id}`; executed={:?}",
        stats.last_graph_executed_executor_ids
    );
}

enum DominantChannel {
    Red,
    Green,
    Blue,
}

fn assert_dominant_color_visible(frame: &CapturedFrame, channel: DominantChannel, label: &str) {
    let count = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            let red = i16::from(pixel[0]);
            let green = i16::from(pixel[1]);
            let blue = i16::from(pixel[2]);
            match channel {
                DominantChannel::Red => red > 36 && red > green + 18 && red > blue + 18,
                DominantChannel::Green => green > 36 && green > red + 18 && green > blue + 18,
                DominantChannel::Blue => blue > 64 && blue > red + 24 && blue > green + 24,
            }
        })
        .count();
    assert!(
        count >= 32,
        "{label} should leave a visible dominant color region; count={count}"
    );
}

fn assert_rgba_frames_nearly_equal(
    actual: &CapturedFrame,
    expected: &CapturedFrame,
    per_channel_tolerance: u8,
    max_mismatched_pixels: usize,
) {
    assert_eq!(actual.width, expected.width, "frame width mismatch");
    assert_eq!(actual.height, expected.height, "frame height mismatch");
    assert_eq!(
        actual.rgba.len(),
        expected.rgba.len(),
        "rgba length mismatch"
    );

    let mut mismatch_count = 0usize;
    let mut first_mismatch = None;
    for (pixel_index, (actual_pixel, expected_pixel)) in actual
        .rgba
        .chunks_exact(4)
        .zip(expected.rgba.chunks_exact(4))
        .enumerate()
    {
        if pixel_diff_exceeds(actual_pixel, expected_pixel, per_channel_tolerance) {
            mismatch_count += 1;
            first_mismatch.get_or_insert((
                pixel_index,
                actual_pixel.to_vec(),
                expected_pixel.to_vec(),
            ));
        }
    }

    assert!(
        mismatch_count <= max_mismatched_pixels,
        "three shading-model deferred frame differs from forward reference by {mismatch_count} pixels; first mismatch={first_mismatch:?}"
    );
}

fn pixel_diff_exceeds(actual: &[u8], expected: &[u8], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .any(|(actual, expected)| actual.abs_diff(*expected) > tolerance)
}
