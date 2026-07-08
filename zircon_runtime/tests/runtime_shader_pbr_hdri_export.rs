use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_irradiance_cube, source_cubemap_face_mip_offset, source_cubemap_mip_size,
    CubemapFace, EnvironmentExtract, PreviewEnvironmentExtract, ProjectionMode,
    RenderOverlayExtract, SceneViewportExtractRequest, SourceCubemapEnvironment,
    SourceCubemapMipChain, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::graphics::{SceneRenderer, ViewportFrame};

#[path = "runtime_shader_pbr_hdri_export/fixture_assets.rs"]
mod fixture_assets;
#[path = "runtime_shader_pbr_hdri_export/frame_assertions.rs"]
mod frame_assertions;
#[path = "runtime_shader_pbr_hdri_export/hdri_metrics.rs"]
mod hdri_metrics;
#[path = "runtime_shader_pbr_hdri_export/scene_fixtures.rs"]
mod scene_fixtures;

use fixture_assets::{
    ambientcg_metal009_texture_uri, ambientcg_metal_texture_uri,
    write_ambientcg_metal009_texture_assets, write_ambientcg_metal_texture_assets,
    AmbientCgMetalFixture, AMBIENTCG_METAL008, AMBIENTCG_METAL009_COLOR,
    AMBIENTCG_METAL009_METALLIC_ROUGHNESS, AMBIENTCG_METAL009_NORMAL_GL, AMBIENTCG_METAL025,
    AMBIENTCG_METAL029,
};
use frame_assertions::{
    assert_mirror_sphere_matches_source_reference,
    assert_mirror_sphere_matches_source_reference_with_camera_view,
    assert_mirror_sphere_reflection_orientation, assert_single_sphere_reflects_environment,
    assert_textured_material_has_surface_variation,
};
use scene_fixtures::{
    write_pbr_matrix_material, write_pbr_matrix_scene, write_single_pbr_material,
    write_single_pbr_sphere_scene_with_camera_view, write_uv_sphere_model,
    SinglePbrSphereCameraView,
};

const PBR_MATRIX_DIMENSION: usize = 10;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1600, 1200);
const PBR_MATRIX_ORTHO_SIZE: f32 = 7.2;
const PBR_MATRIX_STEP_X: f32 = 0.7;
const PBR_MATRIX_STEP_Y: f32 = 0.62;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.21;
const PBR_MATRIX_CELL_SAMPLE_SIZE: u32 = 56;
const PBR_MATRIX_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_10x10_cosine_pmrem_reflection_20260706.png";
const PBR_MATRIX_HDRI_2K_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_2k_10x10_cosine_pmrem_reflection_20260706.png";
const PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png";
const PBR_SINGLE_HDRI_OUTPUT_SIZE: UVec2 = UVec2::new(1280, 960);
const PBR_SINGLE_SPHERE_RINGS: usize = 96;
const PBR_SINGLE_SPHERE_SEGMENTS: usize = 192;
const PBR_SINGLE_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_single_metal_sphere_reflection_20260706.png";
const PBR_TEXTURED_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260706.png";
const PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png";
const PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png";
const PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_multi_view_reflection_20260707.png";
const PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_cardinal_120deg_reflection_20260708.png";
const PBR_MIRROR_MULTI_VIEW_TILE_SIZE: UVec2 = UVec2::new(800, 600);
const PBR_MIRROR_MULTI_VIEW_COLUMNS: u32 = 2;
const PBR_TEXTURED_HDRI_METAL008_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal008_texture_maps_20260707.png";
const PBR_TEXTURED_HDRI_METAL025_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal025_texture_maps_20260707.png";
const PBR_TEXTURED_HDRI_METAL029_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_ambientcg_metal029_texture_maps_20260707.png";
const POLYHAVEN_LAKES_1K_HDRI_ASSET: &str = "polyhaven_lakes_1k.hdr";
const POLYHAVEN_LAKES_2K_HDRI_ASSET: &str = "polyhaven_lakes_2k.hdr";
const LEGACY_EQUIRECT_SAMPLE_COLUMNS: u32 = 16;
const LEGACY_EQUIRECT_SAMPLE_ROWS: u32 = 8;
const LEGACY_GRID_SKY_SAMPLE_Y_MIN: u32 = 24;
const LEGACY_GRID_SKY_SAMPLE_Y_MAX: u32 = 220;
const LEGACY_GRID_OFFSET_SAMPLES: [i32; 6] = [-17, -11, -7, 7, 11, 17];
const PMREM_MIP_DIAGNOSTIC_TILE_SIZE: u32 = 96;

#[derive(Clone, Copy)]
struct MirrorMultiViewCase {
    label: &'static str,
    project_name: &'static str,
    camera_view: SinglePbrSphereCameraView,
}

