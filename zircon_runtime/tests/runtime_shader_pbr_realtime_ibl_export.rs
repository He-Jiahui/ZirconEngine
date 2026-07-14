use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, ProjectionMode, RenderOverlayExtract,
    SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::graphics::{RealtimeIblGpuTimingReport, SceneRenderer, ViewportFrame};

#[path = "runtime_shader_pbr_hdri_export/scene_fixtures.rs"]
mod scene_fixtures;
mod support;

use scene_fixtures::{
    write_pbr_matrix_material, write_pbr_matrix_scene, write_single_pbr_material,
    write_single_pbr_sphere_scene_with_camera_view, write_uv_sphere_model,
    SinglePbrSphereCameraView,
};

const PBR_MATRIX_DIMENSION: usize = 8;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1600, 1200);
const PBR_MATRIX_ORTHO_SIZE: f32 = 5.8;
const PBR_MATRIX_STEP_X: f32 = 0.7;
const PBR_MATRIX_STEP_Y: f32 = 0.62;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.21;
const REALTIME_UPDATE_SLICE_COUNT: usize = 16;
const RENDERDOC_CAPTURE_FINAL_SH9_SLICE_ENV: &str = "ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9";
const OUTPUT_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_sh9_8x8_reflection_20260714.png";
const TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_sh9_8x8_timing_20260714.txt";
const GPU_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_sh9_8x8_gpu_timing_20260714.txt";
const MULTI_VIEW_OUTPUT_SIZE: UVec2 = UVec2::new(800, 600);
const MULTI_VIEW_COLUMNS: u32 = 5;
const MULTI_VIEW_CONTACT_SHEET_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_mirror_cardinal_120deg_contact_sheet_20260714.png";

#[derive(Clone, Copy)]
struct RealtimeMultiViewCase {
    label: &'static str,
    output_name: &'static str,
    camera_view: SinglePbrSphereCameraView,
}

fn realtime_multiview_cases() -> [RealtimeMultiViewCase; 5] {
    [
        RealtimeMultiViewCase {
            label: "front",
            output_name: "runtime_shader_pbr_procedural_realtime_ibl_mirror_front_20260714.png",
            camera_view: SinglePbrSphereCameraView::front(ProjectionMode::Perspective),
        },
        RealtimeMultiViewCase {
            label: "pitch plus 120 degrees",
            output_name:
                "runtime_shader_pbr_procedural_realtime_ibl_mirror_pitch_plus_120_20260714.png",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, 120.0),
        },
        RealtimeMultiViewCase {
            label: "pitch minus 120 degrees",
            output_name:
                "runtime_shader_pbr_procedural_realtime_ibl_mirror_pitch_minus_120_20260714.png",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, -120.0),
        },
        RealtimeMultiViewCase {
            label: "yaw minus 120 degrees",
            output_name:
                "runtime_shader_pbr_procedural_realtime_ibl_mirror_yaw_minus_120_20260714.png",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(-120.0, 0.0),
        },
        RealtimeMultiViewCase {
            label: "yaw plus 120 degrees",
            output_name:
                "runtime_shader_pbr_procedural_realtime_ibl_mirror_yaw_plus_120_20260714.png",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(120.0, 0.0),
        },
    ]
}

#[test]
fn realtime_ibl_export_contract_uses_requested_matrix_and_unreal_slice_count() {
    assert_eq!(PBR_MATRIX_DIMENSION * PBR_MATRIX_DIMENSION, 64);
    assert_eq!(pbr_matrix_axis_value(0), 0.0);
    assert_eq!(pbr_matrix_axis_value(PBR_MATRIX_DIMENSION - 1), 1.0);
    assert_eq!(REALTIME_UPDATE_SLICE_COUNT, 16);
    assert_eq!(realtime_multiview_cases().len(), 5);
}

