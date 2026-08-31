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
    CameraRenderDescriptor, EnvironmentExtract, PreviewEnvironmentExtract, ProjectionMode,
    RenderFrameExtract, RenderFramework, RenderLayerSet, RenderOverlayExtract,
    RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
    DEFAULT_RENDER_LAYER_MASK,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::ResourceState;
use zircon_runtime::graphics::{
    RealtimeIblCpuTimingReport, RealtimeIblGpuTimingReport, ViewportFrame, WgpuRenderFramework,
};

#[path = "runtime_shader_pbr_realtime_ibl_export/cpu_profile_capture.rs"]
mod cpu_profile_capture;
#[path = "runtime_shader_pbr_realtime_ibl_export/gltf_zero_roughness.rs"]
mod gltf_zero_roughness;
#[path = "runtime_shader_pbr_hdri_export/scene_fixtures.rs"]
mod scene_fixtures;
mod support;
#[path = "runtime_shader_pbr_realtime_ibl_export/timing_reports.rs"]
mod timing_reports;

use cpu_profile_capture::{clear_current_cpu_timing_sidecar, RealtimeIblCpuProfileCapture};
use scene_fixtures::{
    write_pbr_matrix_material, write_pbr_matrix_scene, write_single_pbr_material,
    write_single_pbr_sphere_scene_with_camera_view, write_uv_sphere_model,
    SinglePbrSphereCameraView,
};
use timing_reports::{
    assert_realtime_binding_cache_metrics, assert_realtime_capture_and_source_mip_binding_metrics,
    assert_realtime_cpu_timings, assert_realtime_gpu_timings, cpu_timing_report, gpu_timing_report,
};

const PBR_MATRIX_DIMENSION: usize = 8;
const PBR_MATRIX_OUTPUT_SIZE: UVec2 = UVec2::new(1600, 1200);
const PBR_MATRIX_ORTHO_SIZE: f32 = 5.8;
const PBR_MATRIX_STEP_X: f32 = 0.7;
const PBR_MATRIX_STEP_Y: f32 = 0.62;
const PBR_MATRIX_SPHERE_SCALE: f32 = 0.21;
const REALTIME_GENERATION_TICKET_FRAME_COUNT: usize = 21;
const REALTIME_GENERATION_TICKET_COUNT: usize = 3;
const RENDERDOC_CAPTURE_FINAL_SH9_SLICE_ENV: &str = "ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9";
const OUTPUT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_reflection_20260823_p1_2.png";
const TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_timing_20260823_p1_2.txt";
const GPU_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_gpu_timing_20260823_p1_2.txt";
const CPU_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_cpu_timing_20260824.txt";
const MULTI_VIEW_OUTPUT_SIZE: UVec2 = UVec2::new(800, 600);
const MULTI_VIEW_COLUMNS: u32 = 5;
const MULTI_VIEW_CONTACT_SHEET_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_mirror_cardinal_120deg_contact_sheet_20260714.png";
const MULTI_VIEW_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_procedural_realtime_ibl_mirror_cardinal_120deg_timing_20260815.txt";
// The fixed directional-sky mirror baseline has 63 pixels at or above 700 and a 765 maximum.
const DIRECTIONAL_PROCEDURAL_MIRROR_MIN_HIGHLIGHT_RGB_SUM: u16 = 700;
const DIRECTIONAL_PROCEDURAL_MIRROR_MAX_HIGHLIGHT_PIXELS: usize = 128;

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
fn realtime_ibl_export_contract_uses_requested_matrix_and_ticket_budget() {
    assert_eq!(PBR_MATRIX_DIMENSION * PBR_MATRIX_DIMENSION, 64);
    assert_eq!(pbr_matrix_axis_value(0), 0.0);
    assert_eq!(pbr_matrix_axis_value(PBR_MATRIX_DIMENSION - 1), 1.0);
    assert_eq!(REALTIME_GENERATION_TICKET_FRAME_COUNT, 21);
    assert_eq!(realtime_multiview_cases().len(), 5);

    let source = include_str!("runtime_shader_pbr_realtime_ibl_export.rs");
    let multiview_export = source
        .split("\nfn export_procedural_realtime_ibl_mirror_cardinal_120deg_png()")
        .nth(1)
        .and_then(|body| body.split("\nfn prepare_matrix_project").next())
        .expect("multiview export body");
    let (setup, view_loop) = multiview_export
        .split_once("\n    for view_case in cases {")
        .expect("multiview view loop");
    for cold_initialization in [
        "ProjectAssetManager::default",
        ".open_project(",
        "ProjectManager::open",
        "project.scan_and_import()",
        "World::load_scene_from_uri",
        "ProjectAssetTestRuntime::new",
        "WgpuRenderFramework::new",
    ] {
        assert_eq!(
            multiview_export.matches(cold_initialization).count(),
            1,
            "multiview export must initialize {cold_initialization} exactly once"
        );
        assert!(
            setup.contains(cold_initialization),
            "multiview setup must retain {cold_initialization}"
        );
        assert!(
            !view_loop.contains(cold_initialization),
            "multiview view loop must not repeat {cold_initialization}"
        );
    }
    assert!(view_loop.contains("realtime_mirror_camera_descriptor"));
    assert!(!view_loop.contains("render_realtime_mirror_view"));
    assert!(source.contains(concat!("request_graphics_debugger_", "capture(viewport)")));
    assert!(source.contains("final SH9 RenderDoc capture must complete without a debugger error"));
    assert!(source.contains(concat!("submit_compiled_realtime_ibl_", "frame(")));
    assert!(!source.contains(concat!("SceneRenderer", "::new")));
    assert!(!source.contains(concat!("start_graphics_debugger", "_capture")));
    assert!(!source.contains(concat!("stop_graphics_debugger", "_capture")));
}