fn mirror_multi_view_cases() -> [MirrorMultiViewCase; 4] {
    [
        MirrorMultiViewCase {
            label: "multi-view orthographic front perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewOrthoFront",
            camera_view: SinglePbrSphereCameraView::front(ProjectionMode::Orthographic),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective front perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveFront",
            camera_view: SinglePbrSphereCameraView::front(ProjectionMode::Perspective),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective left-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveLeftYaw",
            camera_view: SinglePbrSphereCameraView::perspective_eye([-2.25, 0.0, 3.65]),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective right-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveRightYaw",
            camera_view: SinglePbrSphereCameraView::perspective_eye([2.25, 0.0, 3.65]),
        },
    ]
}

fn mirror_multi_view_row_count() -> u32 {
    let case_count = mirror_multi_view_cases().len() as u32;
    (case_count + PBR_MIRROR_MULTI_VIEW_COLUMNS - 1) / PBR_MIRROR_MULTI_VIEW_COLUMNS
}

fn mirror_cardinal_120deg_view_cases() -> [MirrorMultiViewCase; 4] {
    [
        MirrorMultiViewCase {
            label: "120-degree perspective up-orbit perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Up",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, 120.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective down-orbit perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Down",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, -120.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective left-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Left",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(-120.0, 0.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective right-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Right",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(120.0, 0.0),
        },
    ]
}

fn mirror_cardinal_120deg_row_count() -> u32 {
    let case_count = mirror_cardinal_120deg_view_cases().len() as u32;
    (case_count + PBR_MIRROR_MULTI_VIEW_COLUMNS - 1) / PBR_MIRROR_MULTI_VIEW_COLUMNS
}

#[test]
fn runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MATRIX_HDRI_2K_OUTPUT_NAME);

    assert_shader_test_output_path(&output);
    hdri_metrics::assert_saved_real_hdri_reflection_response(&output);
}

#[test]
fn runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_matches_blur_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(
        PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME,
    );

    assert_shader_test_output_path(&output);
    hdri_metrics::assert_saved_pmrem_mip_diagnostic_blur_response(&output);
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_orientation_and_grazing_metrics() {
    for (output_name, label) in [
        (
            PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME,
            "saved orthographic perfect mirror PBR sphere",
        ),
        (
            PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME,
            "saved perspective perfect mirror PBR sphere",
        ),
    ] {
        let output = runtime_shader_pbr_real_hdri_output_path(output_name);

        assert_shader_test_output_path(&output);
        let frame = load_saved_viewport_frame(&output);
        assert_eq!(
            (frame.width, frame.height),
            (PBR_SINGLE_HDRI_OUTPUT_SIZE.x, PBR_SINGLE_HDRI_OUTPUT_SIZE.y),
            "{label} screenshot should keep the accepted mirror validation dimensions"
        );
        assert_single_sphere_reflects_environment(&frame, label);
        assert_mirror_sphere_reflection_orientation(&frame);
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_source_reference_metrics() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 4);
    source_environment.intensity = 0.65;

    for (output_name, projection_mode, label) in [
        (
            PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME,
            ProjectionMode::Orthographic,
            "saved orthographic perfect mirror PBR sphere",
        ),
        (
            PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME,
            ProjectionMode::Perspective,
            "saved perspective perfect mirror PBR sphere",
        ),
    ] {
        let output = runtime_shader_pbr_real_hdri_output_path(output_name);

        assert_shader_test_output_path(&output);
        let frame = load_saved_viewport_frame(&output);
        assert_eq!(
            (frame.width, frame.height),
            (PBR_SINGLE_HDRI_OUTPUT_SIZE.x, PBR_SINGLE_HDRI_OUTPUT_SIZE.y),
            "{label} screenshot should keep the accepted mirror validation dimensions"
        );
        assert_mirror_sphere_matches_source_reference(
            &frame,
            projection_mode,
            &source_environment,
            label,
        );
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME);
    let rows = mirror_multi_view_row_count();
    let expected_size = (
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.x * PBR_MIRROR_MULTI_VIEW_COLUMNS,
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.y * rows,
    );

    assert_shader_test_output_path(&output);
    let sheet = load_saved_viewport_frame(&output);
    assert_eq!(
        (sheet.width, sheet.height),
        expected_size,
        "multi-view mirror screenshot should keep the accepted contact-sheet dimensions"
    );

    for (index, view_case) in mirror_multi_view_cases().into_iter().enumerate() {
        let tile = viewport_frame_tile(
            &sheet,
            index as u32 % PBR_MIRROR_MULTI_VIEW_COLUMNS,
            index as u32 / PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
        );
        assert_single_sphere_reflects_environment(&tile, view_case.label);
        assert_mirror_sphere_reflection_orientation(&tile);
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_source_reference_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME);
    let rows = mirror_multi_view_row_count();
    let expected_size = (
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.x * PBR_MIRROR_MULTI_VIEW_COLUMNS,
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.y * rows,
    );
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 11);
    source_environment.intensity = 0.65;

    assert_shader_test_output_path(&output);
    let sheet = load_saved_viewport_frame(&output);
    assert_eq!(
        (sheet.width, sheet.height),
        expected_size,
        "multi-view mirror screenshot should keep the accepted contact-sheet dimensions"
    );

    for (index, view_case) in mirror_multi_view_cases().into_iter().enumerate() {
        let tile = viewport_frame_tile(
            &sheet,
            index as u32 % PBR_MIRROR_MULTI_VIEW_COLUMNS,
            index as u32 / PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
        );
        assert_mirror_sphere_matches_source_reference_with_camera_view(
            &tile,
            view_case.camera_view,
            &source_environment,
            view_case.label,
        );
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_environment_metrics() {
    let output =
        runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME);
    let rows = mirror_cardinal_120deg_row_count();
    let expected_size = (
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.x * PBR_MIRROR_MULTI_VIEW_COLUMNS,
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.y * rows,
    );

    assert_shader_test_output_path(&output);
    let sheet = load_saved_viewport_frame(&output);
    assert_eq!(
        (sheet.width, sheet.height),
        expected_size,
        "120-degree mirror screenshot should keep the accepted contact-sheet dimensions"
    );

    for (index, view_case) in mirror_cardinal_120deg_view_cases().into_iter().enumerate() {
        let tile = viewport_frame_tile(
            &sheet,
            index as u32 % PBR_MIRROR_MULTI_VIEW_COLUMNS,
            index as u32 / PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
        );
        assert_single_sphere_reflects_environment(&tile, view_case.label);
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_source_reference_metrics() {
    let output =
        runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME);
    let rows = mirror_cardinal_120deg_row_count();
    let expected_size = (
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.x * PBR_MIRROR_MULTI_VIEW_COLUMNS,
        PBR_MIRROR_MULTI_VIEW_TILE_SIZE.y * rows,
    );
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 12);
    source_environment.intensity = 0.65;

    assert_shader_test_output_path(&output);
    let sheet = load_saved_viewport_frame(&output);
    assert_eq!(
        (sheet.width, sheet.height),
        expected_size,
        "120-degree mirror screenshot should keep the accepted contact-sheet dimensions"
    );

    for (index, view_case) in mirror_cardinal_120deg_view_cases().into_iter().enumerate() {
        let tile = viewport_frame_tile(
            &sheet,
            index as u32 % PBR_MIRROR_MULTI_VIEW_COLUMNS,
            index as u32 / PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
        );
        assert_mirror_sphere_matches_source_reference_with_camera_view(
            &tile,
            view_case.camera_view,
            &source_environment,
            view_case.label,
        );
    }
}