#[test]
#[ignore = "manual WGPU product acceptance for Shader 06 procedural realtime IBL"]
fn export_procedural_realtime_ibl_pbr_matrix_png() {
    let root = unique_temp_project_root("shader_pbr_realtime_ibl");
    prepare_matrix_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
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
        viewport_size: Some(PBR_MATRIX_OUTPUT_SIZE),
        virtual_geometry_debug: None,
    });
    snapshot.environment = EnvironmentExtract::procedural_default();
    snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
    snapshot.overlays = RenderOverlayExtract::default();

    let asset_runtime = support::ProjectAssetTestRuntime::new(asset_manager);
    let mut renderer = SceneRenderer::new(asset_runtime.access()).unwrap();
    let initial_started = Instant::now();
    renderer
        .render(snapshot.clone(), PBR_MATRIX_OUTPUT_SIZE)
        .expect("render initial full realtime IBL publication");
    let initial_elapsed = initial_started.elapsed();

    let mut updated_snapshot = snapshot;
    let sky = &mut updated_snapshot.environment.skybox.procedural;
    sky.horizon_color = Vec4::new(0.72, 0.24, 0.08, 1.0);
    sky.zenith_color = Vec4::new(0.08, 0.32, 0.82, 1.0);
    sky.ground_color = Vec4::new(0.025, 0.04, 0.075, 1.0);
    sky.source_revision = sky.source_revision.wrapping_add(1);
    updated_snapshot.preview = PreviewEnvironmentExtract::from_environment(
        &updated_snapshot.environment,
        true,
        Vec4::ZERO,
    );

    let mut slice_millis = Vec::with_capacity(REALTIME_UPDATE_SLICE_COUNT);
    let capture_final_sh9_slice =
        std::env::var(RENDERDOC_CAPTURE_FINAL_SH9_SLICE_ENV).is_ok_and(|value| value == "1");
    for slice_index in 0..REALTIME_UPDATE_SLICE_COUNT {
        let capture_this_slice =
            capture_final_sh9_slice && slice_index + 1 == REALTIME_UPDATE_SLICE_COUNT;
        if capture_this_slice {
            renderer.start_graphics_debugger_capture();
        }
        let started = Instant::now();
        let render_result = renderer.render(updated_snapshot.clone(), PBR_MATRIX_OUTPUT_SIZE);
        if capture_this_slice {
            renderer
                .stop_graphics_debugger_capture()
                .expect("stop RenderDoc capture after the final SH9 update slice");
        }
        render_result.expect("render realtime IBL update slice");
        slice_millis.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let final_frame = renderer
        .render(updated_snapshot, PBR_MATRIX_OUTPUT_SIZE)
        .expect("render the newly published realtime IBL ready slot");
    assert!(
        renderer.realtime_ibl_gpu_timing_supported(),
        "product adapter must expose encoder timestamp queries for EC-M4 acceptance"
    );
    let gpu_timings = renderer.take_realtime_ibl_gpu_timing_reports();
    assert_realtime_gpu_timings(&gpu_timings);

    let output = shader_test_output_dir().join(OUTPUT_NAME);
    save_viewport_frame_png(&final_frame, &output);
    assert_realtime_matrix_image(&final_frame);
    let timing_output = shader_test_output_dir().join(TIMING_REPORT_NAME);
    fs::write(
        &timing_output,
        timing_report(initial_elapsed.as_secs_f64() * 1000.0, &slice_millis),
    )
    .expect("write realtime IBL frame timing report");
    let gpu_timing_output = shader_test_output_dir().join(GPU_TIMING_REPORT_NAME);
    fs::write(&gpu_timing_output, gpu_timing_report(&gpu_timings))
        .expect("write realtime IBL GPU timing report");
    assert!(output.starts_with(shader_test_output_dir()));
    assert!(timing_output.starts_with(shader_test_output_dir()));
    assert!(gpu_timing_output.starts_with(shader_test_output_dir()));
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "manual WGPU product acceptance for Shader 06 procedural realtime IBL multiview"]
fn export_procedural_realtime_ibl_mirror_cardinal_120deg_png() {
    let mut frames = Vec::new();
    for view_case in realtime_multiview_cases() {
        let frame = render_realtime_mirror_view(view_case);
        assert_realtime_mirror_view(&frame, view_case.label);
        let output = shader_test_output_dir().join(view_case.output_name);
        save_viewport_frame_png(&frame, &output);
        assert!(output.starts_with(shader_test_output_dir()));
        frames.push(frame);
    }

    for frame in frames.iter().skip(1) {
        assert!(
            mean_absolute_rgb_difference(&frames[0], frame) > 1.0,
            "each exact 120-degree camera orbit should visibly change the environment reflection"
        );
    }
    let yaw_difference = mean_absolute_rgb_difference(&frames[3], &frames[4]);
    let yaw_minus_bright = brightest_pixel_position(&frames[3]);
    let yaw_plus_bright = brightest_pixel_position(&frames[4]);
    assert!(
        yaw_difference > 0.5
            && yaw_minus_bright.0.abs_diff(yaw_plus_bright.0) > MULTI_VIEW_OUTPUT_SIZE.x / 4,
        "directional sun should move horizontally between left/right 120-degree reflections: difference={yaw_difference:.3}, minus={yaw_minus_bright:?}, plus={yaw_plus_bright:?}"
    );
    for (index, frame) in frames.iter().enumerate() {
        let bright = brightest_pixel_position(frame);
        assert!(
            bright.2 >= 735,
            "{} should retain the directional sun highlight: brightest={}",
            realtime_multiview_cases()[index].label,
            bright.2
        );
    }
    let bright_positions = frames
        .iter()
        .map(brightest_pixel_position)
        .collect::<Vec<_>>();
    let min_bright_x = bright_positions
        .iter()
        .map(|position| position.0)
        .min()
        .unwrap();
    let max_bright_x = bright_positions
        .iter()
        .map(|position| position.0)
        .max()
        .unwrap();
    let min_bright_y = bright_positions
        .iter()
        .map(|position| position.1)
        .min()
        .unwrap();
    let max_bright_y = bright_positions
        .iter()
        .map(|position| position.1)
        .max()
        .unwrap();
    const VIEWPORT_EDGE_TOLERANCE_PX: u32 = 1;
    assert!(
        min_bright_x <= VIEWPORT_EDGE_TOLERANCE_PX
            || max_bright_x >= MULTI_VIEW_OUTPUT_SIZE.x - 1 - VIEWPORT_EDGE_TOLERANCE_PX
            || min_bright_y <= VIEWPORT_EDGE_TOLERANCE_PX
            || max_bright_y >= MULTI_VIEW_OUTPUT_SIZE.y - 1 - VIEWPORT_EDGE_TOLERANCE_PX,
        "a 120-degree camera orbit should move the directional sun to a viewport boundary: {bright_positions:?}"
    );

    let contact_sheet = shader_test_output_dir().join(MULTI_VIEW_CONTACT_SHEET_NAME);
    save_viewport_frame_contact_sheet_png(&frames, MULTI_VIEW_COLUMNS, &contact_sheet);
    assert!(contact_sheet.starts_with(shader_test_output_dir()));
}

