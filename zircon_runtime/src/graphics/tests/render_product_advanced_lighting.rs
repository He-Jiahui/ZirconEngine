mod advanced_pbr;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, GeometryExtract, OitBufferPlan, OitSettings,
    PostProcessGraphResourceNames, ProjectionMode, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMaterialAlphaMode, RenderQualityProfile, RenderSpriteAnchor,
    RenderSpriteImageMode, RenderSpriteSnapshot, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, SpriteExtract,
};
use crate::core::math::{Quat, Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::{
    oit_render_pass_executor_registrations, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, WgpuRenderFramework,
};
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

const PRODUCT_VIEWPORT_SIZE: UVec2 = UVec2::new(640, 360);
const OIT_FRAGMENT_STORE_PASS: &str = "oit.fragment_store";
const OIT_RESOLVE_PASS: &str = "oit.resolve";
const SORTED_TRANSPARENT_PASS: &str = "transparent-mesh";
const PRODUCT_IMAGE_NAME: &str =
    "plan18_oit_three_crossing_transparent_planes_sorted_vs_oit_wgpu_20260712.png";
const PRODUCT_REPORT_NAME: &str =
    "plan18_oit_three_crossing_transparent_planes_sorted_vs_oit_wgpu_20260712.txt";

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
fn render_product_advanced_lighting_oit_feature_off_matches_sorted_baseline_exactly() {
    let sorted = render_crossing_planes(false, None);
    let feature_registered_but_off = render_crossing_planes(true, None);

    assert_eq!(sorted.frame.width, feature_registered_but_off.frame.width);
    assert_eq!(sorted.frame.height, feature_registered_but_off.frame.height);
    assert_eq!(
        sorted.frame.rgba, feature_registered_but_off.frame.rgba,
        "registering OIT must not alter a camera that has OIT disabled"
    );
    assert!(!feature_registered_but_off
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == OIT_FRAGMENT_STORE_PASS || pass == OIT_RESOLVE_PASS));
    assert!(feature_registered_but_off
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == SORTED_TRANSPARENT_PASS));
}

#[test]
fn render_product_advanced_lighting_oit_three_crossing_planes_differs_from_sorted() {
    let settings = product_oit_settings();
    let sorted = render_crossing_planes(false, Some(settings));
    let oit = render_crossing_planes(true, Some(settings));
    let difference = frame_difference(&sorted.frame, &oit.frame);

    assert!(sorted
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == SORTED_TRANSPARENT_PASS));
    assert!(!sorted
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == OIT_FRAGMENT_STORE_PASS || pass == OIT_RESOLVE_PASS));
    assert!(oit
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == OIT_FRAGMENT_STORE_PASS));
    assert!(oit
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == OIT_RESOLVE_PASS));
    assert!(!oit
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| pass == SORTED_TRANSPARENT_PASS));
    assert!(oit
        .stats
        .last_effective_features
        .iter()
        .any(|name| name == "oit"));
    assert!(
        difference.changed_pixel_count > 1_000,
        "crossing planes should expose a broad per-pixel ordering difference: {difference:?}"
    );
    assert!(
        difference.mean_absolute_rgb_error > 0.5,
        "OIT output should be visibly distinct from object-sorted transparency: {difference:?}"
    );
}

#[test]
#[ignore = "exports the plan-18 OIT WGPU product comparison and performance evidence"]
fn export_render_product_advanced_lighting_oit_three_crossing_planes_png() {
    let settings = product_oit_settings();
    let sorted = render_crossing_planes(false, Some(settings));
    let oit = render_crossing_planes(true, Some(settings));
    let difference = frame_difference(&sorted.frame, &oit.frame);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");

    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &sorted.frame, &oit.frame);

    let report_path = output_dir.join(PRODUCT_REPORT_NAME);
    let report = product_report(&sorted, &oit, settings, difference, &image_path);
    fs::write(&report_path, report).expect("OIT product report should be writable");

    assert!(image_path.is_file(), "OIT product PNG was not exported");
    assert!(report_path.is_file(), "OIT product report was not exported");
}