#[test]
#[ignore = "manual screenshot export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_reflection_png_inner)
        .expect("spawn large-stack HDRI export test")
        .join()
        .expect("HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual 2K screenshot export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_2k_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_2k_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_2k_reflection_png_inner)
        .expect("spawn large-stack 2K HDRI export test")
        .join()
        .expect("2K HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual diagnostic export for source cubemap versus GGX PMREM mip blur validation"]
fn export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_mip_diagnostic".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_inner)
        .expect("spawn large-stack HDRI PMREM mip diagnostic export test")
        .join()
        .expect("HDRI PMREM mip diagnostic export test thread should not panic");
}

#[test]
#[ignore = "manual single material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_single_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_single_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_single_reflection_png_inner)
        .expect("spawn large-stack single HDRI export test")
        .join()
        .expect("single HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual mirror material sphere export for runtime PBR real HDRI reflection orientation validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_mirror_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_mirror_reflection_png_inner)
        .expect("spawn large-stack mirror HDRI export test")
        .join()
        .expect("mirror HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual multi-view mirror material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_multi_view_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_mirror_multi_view_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_mirror_multi_view_png_inner)
        .expect("spawn large-stack mirror multi-view HDRI export test")
        .join()
        .expect("mirror multi-view HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual 120-degree cardinal mirror material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_mirror_120deg_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_inner)
        .expect("spawn large-stack mirror 120-degree HDRI export test")
        .join()
        .expect("mirror 120-degree HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual real texture-map material export for runtime PBR real HDRI validation"]
fn export_runtime_shader_pbr_real_hdri_textured_material_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_textured_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_textured_material_png_inner)
        .expect("spawn large-stack textured HDRI export test")
        .join()
        .expect("textured HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual ambientCG Metal008/025/029 material export for runtime PBR real HDRI validation"]
fn export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_ambientcg_metal_batch".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png_inner)
        .expect("spawn large-stack ambientCG metal batch HDRI export test")
        .join()
        .expect("ambientCG metal batch HDRI export test thread should not panic");
}

fn export_runtime_shader_pbr_real_hdri_reflection_png_inner() {
    export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
        POLYHAVEN_LAKES_1K_HDRI_ASSET,
        PBR_MATRIX_HDRI_OUTPUT_NAME,
        1,
    );
}