fn render_realtime_mirror_view(view_case: RealtimeMultiViewCase) -> ViewportFrame {
    let root = unique_temp_project_root("shader_pbr_realtime_ibl_multiview");
    let paths = prepare_single_mirror_project(&root, view_case.camera_view);
    let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml").unwrap();
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
        viewport_size: Some(MULTI_VIEW_OUTPUT_SIZE),
        virtual_geometry_debug: None,
    });
    snapshot.environment = directional_procedural_environment();
    snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
    snapshot.overlays = RenderOverlayExtract::default();

    let asset_runtime = support::ProjectAssetTestRuntime::new(asset_manager);
    let mut renderer = SceneRenderer::new(asset_runtime.access()).unwrap();
    let frame = renderer
        .render(snapshot, MULTI_VIEW_OUTPUT_SIZE)
        .unwrap_or_else(|error| {
            panic!(
                "render {} realtime IBL mirror view: {error}",
                view_case.label
            )
        });
    let _ = fs::remove_dir_all(root);
    drop(paths);
    frame
}

fn prepare_single_mirror_project(
    root: &Path,
    camera_view: SinglePbrSphereCameraView,
) -> ProjectPaths {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml").unwrap();
    ProjectManifest::new("GraphicsPbrRealtimeIblMultiview", scene_uri, 1)
        .save(paths.manifest_path())
        .unwrap();
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    write_uv_sphere_model(
        asset_root
            .join("models")
            .join("single_pbr_sphere.model.toml"),
        "res://models/single_pbr_sphere.model.toml",
        48,
        96,
    );
    write_single_pbr_material(
        asset_root
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
        "Realtime IBL Mirror Sphere",
        [1.0, 1.0, 1.0, 1.0],
        1.0,
        0.0,
        None,
        None,
        None,
    );
    write_single_pbr_sphere_scene_with_camera_view(
        asset_root
            .join("scenes")
            .join("single_pbr_sphere.scene.toml"),
        camera_view,
    );
    paths
}