fn render_crossing_planes(registered_oit: bool, settings: Option<OitSettings>) -> ProductRender {
    let framework = if registered_oit {
        WgpuRenderFramework::new_for_test_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [oit_render_feature_descriptor()],
            oit_render_pass_executor_registrations(),
            Vec::new(),
        )
    } else {
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
    }
    .expect("WGPU framework should initialize for the OIT product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_VIEWPORT_SIZE))
        .expect("OIT product viewport should be created");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("OIT product quality profile should be accepted");

    let start = Instant::now();
    framework
        .submit_frame_extract(viewport, crossing_planes_extract(settings))
        .expect("OIT product frame should submit");
    let submit_cpu_micros = start.elapsed().as_micros();
    let frame = framework
        .capture_frame(viewport)
        .expect("OIT product frame capture should succeed")
        .expect("OIT product viewport should expose a captured frame");
    let stats = framework
        .query_stats()
        .expect("OIT product stats should be available");
    framework
        .destroy_viewport(viewport)
        .expect("OIT product viewport should be destroyed");

    ProductRender {
        frame,
        stats,
        submit_cpu_micros,
    }
}

fn crossing_planes_extract(settings: Option<OitSettings>) -> RenderFrameExtract {
    let mut snapshot = super::render_product_submit::snapshot_with_projection_for_sprite_tests(
        ProjectionMode::Orthographic,
    );
    snapshot.scene.camera.core_pipeline = CorePipelineKind::Core3d;
    snapshot.scene.camera.transform = Transform::default();
    snapshot.scene.camera.ortho_size = 5.0;
    snapshot.overlays.display_mode = DisplayMode::Shaded;
    snapshot.preview.lighting_enabled = false;
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.clear_color = Vec4::new(0.025, 0.03, 0.04, 1.0);

    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(18_400), snapshot)
            .with_viewport_size(PRODUCT_VIEWPORT_SIZE);
    extract.geometry = GeometryExtract::from_meshes(CorePipelineKind::Core3d, Vec::new());
    extract.sprites = SpriteExtract::from_sprites(
        CorePipelineKind::Core3d,
        vec![
            crossing_plane(
                18_401,
                Quat::from_rotation_y(52.0_f32.to_radians()),
                Vec4::new(1.0, 0.08, 0.06, 0.56),
                0,
            ),
            crossing_plane(
                18_402,
                Quat::from_rotation_y(-52.0_f32.to_radians()),
                Vec4::new(0.04, 0.95, 0.22, 0.56),
                1,
            ),
            crossing_plane(
                18_403,
                Quat::from_rotation_x(58.0_f32.to_radians()),
                Vec4::new(0.08, 0.26, 1.0, 0.56),
                2,
            ),
        ],
    );
    extract.lighting.advanced_lighting.oit = settings;
    extract
}

fn crossing_plane(entity: u64, rotation: Quat, color: Vec4, z_order: i32) -> RenderSpriteSnapshot {
    RenderSpriteSnapshot {
        entity,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -4.0)).with_rotation(rotation),
        image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "res://textures/oit-product-white.png",
        )),
        material: None,
        atlas_region: None,
        rect: None,
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::CENTER,
        custom_size: Some(Vec2::new(4.6, 4.6)),
        image_mode: RenderSpriteImageMode::Stretch,
        color,
        z_order,
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    }
}

fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("advanced-lighting-oit-product")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn product_oit_settings() -> OitSettings {
    OitSettings {
        fragments_per_pixel_average: 4.0,
        sorted_fragment_max_count: 8,
        alpha_threshold: 0.0,
    }
}

fn oit_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "oit",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "advanced_lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                OIT_FRAGMENT_STORE_PASS,
                QueueLane::Graphics,
            )
            .with_executor_id(OIT_FRAGMENT_STORE_PASS)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .write_buffer(PostProcessGraphResourceNames::OIT_LAYERS)
            .write_buffer(PostProcessGraphResourceNames::OIT_COUNTS),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                OIT_RESOLVE_PASS,
                QueueLane::Graphics,
            )
            .with_executor_id(OIT_RESOLVE_PASS)
            .read_buffer(PostProcessGraphResourceNames::OIT_LAYERS)
            .read_buffer(PostProcessGraphResourceNames::OIT_COUNTS)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::StorageBuffers)
    .when_advanced_lighting_oit_enabled()
    .with_replaced_pass(SORTED_TRANSPARENT_PASS)
}

fn frame_difference(sorted: &CapturedFrame, oit: &CapturedFrame) -> FrameDifference {
    assert_eq!(sorted.width, oit.width);
    assert_eq!(sorted.height, oit.height);
    assert_eq!(sorted.rgba.len(), oit.rgba.len());

    let mut difference = FrameDifference::default();
    let mut absolute_error_sum = 0_u64;
    for (sorted_pixel, oit_pixel) in sorted.rgba.chunks_exact(4).zip(oit.rgba.chunks_exact(4)) {
        let mut pixel_changed = false;
        for channel in 0..3 {
            let error = sorted_pixel[channel].abs_diff(oit_pixel[channel]);
            pixel_changed |= error != 0;
            absolute_error_sum = absolute_error_sum.saturating_add(u64::from(error));
            difference.max_rgb_error = difference.max_rgb_error.max(error);
        }
        if pixel_changed {
            difference.changed_pixel_count = difference.changed_pixel_count.saturating_add(1);
        }
    }
    let rgb_sample_count = u64::from(sorted.width)
        .saturating_mul(u64::from(sorted.height))
        .saturating_mul(3);
    difference.mean_absolute_rgb_error = absolute_error_sum as f64 / rgb_sample_count.max(1) as f64;
    difference
}