fn export_runtime_shader_pbr_real_hdri_2k_reflection_png_inner() {
    export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
        POLYHAVEN_LAKES_2K_HDRI_ASSET,
        PBR_MATRIX_HDRI_2K_OUTPUT_NAME,
        2,
    );
}

fn export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_inner() {
    let environment = polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_1K_HDRI_ASSET, 1);
    let output = runtime_shader_pbr_real_hdri_output_path(
        PBR_MATRIX_HDRI_1K_PMREM_MIP_DIAGNOSTIC_OUTPUT_NAME,
    );

    save_pmrem_mip_diagnostic(&environment.mip_chain, &output);
    assert_shader_test_output_path(&output);
}

fn export_runtime_shader_pbr_real_hdri_single_reflection_png_inner() {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 2),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        "GraphicsPbrRealHdriSingleReflection",
        |paths| {
            write_single_pbr_material(
                paths
                    .assets_root()
                    .join("materials")
                    .join("single_metal_sphere.zmaterial"),
                "Single Metal Sphere",
                [0.86, 0.88, 0.9, 1.0],
                1.0,
                0.04,
                None,
                None,
                None,
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_SINGLE_HDRI_OUTPUT_NAME);

    save_viewport_frame_png(
        &frame,
        &output,
        "single real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, "single metal PBR sphere");
}

fn export_runtime_shader_pbr_real_hdri_mirror_reflection_png_inner() {
    export_runtime_shader_pbr_real_hdri_mirror_reflection_png_with_projection(
        ProjectionMode::Orthographic,
        "GraphicsPbrRealHdriMirrorOrthographicReflection",
        PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME,
        "orthographic perfect mirror PBR sphere",
    );
    export_runtime_shader_pbr_real_hdri_mirror_reflection_png_with_projection(
        ProjectionMode::Perspective,
        "GraphicsPbrRealHdriMirrorPerspectiveReflection",
        PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME,
        "perspective perfect mirror PBR sphere",
    );
}

fn export_runtime_shader_pbr_real_hdri_mirror_reflection_png_with_projection(
    projection_mode: ProjectionMode,
    project_name: &str,
    output_name: &str,
    assertion_label: &str,
) {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 4);
    // Keep the zero-roughness mirror validation readable instead of clipping the HDRI highlight
    // into a nearly solid white sphere.
    source_environment.intensity = 0.65;
    let environment = EnvironmentExtract::source_cubemap(source_environment);
    let frame = render_single_pbr_sphere_frame_with_environment_and_projection(
        environment,
        project_name,
        projection_mode,
        write_perfect_mirror_material,
    );
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);

    save_viewport_frame_png(
        &frame,
        &output,
        "mirror real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, assertion_label);
    assert_mirror_sphere_reflection_orientation(&frame);
}

fn export_runtime_shader_pbr_real_hdri_mirror_multi_view_png_inner() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 11);
    source_environment.intensity = 0.65;

    let mut frames = Vec::new();
    for view_case in mirror_multi_view_cases() {
        let frame = render_single_pbr_sphere_frame_with_environment_and_camera_view(
            EnvironmentExtract::source_cubemap(source_environment.clone()),
            view_case.project_name,
            view_case.camera_view,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
            write_perfect_mirror_material,
        );
        assert_single_sphere_reflects_environment(&frame, view_case.label);
        assert_mirror_sphere_reflection_orientation(&frame);
        frames.push(frame);
    }

    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME);
    save_viewport_frame_contact_sheet_png(
        &frames,
        PBR_MIRROR_MULTI_VIEW_COLUMNS,
        &output,
        "multi-view mirror real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
}

fn export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_inner() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 12);
    source_environment.intensity = 0.65;

    let mut frames = Vec::new();
    for view_case in mirror_cardinal_120deg_view_cases() {
        let frame = render_single_pbr_sphere_frame_with_environment_and_camera_view(
            EnvironmentExtract::source_cubemap(source_environment.clone()),
            view_case.project_name,
            view_case.camera_view,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
            write_perfect_mirror_material,
        );
        assert_single_sphere_reflects_environment(&frame, view_case.label);
        assert_mirror_sphere_matches_source_reference_with_camera_view(
            &frame,
            view_case.camera_view,
            &source_environment,
            view_case.label,
        );
        frames.push(frame);
    }

    let output =
        runtime_shader_pbr_real_hdri_output_path(PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME);
    save_viewport_frame_contact_sheet_png(
        &frames,
        PBR_MIRROR_MULTI_VIEW_COLUMNS,
        &output,
        "120-degree mirror real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
}