fn directional_procedural_environment() -> EnvironmentExtract {
    let mut environment = EnvironmentExtract::procedural_default();
    let sky = &mut environment.skybox.procedural;
    sky.horizon_color = Vec4::new(0.72, 0.24, 0.08, 1.0);
    sky.zenith_color = Vec4::new(0.08, 0.32, 0.82, 1.0);
    sky.ground_color = Vec4::new(0.025, 0.04, 0.075, 1.0);
    sky.sun_direction = Vec4::new(0.52, 0.42, 0.74, 0.0);
    sky.sun_color = Vec4::new(1.0, 0.72, 0.34, 1.0);
    sky.sun_intensity = 4.0;
    sky.sun_angular_radius_radians = 0.045;
    sky.source_revision = sky.source_revision.wrapping_add(2);
    environment
}

fn prepare_matrix_project(root: &Path) -> ProjectPaths {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/pbr_matrix.scene.toml").unwrap();
    ProjectManifest::new("GraphicsPbrRealtimeIbl", scene_uri, 1)
        .save(paths.manifest_path())
        .unwrap();
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    write_uv_sphere_model(
        asset_root
            .join("models")
            .join("pbr_matrix_sphere.model.toml"),
        "res://models/pbr_matrix_sphere.model.toml",
        32,
        64,
    );
    for row in 0..PBR_MATRIX_DIMENSION {
        for column in 0..PBR_MATRIX_DIMENSION {
            write_pbr_matrix_material(
                asset_root
                    .join("materials")
                    .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                pbr_matrix_axis_value(column),
                pbr_matrix_axis_value(row),
            );
        }
    }
    write_pbr_matrix_scene(asset_root.join("scenes").join("pbr_matrix.scene.toml"));
    paths
}

fn assert_realtime_matrix_image(frame: &ViewportFrame) {
    assert_eq!((frame.width, frame.height), (1600, 1200));
    let unique_colors = frame
        .rgba
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect::<HashSet<_>>()
        .len();
    assert!(
        unique_colors > 500,
        "realtime IBL matrix should not be blank or visibly quantized: {unique_colors} colors"
    );

    let upper = average_luma_band(frame, 0, frame.height / 3);
    let lower = average_luma_band(frame, frame.height * 2 / 3, frame.height);
    assert!(
        (upper - lower).abs() > 8.0,
        "captured procedural sky should remain visible in reflections: upper={upper:.2}, lower={lower:.2}"
    );
}