#[test]
fn realtime_multiview_camera_descriptor_matches_fixture_camera() {
    let expected_layers = RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK);
    for camera_view in [
        SinglePbrSphereCameraView::front(ProjectionMode::Perspective),
        SinglePbrSphereCameraView::front(ProjectionMode::Orthographic),
        SinglePbrSphereCameraView::perspective_orbit_degrees(120.0, -120.0),
    ] {
        let descriptor = realtime_mirror_camera_descriptor(camera_view, MULTI_VIEW_OUTPUT_SIZE);
        let eye = Vec3::new(camera_view.eye[0], camera_view.eye[1], camera_view.eye[2]);
        let target = Vec3::new(
            camera_view.target[0],
            camera_view.target[1],
            camera_view.target[2],
        );

        assert_eq!(
            descriptor.camera.transform,
            Transform::looking_at(eye, target, Vec3::Y)
        );
        assert_eq!(
            descriptor.camera.projection_mode,
            camera_view.projection_mode
        );
        assert_eq!(descriptor.camera.fov_y_radians, 60.0_f32.to_radians());
        assert_eq!(descriptor.camera.ortho_size, camera_view.ortho_size);
        assert_eq!(descriptor.camera.z_near, 0.1);
        assert_eq!(descriptor.camera.z_far, 100.0);
        assert_eq!(
            descriptor.camera.aspect_ratio,
            MULTI_VIEW_OUTPUT_SIZE.x as f32 / MULTI_VIEW_OUTPUT_SIZE.y as f32
        );
        assert_eq!(descriptor.culling_mask, expected_layers);
        assert_eq!(descriptor.volume_mask, expected_layers);
    }
}