fn write_perfect_mirror_material(paths: &ProjectPaths) {
    write_single_pbr_material(
        paths
            .assets_root()
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
        "Perfect Mirror Sphere",
        [1.0, 1.0, 1.0, 1.0],
        1.0,
        0.0,
        None,
        None,
        None,
    );
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_inner() {
    export_runtime_shader_pbr_real_hdri_textured_material_png_with_metal009();
}

fn export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png_inner() {
    for (fixture, output_name) in [
        (AMBIENTCG_METAL008, PBR_TEXTURED_HDRI_METAL008_OUTPUT_NAME),
        (AMBIENTCG_METAL025, PBR_TEXTURED_HDRI_METAL025_OUTPUT_NAME),
        (AMBIENTCG_METAL029, PBR_TEXTURED_HDRI_METAL029_OUTPUT_NAME),
    ] {
        export_runtime_shader_pbr_real_hdri_textured_material_png_with_fixture(
            fixture,
            output_name,
        );
    }
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_with_metal009() {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 3),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        "GraphicsPbrRealHdriTexturedMaterial",
        |paths| {
            write_ambientcg_metal009_texture_assets(paths);
            write_single_pbr_material(
                paths
                    .assets_root()
                    .join("materials")
                    .join("single_metal_sphere.zmaterial"),
                "AmbientCG Metal009 Texture Maps",
                [1.0, 1.0, 1.0, 1.0],
                1.0,
                1.0,
                Some(&ambientcg_metal009_texture_uri(AMBIENTCG_METAL009_COLOR)),
                Some(&ambientcg_metal009_texture_uri(
                    AMBIENTCG_METAL009_NORMAL_GL,
                )),
                Some(&ambientcg_metal009_texture_uri(
                    AMBIENTCG_METAL009_METALLIC_ROUGHNESS,
                )),
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_TEXTURED_HDRI_OUTPUT_NAME);

    save_viewport_frame_png(
        &frame,
        &output,
        "real HDRI PBR textured material screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, "textured PBR sphere");
    assert_textured_material_has_surface_variation(&frame);
}

fn export_runtime_shader_pbr_real_hdri_textured_material_png_with_fixture(
    fixture: AmbientCgMetalFixture,
    output_name: &str,
) {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 10),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        &format!("GraphicsPbrRealHdri{}TextureMaps", fixture.id),
        |paths| {
            write_ambientcg_metal_texture_assets(paths, fixture);
            write_single_pbr_material(
                paths
                    .assets_root()
                    .join("materials")
                    .join("single_metal_sphere.zmaterial"),
                &format!("AmbientCG {} Texture Maps", fixture.id),
                [1.0, 1.0, 1.0, 1.0],
                1.0,
                1.0,
                Some(&ambientcg_metal_texture_uri(fixture, fixture.color)),
                Some(&ambientcg_metal_texture_uri(fixture, fixture.normal_gl)),
                Some(&ambientcg_metal_texture_uri(
                    fixture,
                    fixture.metallic_roughness,
                )),
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);

    save_viewport_frame_png(
        &frame,
        &output,
        "real HDRI PBR ambientCG textured material screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, fixture.id);
    assert_textured_material_has_surface_variation(&frame);
}

fn export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
    asset_name: &str,
    output_name: &str,
    source_revision: u64,
) {
    let frame = render_pbr_matrix_frame_with_environment(EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(asset_name, source_revision),
    ));
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);

    save_viewport_frame_png(&frame, &output, "real HDRI PBR reflection screenshot");
    assert_shader_test_output_path(&output);
    hdri_metrics::assert_real_hdri_reflection_response(&frame);
}

fn render_pbr_matrix_frame_with_environment(
    environment: EnvironmentExtract,
) -> zircon_runtime::graphics::ViewportFrame {
    render_project_frame_with_environment(
        "graphics_pbr_real_hdri_integration",
        "GraphicsPbrRealHdriIntegration",
        "res://scenes/pbr_matrix.scene.toml",
        PBR_MATRIX_OUTPUT_SIZE,
        environment,
        |paths| {
            write_uv_sphere_model(
                paths
                    .assets_root()
                    .join("models")
                    .join("pbr_matrix_sphere.model.toml"),
                "res://models/pbr_matrix_sphere.model.toml",
                24,
                48,
            );
            for row in 0..PBR_MATRIX_DIMENSION {
                for column in 0..PBR_MATRIX_DIMENSION {
                    write_pbr_matrix_material(
                        paths
                            .assets_root()
                            .join("materials")
                            .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                        pbr_matrix_axis_value(column),
                        pbr_matrix_axis_value(row),
                    );
                }
            }
            write_pbr_matrix_scene(
                paths
                    .assets_root()
                    .join("scenes")
                    .join("pbr_matrix.scene.toml"),
            );
        },
    )
}

fn render_single_pbr_sphere_frame_with_environment(
    environment: EnvironmentExtract,
    project_name: &str,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> zircon_runtime::graphics::ViewportFrame {
    render_single_pbr_sphere_frame_with_environment_and_projection(
        environment,
        project_name,
        ProjectionMode::Orthographic,
        write_material_assets,
    )
}

fn render_single_pbr_sphere_frame_with_environment_and_projection(
    environment: EnvironmentExtract,
    project_name: &str,
    projection_mode: ProjectionMode,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> zircon_runtime::graphics::ViewportFrame {
    render_single_pbr_sphere_frame_with_environment_and_camera_view(
        environment,
        project_name,
        SinglePbrSphereCameraView::front(projection_mode),
        PBR_SINGLE_HDRI_OUTPUT_SIZE,
        write_material_assets,
    )
}

fn render_single_pbr_sphere_frame_with_environment_and_camera_view(
    environment: EnvironmentExtract,
    project_name: &str,
    camera_view: SinglePbrSphereCameraView,
    output_size: UVec2,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> zircon_runtime::graphics::ViewportFrame {
    render_project_frame_with_environment(
        "graphics_pbr_single_real_hdri_integration",
        project_name,
        "res://scenes/single_pbr_sphere.scene.toml",
        output_size,
        environment,
        |paths| {
            write_uv_sphere_model(
                paths
                    .assets_root()
                    .join("models")
                    .join("single_pbr_sphere.model.toml"),
                "res://models/single_pbr_sphere.model.toml",
                PBR_SINGLE_SPHERE_RINGS,
                PBR_SINGLE_SPHERE_SEGMENTS,
            );
            write_material_assets(paths);
            write_single_pbr_sphere_scene_with_camera_view(
                paths
                    .assets_root()
                    .join("scenes")
                    .join("single_pbr_sphere.scene.toml"),
                camera_view,
            );
        },
    )
}

fn render_project_frame_with_environment(
    temp_label: &str,
    project_name: &str,
    scene_uri_text: &str,
    output_size: UVec2,
    environment: EnvironmentExtract,
    write_project_assets: impl FnOnce(&ProjectPaths),
) -> zircon_runtime::graphics::ViewportFrame {
    let root = unique_temp_project_root(temp_label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    let scene_uri = AssetUri::parse(scene_uri_text).unwrap();
    ProjectManifest::new(project_name, scene_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    write_project_assets(&paths);

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world =
        zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(output_size),
        virtual_geometry_debug: None,
    });
    snapshot.environment = environment;
    snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
    snapshot.overlays = RenderOverlayExtract::default();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, output_size).unwrap();
    let _ = fs::remove_dir_all(root);
    frame
}

fn save_viewport_frame_png(
    frame: &zircon_runtime::graphics::ViewportFrame,
    output: &Path,
    context: &str,
) {
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered real HDRI PBR frame should match output image dimensions")
        .save_with_format(output, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("write {context}: {error}"));
}

fn save_viewport_frame_contact_sheet_png(
    frames: &[ViewportFrame],
    columns: u32,
    output: &Path,
    context: &str,
) {
    assert!(
        !frames.is_empty(),
        "{context} should contain at least one frame"
    );
    assert!(columns > 0, "{context} should use at least one column");

    let tile_width = frames[0].width;
    let tile_height = frames[0].height;
    let rows = (frames.len() as u32 + columns - 1) / columns;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(tile_width * columns, tile_height * rows);

    for (index, frame) in frames.iter().enumerate() {
        assert_eq!(
            (frame.width, frame.height),
            (tile_width, tile_height),
            "{context} frames should share one tile size"
        );
        let tile_column = index as u32 % columns;
        let tile_row = index as u32 / columns;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let source_index = ((y * tile_width + x) * 4) as usize;
                image.put_pixel(
                    tile_column * tile_width + x,
                    tile_row * tile_height + y,
                    Rgba([
                        frame.rgba[source_index],
                        frame.rgba[source_index + 1],
                        frame.rgba[source_index + 2],
                        frame.rgba[source_index + 3],
                    ]),
                );
            }
        }
    }

    image
        .save_with_format(output, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("write {context}: {error}"));
}

fn load_saved_viewport_frame(path: &Path) -> ViewportFrame {
    let image = image::open(path)
        .unwrap_or_else(|error| panic!("read saved runtime shader screenshot {path:?}: {error}"))
        .to_rgba8();
    let width = image.width();
    let height = image.height();

    ViewportFrame {
        width,
        height,
        rgba: image.into_raw(),
        generation: 0,
        capture_report: Default::default(),
    }
}

fn viewport_frame_tile(
    sheet: &ViewportFrame,
    column: u32,
    row: u32,
    tile_size: UVec2,
) -> ViewportFrame {
    let x0 = column * tile_size.x;
    let y0 = row * tile_size.y;
    assert!(
        x0 + tile_size.x <= sheet.width && y0 + tile_size.y <= sheet.height,
        "requested multi-view tile should fit inside saved contact sheet"
    );

    let mut rgba = vec![0_u8; (tile_size.x * tile_size.y * 4) as usize];
    for y in 0..tile_size.y {
        let source_start = (((y0 + y) * sheet.width + x0) * 4) as usize;
        let source_end = source_start + (tile_size.x * 4) as usize;
        let target_start = (y * tile_size.x * 4) as usize;
        let target_end = target_start + (tile_size.x * 4) as usize;
        rgba[target_start..target_end].copy_from_slice(&sheet.rgba[source_start..source_end]);
    }

    ViewportFrame {
        width: tile_size.x,
        height: tile_size.y,
        rgba,
        generation: 0,
        capture_report: Default::default(),
    }
}

fn polyhaven_lakes_source_cubemap_environment(
    asset_name: &str,
    source_revision: u64,
) -> SourceCubemapEnvironment {
    let path = shader_test_asset_dir().join(asset_name);
    let bytes = fs::read(&path).expect("read Poly Haven lakes HDRI");
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)
        .expect("decode Poly Haven lakes HDRI")
        .to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let face_size =
        zircon_runtime::core::framework::render::source_cubemap_face_size_from_equirect_height(
            image.height(),
        );
    let mip_chain = zircon_runtime::core::framework::render::build_source_cubemap_from_equirect(
        face_size,
        |u, v| expose_hdr_sample(sample_hdri_bilinear(&image, u, v), exposure),
    );
    let irradiance_cube = build_source_cubemap_irradiance_cube(&mip_chain);

    let mut environment =
        SourceCubemapEnvironment::new(mip_chain, source_revision, source_hash_words(&bytes))
            .with_irradiance_cube(irradiance_cube);
    environment.intensity = 1.45;
    environment.rotation_radians = 0.0;
    environment
}