fn write_side_by_side_png(path: &Path, sorted: &CapturedFrame, oit: &CapturedFrame) {
    assert_eq!(sorted.width, oit.width);
    assert_eq!(sorted.height, oit.height);
    const SEPARATOR_WIDTH: u32 = 6;
    let output_width = sorted
        .width
        .saturating_mul(2)
        .saturating_add(SEPARATOR_WIDTH);
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(output_width, sorted.height, |x, y| {
        if x < sorted.width {
            captured_pixel(sorted, x, y)
        } else if x < sorted.width + SEPARATOR_WIDTH {
            Rgba([235, 239, 245, 255])
        } else {
            captured_pixel(oit, x - sorted.width - SEPARATOR_WIDTH, y)
        }
    });
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("OIT side-by-side PNG should be writable");
}

fn captured_pixel(frame: &CapturedFrame, x: u32, y: u32) -> Rgba<u8> {
    let index = (u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4;
    let index = usize::try_from(index).expect("captured pixel index should fit usize");
    Rgba([
        frame.rgba[index],
        frame.rgba[index + 1],
        frame.rgba[index + 2],
        frame.rgba[index + 3],
    ])
}

fn product_report(
    sorted: &ProductRender,
    oit: &ProductRender,
    settings: OitSettings,
    difference: FrameDifference,
    image_path: &Path,
) -> String {
    let buffer_plan =
        OitBufferPlan::for_view([PRODUCT_VIEWPORT_SIZE.x, PRODUCT_VIEWPORT_SIZE.y], settings);
    format!(
        concat!(
            "Plan 18 AF-M4 OIT WGPU product evidence\n",
            "image={}\n",
            "viewport={}x{}\n",
            "fragments_per_pixel_average={}\n",
            "sorted_fragment_max_count={}\n",
            "changed_pixel_count={}\n",
            "mean_absolute_rgb_error={:.4}\n",
            "max_rgb_error={}\n",
            "sorted_submit_cpu_micros={}\n",
            "oit_submit_cpu_micros={}\n",
            "sorted_graph_profile_cpu_micros={}\n",
            "oit_graph_profile_cpu_micros={}\n",
            "oit_fragment_store_cpu_micros={}\n",
            "oit_resolve_cpu_micros={}\n",
            "oit_expected_layer_buffer_bytes={}\n",
            "oit_expected_count_buffer_bytes={}\n",
            "oit_graph_transient_buffer_bytes_reserved={}\n",
            "sorted_passes={:?}\n",
            "oit_passes={:?}\n"
        ),
        image_path.display(),
        PRODUCT_VIEWPORT_SIZE.x,
        PRODUCT_VIEWPORT_SIZE.y,
        settings.fragments_per_pixel_average,
        settings.sorted_fragment_max_count,
        difference.changed_pixel_count,
        difference.mean_absolute_rgb_error,
        difference.max_rgb_error,
        sorted.submit_cpu_micros,
        oit.submit_cpu_micros,
        sorted
            .stats
            .last_graph_execution_profile_report
            .total_cpu_elapsed_micros(),
        oit.stats
            .last_graph_execution_profile_report
            .total_cpu_elapsed_micros(),
        pass_cpu_micros(&oit.stats, OIT_FRAGMENT_STORE_PASS),
        pass_cpu_micros(&oit.stats, OIT_RESOLVE_PASS),
        buffer_plan.layer_buffer_size_bytes,
        buffer_plan.count_buffer_size_bytes,
        oit.stats.last_graph_transient_buffer_bytes_reserved,
        sorted.stats.last_graph_executed_passes,
        oit.stats.last_graph_executed_passes,
    )
}

fn pass_cpu_micros(stats: &RenderStats, pass_name: &str) -> u64 {
    stats
        .last_graph_execution_profile_report
        .pass_profiles
        .iter()
        .filter(|profile| profile.pass_name == pass_name)
        .fold(0_u64, |total, profile| {
            total.saturating_add(profile.cpu_elapsed_micros)
        })
}

fn render_product_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should have a workspace parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