fn assert_realtime_mirror_view(frame: &ViewportFrame, label: &str) {
    assert_eq!(
        (frame.width, frame.height),
        (MULTI_VIEW_OUTPUT_SIZE.x, MULTI_VIEW_OUTPUT_SIZE.y),
        "{label} should keep the requested product dimensions"
    );
    let unique_colors = frame
        .rgba
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect::<HashSet<_>>()
        .len();
    assert!(
        unique_colors > 500,
        "{label} should contain a continuous sky and mirror reflection: {unique_colors} colors"
    );

    let center_difference = mean_absolute_rgb_difference_rects(
        frame,
        frame.width / 3,
        frame.height / 3,
        frame.width * 2 / 3,
        frame.height * 2 / 3,
        0,
        0,
    );
    assert!(
        center_difference > 1.0,
        "{label} mirror sphere should remain distinguishable from the sky: center_rgb_difference={center_difference:.2}"
    );
}

fn mean_absolute_rgb_difference_rects(
    frame: &ViewportFrame,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
    reference_x: u32,
    reference_y: u32,
) -> f64 {
    let reference = ((reference_y * frame.width + reference_x) * 4) as usize;
    let reference = &frame.rgba[reference..reference + 3];
    let mut difference = 0_u64;
    let mut count = 0_u64;
    for y in start_y..end_y {
        for x in start_x..end_x {
            let index = ((y * frame.width + x) * 4) as usize;
            for channel in 0..3 {
                difference += frame.rgba[index + channel].abs_diff(reference[channel]) as u64;
                count += 1;
            }
        }
    }
    difference as f64 / count.max(1) as f64
}

fn mean_absolute_rgb_difference(first: &ViewportFrame, second: &ViewportFrame) -> f64 {
    assert_eq!((first.width, first.height), (second.width, second.height));
    let mut difference = 0_u64;
    let mut samples = 0_u64;
    for (first, second) in first.rgba.chunks_exact(4).zip(second.rgba.chunks_exact(4)) {
        for channel in 0..3 {
            difference += first[channel].abs_diff(second[channel]) as u64;
            samples += 1;
        }
    }
    difference as f64 / samples.max(1) as f64
}

fn brightest_pixel_position(frame: &ViewportFrame) -> (u32, u32, u16) {
    let mut brightest = (0, 0, 0_u16);
    for y in 0..frame.height {
        for x in 0..frame.width {
            let index = ((y * frame.width + x) * 4) as usize;
            let value = frame.rgba[index] as u16
                + frame.rgba[index + 1] as u16
                + frame.rgba[index + 2] as u16;
            if value > brightest.2 {
                brightest = (x, y, value);
            }
        }
    }
    brightest
}

fn average_luma_band(frame: &ViewportFrame, start_y: u32, end_y: u32) -> f64 {
    let mut sum = 0.0;
    let mut count = 0_u64;
    for y in start_y..end_y {
        for x in 0..frame.width {
            let index = ((y * frame.width + x) * 4) as usize;
            sum += 0.2126 * frame.rgba[index] as f64
                + 0.7152 * frame.rgba[index + 1] as f64
                + 0.0722 * frame.rgba[index + 2] as f64;
            count += 1;
        }
    }
    sum / count.max(1) as f64
}