fn save_pmrem_mip_diagnostic(mip_chain: &SourceCubemapMipChain, output: &Path) {
    let mip_count = mip_chain.mip_count();
    let face_count = CubemapFace::ALL.len() as u32;
    let width = PMREM_MIP_DIAGNOSTIC_TILE_SIZE * mip_count;
    let height = PMREM_MIP_DIAGNOSTIC_TILE_SIZE * face_count * 2;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for (face_index, face) in CubemapFace::ALL.iter().copied().enumerate() {
        for mip in 0..mip_count {
            paint_mip_diagnostic_tile(
                &mut image,
                mip_chain,
                mip_chain.source_texels(),
                face,
                mip,
                mip * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
                face_index as u32 * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
            );
            paint_mip_diagnostic_tile(
                &mut image,
                mip_chain,
                mip_chain.texels(),
                face,
                mip,
                mip * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
                (face_count + face_index as u32) * PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
            );
        }
    }

    image
        .save_with_format(output, ImageFormat::Png)
        .expect("write PMREM mip diagnostic screenshot");
}

fn paint_mip_diagnostic_tile(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    mip_chain: &SourceCubemapMipChain,
    texels: &[[f32; 4]],
    face: CubemapFace,
    mip: u32,
    origin_x: u32,
    origin_y: u32,
) {
    for y in 0..PMREM_MIP_DIAGNOSTIC_TILE_SIZE {
        for x in 0..PMREM_MIP_DIAGNOSTIC_TILE_SIZE {
            let u = (x as f32 + 0.5) / PMREM_MIP_DIAGNOSTIC_TILE_SIZE as f32;
            let v = (y as f32 + 0.5) / PMREM_MIP_DIAGNOSTIC_TILE_SIZE as f32;
            image.put_pixel(
                origin_x + x,
                origin_y + y,
                Rgba(linear_hdr_to_srgb8(sample_face_mip_bilinear(
                    texels,
                    mip_chain.face_size(),
                    mip_chain.mip_count(),
                    face,
                    mip,
                    u,
                    v,
                ))),
            );
        }
    }
}