#[test]
fn realtime_multiview_timing_report_separates_setup_and_reused_frames() {
    let report = multiview_timing_report(12.5, &[7.0, 3.0, 4.0]);

    assert!(report.contains("multiview_setup_cpu_ms=12.500"));
    assert!(report.contains("first_view_render_cpu_ms=7.000"));
    assert!(report.contains("reused_view_render_count=2"));
    assert!(report.contains("reused_view_render_total_cpu_ms=7.000"));
    assert!(report.contains("reused_view_render_average_cpu_ms=3.500"));
    assert!(report.contains("view_02_render_cpu_ms=4.000"));
}

#[test]
fn realtime_ibl_cpu_timing_report_preserves_cpu_clock_boundaries() {
    let report = RealtimeIblCpuTimingReport {
        profile_capture_epoch: 17,
        frame_number: 4,
        generation_start_frame_number: 1,
        generation_elapsed_frame_count: 4,
        coalesced_source_change_count: 3,
        queued_generation_pending: true,
        command_plan_creation_micros: 13,
        pipeline_ensure_micros: 7,
        binding_creation_micros: 5,
        execution_resource_binding_micros: 3,
        validation_micros: 2,
        execution_resource_cache_hits: 1,
        execution_resource_cache_misses: 0,
        execution_resource_cache_entry_count: 2,
        execution_resource_cache_topology_capacity: 42,
        ..RealtimeIblCpuTimingReport::default()
    };

    let serialized = cpu_timing_report(&[report]);

    assert!(serialized.contains("clock_domain=cpu_command_recording_only"));
    assert!(serialized.contains("profile_capture_epoch=17"));
    assert!(serialized.contains("generation_start_frame=1"));
    assert!(serialized.contains("generation_elapsed_frames=4"));
    assert!(serialized.contains("coalesced_source_changes=3"));
    assert!(serialized.contains("queued_generation_pending=true"));
    assert!(serialized.contains("command_plan_creation_micros=13"));
    assert!(serialized.contains("execution_resource_cache_hits=1"));
    assert!(serialized.contains("execution_resource_cache_entry_count=2"));
    assert!(serialized.contains("execution_resource_cache_topology_capacity=42"));
    assert!(!serialized.contains("elapsed_gpu_nanoseconds"));
}

#[test]
fn realtime_ibl_cpu_timing_sidecar_uses_shared_stale_file_cleanup() {
    let source = include_str!("runtime_shader_pbr_realtime_ibl_export.rs");
    let matrix_export = source
        .split("\nfn export_procedural_realtime_ibl_pbr_matrix_png()")
        .nth(1)
        .and_then(|body| {
            body.split("\nfn export_procedural_realtime_ibl_mirror_cardinal_120deg_png()")
                .next()
        })
        .expect("realtime IBL matrix export body");

    assert!(matrix_export.contains("clear_current_cpu_timing_sidecar(&cpu_timing_output);"));
    assert!(!matrix_export.contains("match fs::remove_file(&cpu_timing_output)"));
}

#[test]
fn realtime_ibl_gpu_timing_sidecar_excludes_cpu_clock_windows() {
    let source = include_str!("runtime_shader_pbr_realtime_ibl_export/timing_reports.rs");
    let gpu_report = source
        .split("pub(super) fn gpu_timing_report")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn cpu_timing_report").next())
        .expect("GPU timing sidecar formatter");

    for cpu_window in [
        "binding_creation_micros=",
        "capture_binding_creation_micros=",
        "source_mip_binding_creation_micros=",
        "execution_resource_cache_hits=",
        "execution_resource_cache_misses=",
        "execution_resource_cache_entry_count=",
        "execution_resource_cache_topology_capacity=",
    ] {
        assert!(
            !gpu_report.contains(cpu_window),
            "GPU timing sidecar must not serialize CPU clock window {cpu_window}"
        );
    }
}