fn timing_report(initial_millis: f64, slice_millis: &[f64]) -> String {
    let average = slice_millis.iter().sum::<f64>() / slice_millis.len().max(1) as f64;
    let maximum = slice_millis.iter().copied().fold(0.0_f64, f64::max);
    let slices = slice_millis
        .iter()
        .enumerate()
        .map(|(index, millis)| format!("slice_{index:02}_cpu_ms={millis:.3}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "initial_full_update_cpu_ms={initial_millis:.3}\nupdate_slice_count={}\nupdate_slice_average_cpu_ms={average:.3}\nupdate_slice_max_cpu_ms={maximum:.3}\n{slices}\n",
        slice_millis.len()
    )
}

fn assert_realtime_gpu_timings(reports: &[RealtimeIblGpuTimingReport]) {
    assert_eq!(
        reports.len(),
        REALTIME_UPDATE_SLICE_COUNT + 1,
        "initial publication plus every sliced update must produce a GPU timestamp"
    );
    assert!(reports[0].full_update);
    assert!(
        reports
            .iter()
            .all(|report| report.elapsed_gpu_nanoseconds > 0.0),
        "every realtime IBL batch must consume measurable GPU time: {reports:?}"
    );
    assert!(
        reports
            .iter()
            .any(|report| report.operation_label == "diffuse_sh9"),
        "the final GPU SH9 slice must be timestamped"
    );
    assert!(
        reports.iter().skip(1).all(|report| !report.full_update),
        "only the initial publication may use the full-update path"
    );
    let initial_nanoseconds = reports[0].elapsed_gpu_nanoseconds;
    let sliced_maximum_nanoseconds = reports
        .iter()
        .skip(1)
        .map(|report| report.elapsed_gpu_nanoseconds)
        .fold(0.0_f64, f64::max);
    assert!(
        sliced_maximum_nanoseconds < initial_nanoseconds * 0.75,
        "time slicing must keep the heaviest update below 75% of the full publication: full={initial_nanoseconds}ns sliced_max={sliced_maximum_nanoseconds}ns"
    );
}

fn gpu_timing_report(reports: &[RealtimeIblGpuTimingReport]) -> String {
    let initial_millis = reports
        .first()
        .map(|report| report.elapsed_gpu_nanoseconds / 1_000_000.0)
        .unwrap_or_default();
    let sliced = reports.iter().skip(1).collect::<Vec<_>>();
    let sliced_average_millis = sliced
        .iter()
        .map(|report| report.elapsed_gpu_nanoseconds / 1_000_000.0)
        .sum::<f64>()
        / sliced.len().max(1) as f64;
    let sliced_maximum_millis = sliced
        .iter()
        .map(|report| report.elapsed_gpu_nanoseconds / 1_000_000.0)
        .fold(0.0_f64, f64::max);
    let samples = reports
        .iter()
        .map(|report| {
            format!(
                "frame_{:02}_gpu_ms={:.6} state={} full_update={} passes={} dispatches={} operations={}",
                report.frame_number,
                report.elapsed_gpu_nanoseconds / 1_000_000.0,
                report.logical_state,
                report.full_update,
                report.pass_count,
                report.dispatch_count,
                report.operation_label,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "timestamp_query_supported=true\nsample_count={}\ninitial_full_update_gpu_ms={initial_millis:.6}\nupdate_slice_average_gpu_ms={sliced_average_millis:.6}\nupdate_slice_max_gpu_ms={sliced_maximum_millis:.6}\n{samples}\n",
        reports.len()
    )
}

fn save_viewport_frame_png(frame: &ViewportFrame, output: &Path) {
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("realtime IBL frame dimensions")
        .save_with_format(output, ImageFormat::Png)
        .expect("write realtime IBL PBR matrix screenshot");
}

fn save_viewport_frame_contact_sheet_png(frames: &[ViewportFrame], columns: u32, output: &Path) {
    assert!(!frames.is_empty());
    assert!(columns > 0);
    let tile_width = frames[0].width;
    let tile_height = frames[0].height;
    let rows = (frames.len() as u32 + columns - 1) / columns;
    let mut sheet = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(tile_width * columns, tile_height * rows);
    for (index, frame) in frames.iter().enumerate() {
        assert_eq!((frame.width, frame.height), (tile_width, tile_height));
        let column = index as u32 % columns;
        let row = index as u32 / columns;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let source = ((y * tile_width + x) * 4) as usize;
                sheet.put_pixel(
                    column * tile_width + x,
                    row * tile_height + y,
                    Rgba([
                        frame.rgba[source],
                        frame.rgba[source + 1],
                        frame.rgba[source + 2],
                        frame.rgba[source + 3],
                    ]),
                );
            }
        }
    }
    sheet
        .save_with_format(output, ImageFormat::Png)
        .expect("write realtime IBL 120-degree contact sheet");
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
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader");
    fs::create_dir_all(&output).unwrap();
    output
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_{label}_{}_{}_{}",
        std::process::id(),
        NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed),
        unique
    ))
}