fn sample_face_mip_bilinear(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip: u32,
    u: f32,
    v: f32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let x = u.clamp(0.0, 1.0) * mip_size.saturating_sub(1) as f32;
    let y = v.clamp(0.0, 1.0) * mip_size.saturating_sub(1) as f32;
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(mip_size.saturating_sub(1));
    let y1 = (y0 + 1).min(mip_size.saturating_sub(1));
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let c00 = face_mip_texel(texels, face_size, mip_count, face, mip, x0, y0);
    let c10 = face_mip_texel(texels, face_size, mip_count, face, mip, x1, y0);
    let c01 = face_mip_texel(texels, face_size, mip_count, face, mip, x0, y1);
    let c11 = face_mip_texel(texels, face_size, mip_count, face, mip, x1, y1);

    lerp4(lerp4(c00, c10, tx), lerp4(c01, c11, tx), ty)
}

fn face_mip_texel(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip: u32,
    x: u32,
    y: u32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
    texels[offset + y as usize * mip_size as usize + x as usize]
}

fn linear_hdr_to_srgb8(rgba: [f32; 4]) -> [u8; 4] {
    [
        linear_channel_to_srgb8(rgba[0]),
        linear_channel_to_srgb8(rgba[1]),
        linear_channel_to_srgb8(rgba[2]),
        255,
    ]
}