#[test]
fn realtime_ibl_export_temporary_projects_stay_under_evidence_root() {
    let root = unique_temp_project_root("shader_pbr_realtime_ibl_test");

    assert!(root.starts_with(shader_test_output_dir().join(".work")));
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
    let imported = project.scan_and_import().unwrap();
    let scene_record = imported
        .iter()
        .find(|record| record.primary_locator() == &scene_uri)
        .expect("realtime IBL matrix scene must be discovered during import");
    assert_eq!(
        scene_record.state,
        ResourceState::Ready,
        "realtime IBL matrix scene must publish an artifact: {:#?}",
        scene_record.diagnostics
    );
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
    let framework =
        WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool()).unwrap();
    let viewport = framework
        .create_viewport(
            RenderViewportDescriptor::new(PBR_MATRIX_OUTPUT_SIZE)
                .with_label("shader06.realtime-ibl-matrix"),
        )
        .unwrap();
    let cpu_timing_output = shader_test_output_dir().join(CPU_TIMING_REPORT_NAME);
    assert!(cpu_timing_output.starts_with(shader_test_output_dir()));
    clear_current_cpu_timing_sidecar(&cpu_timing_output);
    let cpu_timing_feature_enabled = RealtimeIblCpuProfileCapture::feature_enabled();
    let mut cpu_timing_capture = RealtimeIblCpuProfileCapture::begin();
    let mut initial_ticket_millis = Vec::with_capacity(REALTIME_GENERATION_TICKET_FRAME_COUNT);
    for _ in 0..REALTIME_GENERATION_TICKET_FRAME_COUNT {
        let started = Instant::now();
        submit_compiled_realtime_ibl_frame(&framework, viewport, snapshot.clone());
        initial_ticket_millis.push(started.elapsed().as_secs_f64() * 1000.0);
    }

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

    let mut slice_millis = Vec::with_capacity(REALTIME_GENERATION_TICKET_FRAME_COUNT);
    let capture_final_sh9_slice =
        std::env::var(RENDERDOC_CAPTURE_FINAL_SH9_SLICE_ENV).is_ok_and(|value| value == "1");
    for slice_index in 0..REALTIME_GENERATION_TICKET_FRAME_COUNT {
        let capture_this_slice =
            capture_final_sh9_slice && slice_index + 1 == REALTIME_GENERATION_TICKET_FRAME_COUNT;
        if capture_this_slice {
            framework
                .request_graphics_debugger_capture(viewport)
                .expect("request RenderDoc capture for final SH9 update slice");
        }
        let started = Instant::now();
        submit_compiled_realtime_ibl_frame(&framework, viewport, updated_snapshot.clone());
        if capture_this_slice {
            let capture_status = framework
                .query_graphics_debugger_status()
                .expect("query final SH9 RenderDoc capture status");
            assert!(
                !capture_status.capture_pending,
                "final SH9 RenderDoc capture must complete in its requested frame"
            );
            assert_eq!(
                capture_status.last_error, None,
                "final SH9 RenderDoc capture must complete without a debugger error"
            );
        }
        slice_millis.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    let mut warm_snapshot = updated_snapshot;
    warm_snapshot.environment.skybox.procedural.source_revision = warm_snapshot
        .environment
        .skybox
        .procedural
        .source_revision
        .wrapping_add(1);
    warm_snapshot.preview =
        PreviewEnvironmentExtract::from_environment(&warm_snapshot.environment, true, Vec4::ZERO);
    let mut warm_ticket_millis = Vec::with_capacity(REALTIME_GENERATION_TICKET_FRAME_COUNT);
    for slice_index in 0..REALTIME_GENERATION_TICKET_FRAME_COUNT {
        let capture_this_slice =
            capture_final_sh9_slice && slice_index + 1 == REALTIME_GENERATION_TICKET_FRAME_COUNT;
        if capture_this_slice {
            framework
                .request_graphics_debugger_capture(viewport)
                .expect("request RenderDoc capture for final SH9 warm slice");
        }
        let started = Instant::now();
        submit_compiled_realtime_ibl_frame(&framework, viewport, warm_snapshot.clone());
        if capture_this_slice {
            let capture_status = framework
                .query_graphics_debugger_status()
                .expect("query final SH9 RenderDoc capture status");
            assert!(
                !capture_status.capture_pending,
                "final SH9 RenderDoc capture must complete in its requested frame"
            );
            assert_eq!(
                capture_status.last_error, None,
                "final SH9 RenderDoc capture must complete without a debugger error"
            );
        }
        warm_ticket_millis.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    submit_compiled_realtime_ibl_frame(&framework, viewport, warm_snapshot);
    let final_frame = capture_compiled_viewport_frame(&framework, viewport);
    assert!(
        framework.realtime_ibl_gpu_timing_supported(),
        "framework must expose compiled realtime IBL timestamp queries for EC-M4 acceptance"
    );
    let gpu_timings = framework
        .take_realtime_ibl_gpu_timing_reports()
        .expect("drain compiled realtime IBL GPU timing reports");
    assert_realtime_gpu_timings(&gpu_timings, REALTIME_GENERATION_TICKET_COUNT);
    assert_realtime_binding_cache_metrics(&gpu_timings);
    assert_realtime_capture_and_source_mip_binding_metrics(
        &gpu_timings,
        REALTIME_GENERATION_TICKET_COUNT,
    );
    let cpu_timings = if cpu_timing_capture.has_owned_capture() {
        cpu_timing_capture.stop();
        framework
            .take_realtime_ibl_cpu_timing_reports()
            .expect("drain compiled realtime IBL CPU timing reports")
    } else if !cpu_timing_feature_enabled {
        framework
            .take_realtime_ibl_cpu_timing_reports()
            .expect("drain disabled realtime IBL CPU timing reports")
    } else {
        Vec::new()
    };
    if cpu_timing_capture.has_owned_capture() {
        assert_realtime_cpu_timings(&cpu_timings, REALTIME_GENERATION_TICKET_COUNT);
    } else if !cpu_timing_feature_enabled {
        assert!(
            cpu_timings.is_empty(),
            "CPU timing reports require the profiling feature capture"
        );
    }

    let output = shader_test_output_dir().join(OUTPUT_NAME);
    save_viewport_frame_png(&final_frame, &output);
    assert_realtime_matrix_image(&final_frame);
    let timing_output = shader_test_output_dir().join(TIMING_REPORT_NAME);
    fs::write(
        &timing_output,
        timing_report(&initial_ticket_millis, &slice_millis, &warm_ticket_millis),
    )
    .expect("write realtime IBL frame timing report");
    let gpu_timing_output = shader_test_output_dir().join(GPU_TIMING_REPORT_NAME);
    fs::write(&gpu_timing_output, gpu_timing_report(&gpu_timings))
        .expect("write realtime IBL GPU timing report");
    if cpu_timing_capture.has_owned_capture() {
        fs::write(&cpu_timing_output, cpu_timing_report(&cpu_timings))
            .expect("write realtime IBL CPU timing report");
    }
    assert!(output.starts_with(shader_test_output_dir()));
    assert!(timing_output.starts_with(shader_test_output_dir()));
    assert!(gpu_timing_output.starts_with(shader_test_output_dir()));
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "manual WGPU product acceptance for Shader 06 procedural realtime IBL multiview"]
fn export_procedural_realtime_ibl_mirror_cardinal_120deg_png() {
    let setup_started = Instant::now();
    let cases = realtime_multiview_cases();
    let root = unique_temp_project_root("shader_pbr_realtime_ibl_multiview");
    let paths = prepare_single_mirror_project(&root, cases[0].camera_view);
    let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml").unwrap();
    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world =
        zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let environment = directional_procedural_environment();
    let asset_runtime = support::ProjectAssetTestRuntime::new(asset_manager);
    let framework =
        WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool()).unwrap();
    let viewport = framework
        .create_viewport(
            RenderViewportDescriptor::new(MULTI_VIEW_OUTPUT_SIZE)
                .with_label("shader06.realtime-ibl-multiview"),
        )
        .unwrap();
    let setup_elapsed = setup_started.elapsed();
    let mut frames = Vec::new();
    let mut render_millis = Vec::with_capacity(cases.len());
    for view_case in cases {
        let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
            settings: ViewportRenderSettings::default(),
            active_camera_override: None,
            camera: Some(realtime_mirror_camera_descriptor(
                view_case.camera_view,
                MULTI_VIEW_OUTPUT_SIZE,
            )),
            viewport_size: Some(MULTI_VIEW_OUTPUT_SIZE),
            virtual_geometry_debug: None,
        });
        snapshot.environment = environment.clone();
        snapshot.preview =
            PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
        snapshot.overlays = RenderOverlayExtract::default();
        let render_started = Instant::now();
        submit_compiled_realtime_ibl_frame(&framework, viewport, snapshot);
        let frame = capture_compiled_viewport_frame(&framework, viewport);
        render_millis.push(render_started.elapsed().as_secs_f64() * 1000.0);
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
    let timing_output = shader_test_output_dir().join(MULTI_VIEW_TIMING_REPORT_NAME);
    fs::write(
        &timing_output,
        multiview_timing_report(setup_elapsed.as_secs_f64() * 1000.0, &render_millis),
    )
    .expect("write realtime IBL multiview timing report");
    assert!(timing_output.starts_with(shader_test_output_dir()));
    let _ = fs::remove_dir_all(root);
    drop(paths);
}