fn linear_channel_to_srgb8(channel: f32) -> u8 {
    let mapped = channel.max(0.0) / (1.0 + channel.max(0.0));
    let srgb = if mapped <= 0.003_130_8 {
        mapped * 12.92
    } else {
        1.055 * mapped.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn sample_hdri_bilinear(image: &image::Rgb32FImage, u: f32, v: f32) -> [f32; 3] {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let texel_x = u.fract() * width as f32 - 0.5;
    let texel_y = v.clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = texel_x.floor() as i32;
    let y0 = texel_y.floor() as i32;
    let tx = texel_x - texel_x.floor();
    let ty = texel_y - texel_y.floor();
    let x0u = ((x0 % width as i32 + width as i32) % width as i32) as u32;
    let x1u = (x0u + 1) % width;
    let y0u = (y0.clamp(0, height.saturating_sub(1) as i32)) as u32;
    let y1u = (y0u + 1).min(height - 1);
    let c00 = image.get_pixel(x0u, y0u).0;
    let c10 = image.get_pixel(x1u, y0u).0;
    let c01 = image.get_pixel(x0u, y1u).0;
    let c11 = image.get_pixel(x1u, y1u).0;
    [
        lerp(lerp(c00[0], c10[0], tx), lerp(c01[0], c11[0], tx), ty),
        lerp(lerp(c00[1], c10[1], tx), lerp(c01[1], c11[1], tx), ty),
        lerp(lerp(c00[2], c10[2], tx), lerp(c01[2], c11[2], tx), ty),
    ]
}

fn sampled_hdri_exposure(image: &image::Rgb32FImage) -> f32 {
    let step_x = (image.width() / 128).max(1);
    let step_y = (image.height() / 64).max(1);
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    let mut y = 0;
    while y < image.height() {
        let mut x = 0;
        while x < image.width() {
            sum += luma(image.get_pixel(x, y).0);
            count += 1.0;
            x += step_x;
        }
        y += step_y;
    }
    (0.45 / (sum / count.max(1.0)).max(0.0001)).clamp(0.02, 4.0)
}

fn expose_hdr_sample(rgb: [f32; 3], exposure: f32) -> [f32; 4] {
    let exposed = rgb.map(|channel| (channel.max(0.0) * exposure).min(65_504.0));
    [exposed[0], exposed[1], exposed[2], 1.0]
}

fn source_hash_words(bytes: &[u8]) -> [u32; 4] {
    let mut state = [0x811c9dc5_u32, 0x9e3779b9, 0x85ebca6b, 0xc2b2ae35];
    for (index, byte) in bytes.iter().enumerate() {
        let slot = index & 3;
        state[slot] ^= u32::from(*byte);
        state[slot] = state[slot].wrapping_mul(16_777_619);
    }
    state
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn pbr_matrix_axis_value(index: usize) -> f32 {
    index as f32 / (PBR_MATRIX_DIMENSION - 1) as f32
}

fn pbr_matrix_world_x(column: usize) -> f32 {
    (column as f32 - (PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5) * PBR_MATRIX_STEP_X
}

fn pbr_matrix_world_y(row: usize) -> f32 {
    ((PBR_MATRIX_DIMENSION as f32 - 1.0) * 0.5 - row as f32) * PBR_MATRIX_STEP_Y
}

fn shader_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader");
    fs::create_dir_all(&output_dir).unwrap();
    output_dir
}

fn runtime_shader_pbr_real_hdri_output_path(output_name: &str) -> PathBuf {
    shader_test_output_dir().join(output_name)
}

fn assert_shader_test_output_path(path: &Path) {
    let output_dir = shader_test_output_dir();
    assert!(
        path.starts_with(&output_dir),
        "shader validation image should be written under docs/tests/runtime/shader, path={path:?}, expected_dir={output_dir:?}"
    );
}

fn shader_test_asset_dir() -> PathBuf {
    let asset_dir = shader_test_output_dir().join("assets");
    fs::create_dir_all(&asset_dir).unwrap();
    asset_dir
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let process_id = std::process::id();
    let sequence = NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_graphics_{label}_{process_id}_{sequence}_{unique}"
    ))
}

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}