fn submit_compiled_realtime_ibl_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    snapshot: zircon_runtime::core::framework::render::RenderSceneSnapshot,
) {
    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(0), snapshot),
        )
        .expect("submit compiled realtime IBL frame");
}

fn capture_compiled_viewport_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> ViewportFrame {
    let captured = framework
        .capture_frame(viewport)
        .expect("capture compiled realtime IBL frame")
        .expect("compiled realtime IBL frame should be available");
    ViewportFrame {
        width: captured.width,
        height: captured.height,
        rgba: captured.rgba,
        generation: captured.generation,
        capture_report: captured.capture_report,
    }
}

fn realtime_mirror_camera_descriptor(
    camera_view: SinglePbrSphereCameraView,
    viewport_size: UVec2,
) -> CameraRenderDescriptor {
    let eye = Vec3::new(camera_view.eye[0], camera_view.eye[1], camera_view.eye[2]);
    let target = Vec3::new(
        camera_view.target[0],
        camera_view.target[1],
        camera_view.target[2],
    );
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(eye, target, Vec3::Y),
        projection_mode: camera_view.projection_mode,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: camera_view.ortho_size,
        z_near: 0.1,
        z_far: 100.0,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, camera);
    let default_layers = RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK);
    descriptor.culling_mask = default_layers.clone();
    descriptor.volume_mask = default_layers;
    descriptor.apply_target_size(viewport_size);
    descriptor
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

fn multiview_timing_report(setup_millis: f64, render_millis: &[f64]) -> String {
    let first_render_millis = render_millis.first().copied().unwrap_or_default();
    let reused_render_millis = &render_millis[render_millis.len().min(1)..];
    let reused_total_millis = reused_render_millis.iter().sum::<f64>();
    let reused_average_millis = reused_total_millis / reused_render_millis.len().max(1) as f64;
    let samples = render_millis
        .iter()
        .enumerate()
        .map(|(index, millis)| format!("view_{index:02}_render_cpu_ms={millis:.3}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "multiview_setup_cpu_ms={setup_millis:.3}\nfirst_view_render_cpu_ms={first_render_millis:.3}\nreused_view_render_count={}\nreused_view_render_total_cpu_ms={reused_total_millis:.3}\nreused_view_render_average_cpu_ms={reused_average_millis:.3}\n{samples}\n",
        reused_render_millis.len()
    )
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

fn assert_directional_procedural_mirror_highlight(frame: &ViewportFrame, label: &str) {
    let (x, y, brightest_sum) = brightest_pixel_position(frame);
    assert!(
        brightest_sum >= DIRECTIONAL_PROCEDURAL_MIRROR_MIN_HIGHLIGHT_RGB_SUM,
        "{label} must retain the near-mirror directional-sun highlight: brightest_sum={brightest_sum}"
    );
    assert!(
        (frame.width / 3..=frame.width * 2 / 3).contains(&x)
            && (frame.height / 4..=frame.height * 3 / 4).contains(&y),
        "{label} directional-sun highlight must remain on the sphere: x={x}, y={y}, width={}, height={}",
        frame.width,
        frame.height
    );
    let highlight_pixel_count = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])
                >= DIRECTIONAL_PROCEDURAL_MIRROR_MIN_HIGHLIGHT_RGB_SUM
        })
        .count();
    assert!(
        (1..=DIRECTIONAL_PROCEDURAL_MIRROR_MAX_HIGHLIGHT_PIXELS)
            .contains(&highlight_pixel_count),
        "{label} directional-sun highlight must stay localized: count={highlight_pixel_count}, max={DIRECTIONAL_PROCEDURAL_MIRROR_MAX_HIGHLIGHT_PIXELS}"
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

fn timing_report(
    initial_ticket_millis: &[f64],
    update_ticket_millis: &[f64],
    warm_ticket_millis: &[f64],
) -> String {
    let initial_average =
        initial_ticket_millis.iter().sum::<f64>() / initial_ticket_millis.len().max(1) as f64;
    let initial_maximum = initial_ticket_millis
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let update_average =
        update_ticket_millis.iter().sum::<f64>() / update_ticket_millis.len().max(1) as f64;
    let update_maximum = update_ticket_millis.iter().copied().fold(0.0_f64, f64::max);
    let warm_average =
        warm_ticket_millis.iter().sum::<f64>() / warm_ticket_millis.len().max(1) as f64;
    let warm_maximum = warm_ticket_millis.iter().copied().fold(0.0_f64, f64::max);
    let initial_slices = initial_ticket_millis
        .iter()
        .enumerate()
        .map(|(index, millis)| format!("initial_ticket_frame_{index:02}_cpu_ms={millis:.3}"))
        .collect::<Vec<_>>()
        .join("\n");
    let update_slices = update_ticket_millis
        .iter()
        .enumerate()
        .map(|(index, millis)| format!("update_ticket_frame_{index:02}_cpu_ms={millis:.3}"))
        .collect::<Vec<_>>()
        .join("\n");
    let warm_slices = warm_ticket_millis
        .iter()
        .enumerate()
        .map(|(index, millis)| format!("warm_ticket_frame_{index:02}_cpu_ms={millis:.3}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "initial_ticket_frame_count={}\ninitial_ticket_average_cpu_ms={initial_average:.3}\ninitial_ticket_max_cpu_ms={initial_maximum:.3}\nupdate_ticket_frame_count={}\nupdate_ticket_average_cpu_ms={update_average:.3}\nupdate_ticket_max_cpu_ms={update_maximum:.3}\nwarm_ticket_frame_count={}\nwarm_ticket_average_cpu_ms={warm_average:.3}\nwarm_ticket_max_cpu_ms={warm_maximum:.3}\n{initial_slices}\n{update_slices}\n{warm_slices}\n",
        initial_ticket_millis.len(),
        update_ticket_millis.len(),
        warm_ticket_millis.len(),
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
    let scratch_root = shader_test_output_dir().join(".work");
    fs::create_dir_all(&scratch_root).expect("create Shader 06 temporary project root");
    scratch_root.join(format!(
        "zircon_{label}_{}_{}_{}",
        std::process::id(),
        NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed),
        unique
    ))
}
