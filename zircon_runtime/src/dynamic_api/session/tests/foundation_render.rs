use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{project_asset_manager_handle, AssetUri, ProjectPaths};
use crate::core::framework::input::{InputButton, InputEvent};
use crate::core::framework::render::RenderStats;
use crate::core::manager::resolve_manager_service;
use crate::core::resource::ResourceState;
use crate::runtime_diagnostics::collect_runtime_diagnostics;
use image::ImageFormat;
use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile, RuntimeProjectConfig};

const CAPTURE_WIDTH: u32 = 640;
const CAPTURE_HEIGHT: u32 = 360;
const CAPTURE_ENV: &str = "ZR_F2_BASIC_SCENE_CAPTURE_PNG";

#[test]
fn render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down() {
    let project = F2Project::create();
    let config = RuntimeProjectConfig::from_root(project.root.clone())
        .expect("F2 project root should resolve before session creation");

    let (first, second_gpu_upload_bytes) = {
        let mut session =
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Runtime, Some(config.clone()))
                .expect("F2 runtime session should load the persisted project");
        assert_template_assets_ready(&session);
        assert_input_ingress(&mut session);
        session.tick_frame().expect("F2 runtime tick");

        let first = capture_product_frame(&mut session);
        assert_basic_scene_frame(&first, "first launch");
        assert_product_diagnostics(&session);
        export_capture_if_requested(&first);

        let second = capture_product_frame(&mut session);
        assert_basic_scene_frame(&second, "unchanged second frame");
        assert_steady_state_performance(&first.stats, &second.stats);
        (first, second.stats.last_gpu_scene_uploaded_bytes)
    };

    let restarted = {
        let mut session =
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Runtime, Some(config))
                .expect("F2 runtime session should restart after deterministic teardown");
        assert_template_assets_ready(&session);
        session.tick_frame().expect("restarted F2 runtime tick");
        capture_product_frame(&mut session)
    };
    assert_basic_scene_frame(&restarted, "second launch after teardown");
    assert_eq!(
        restarted.stats.last_mesh_draw_count, first.stats.last_mesh_draw_count,
        "restarting the persisted project must reproduce the same visible mesh draw count"
    );
    assert_eq!(
        restarted.stats.last_directional_light_count, first.stats.last_directional_light_count,
        "restarting the persisted project must reproduce the same directional light count"
    );

    println!(
        "f2_basic_scene first_passes={} first_draws={} first_lights={} graph_hits={} graph_misses={} second_gpu_upload_bytes={} restarted_draws={}",
        first.stats.last_graph_executed_pass_count,
        first.stats.last_mesh_draw_count,
        first.stats.last_directional_light_count,
        first.stats.last_graph_compiled_cache_hit_count,
        first.stats.last_graph_compiled_cache_miss_count,
        second_gpu_upload_bytes,
        restarted.stats.last_mesh_draw_count,
    );
    project.assert_removable_after_sessions_drop();
}

#[test]
fn f2_exported_png_roundtrips_captured_rgba() {
    let root = unique_f2_capture_root("png-roundtrip");
    let path = root.join("frame.png");
    let rgba = vec![
        0, 0, 0, 0, // Transparent clear pixel.
        12, 34, 56, 255, // Opaque visible pixel.
        90, 80, 70, 128, // Partial alpha must survive PNG encoding.
        255, 255, 255, 1,
    ];

    write_f2_capture_png(&path, 2, 2, &rgba);
    assert_f2_capture_png(&path, 2, 2, &rgba);

    std::fs::remove_dir_all(root).expect("remove F2 PNG roundtrip fixture");
}

#[test]
fn f2_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = unique_f2_fixture_root("root-location");
    let executable = std::env::current_exe().expect("locate the F2 test executable");
    let binary_directory = executable
        .parent()
        .expect("F2 test executable must have a parent directory");
    let resolved_binary_directory =
        ProjectPaths::resolve_existing(binary_directory).expect("resolve F2 test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "F2 fixture output must retain the test binary's physical output root"
    );
}

fn assert_template_assets_ready(session: &RuntimeDynamicSession) {
    let core = session.runtime.handle();
    let handle = project_asset_manager_handle(&core).expect("F2 project asset manager handle");
    let manager =
        resolve_manager_service(&core, handle).expect("resolve F2 project asset manager service");
    let project = manager
        .current_project_manager()
        .expect("F2 runtime must retain the opened project");

    for uri in [
        "res://scenes/main.scene.toml",
        "res://models/cube.obj",
        "res://materials/default.zmaterial",
        "res://shaders/pbr_shader",
    ] {
        let uri = AssetUri::parse(uri).expect("F2 template asset URI");
        let record = project
            .registry()
            .get_by_locator(&uri)
            .unwrap_or_else(|| panic!("F2 project is missing imported asset {uri}"));
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "F2 asset {uri} must import successfully: {}",
            record.failure_reason().unwrap_or("no import diagnostic")
        );
        assert!(
            record.artifact_locator().is_some(),
            "F2 asset {uri} must have an artifact locator"
        );
    }
}

fn assert_input_ingress(session: &mut RuntimeDynamicSession) {
    let viewport = ZrRuntimeViewportHandle::new(1);
    let resized_size = ZrRuntimeViewportSizeV1::new(CAPTURE_WIDTH / 2, CAPTURE_HEIGHT / 2);
    let pointer = [CAPTURE_WIDTH as f32 * 0.5, CAPTURE_HEIGHT as f32 * 0.5];
    let events = [
        ZrRuntimeEventV1::viewport_resized(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, resized_size),
        ZrRuntimeEventV1::pointer_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            pointer[0],
            pointer[1],
        ),
        ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            pointer[0],
            pointer[1],
        ),
        ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
            u32::from(b'W'),
            0,
            ZrByteSlice::from_static(b"W"),
        ),
    ];
    for event in events {
        let status = session.handle_event(event);
        assert!(
            status.is_ok(),
            "F2 input event should be accepted: {status:?}"
        );
    }
    assert_eq!(
        session.camera_controller.viewport_size(),
        crate::core::math::UVec2::new(resized_size.width, resized_size.height),
        "F2 viewport-resize ingress must update the runtime viewport before frame capture chooses its own size"
    );

    let input_events = session
        .resolve_input_manager()
        .expect("F2 input manager")
        .drain_events();
    assert!(input_events.iter().any(|event| matches!(
        event,
        InputEvent::CursorMoved { x, y } if *x == pointer[0] && *y == pointer[1]
    )));
    assert!(input_events
        .iter()
        .any(|event| matches!(event, InputEvent::ButtonPressed(_))));
    assert!(input_events.iter().any(|event| matches!(
        event,
        InputEvent::KeyboardInput {
            key_code,
            pressed: true,
            ..
        } if *key_code == u32::from(b'W')
    )));

    for event in [
        ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
            u32::from(b'W'),
            0,
            ZrByteSlice::from_static(b"W"),
        ),
        ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
            pointer[0],
            pointer[1],
        ),
    ] {
        let status = session.handle_event(event);
        assert!(
            status.is_ok(),
            "F2 release event should be accepted: {status:?}"
        );
    }
    let released_events = session
        .resolve_input_manager()
        .expect("F2 input manager")
        .drain_events();
    assert!(released_events
        .iter()
        .any(|event| matches!(event, InputEvent::ButtonReleased(InputButton::MouseLeft))));
    assert!(released_events.iter().any(|event| matches!(
        event,
        InputEvent::KeyboardInput {
            key_code,
            pressed: false,
            ..
        } if *key_code == u32::from(b'W')
    )));
}

fn capture_product_frame(session: &mut RuntimeDynamicSession) -> ProductFrame {
    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(CAPTURE_WIDTH, CAPTURE_HEIGHT),
        ))
        .expect("F2 WGPU frame capture");
    let rgba = frame.rgba;
    let stats = collect_runtime_diagnostics(&session.runtime.handle())
        .render
        .stats
        .expect("F2 capture must publish render stats");
    ProductFrame {
        width: frame.width,
        height: frame.height,
        rgba,
        stats,
    }
}

fn assert_basic_scene_frame(frame: &ProductFrame, label: &str) {
    assert_eq!((frame.width, frame.height), (CAPTURE_WIDTH, CAPTURE_HEIGHT));
    assert_eq!(
        frame.rgba.len(),
        (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize,
        "{label} must return a complete RGBA frame"
    );
    let non_transparent_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[3] != 0)
        .count();
    assert!(
        non_transparent_pixels > 0,
        "{label} must contain non-transparent pixels in the presented RGBA frame"
    );
    let background = &frame.rgba[..4];
    let changed_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();
    assert!(
        changed_pixels > 100,
        "{label} must contain visible non-background output, changed_pixels={changed_pixels}"
    );
    assert!(
        frame.stats.last_graph_executed_pass_count > 0,
        "{label} must execute the RenderGraph"
    );
    assert!(
        frame.stats.last_mesh_draw_count > 0,
        "{label} must submit the persisted visible primitive"
    );
    assert!(
        frame.stats.last_directional_light_count > 0,
        "{label} must extract the persisted directional light"
    );
    assert_eq!(
        frame.stats.last_material_validation_error_count, 0,
        "{label} must not hide material validation errors"
    );
    assert_eq!(
        frame.stats.last_material_fallback_count, 0,
        "{label} must render the persisted material without fallback resources"
    );
}

fn assert_product_diagnostics(session: &RuntimeDynamicSession) {
    let diagnostics = super::super::diagnostics::runtime_diagnostics_response(session)
        .runtime_diagnostics
        .expect("F2 runtime diagnostics snapshot");

    assert_eq!(
        diagnostics.project_identity.as_deref(),
        Some("F2BasicScene"),
        "F2 diagnostics must identify the opened project"
    );
    assert_eq!(
        diagnostics.scene_uri.as_deref(),
        Some("res://scenes/main.scene.toml"),
        "F2 diagnostics must identify the persisted default scene"
    );
    assert!(
        diagnostics
            .render_backend_name
            .is_some_and(|name| !name.trim().is_empty()),
        "F2 diagnostics must identify the active render backend"
    );
}

fn assert_steady_state_performance(first: &RenderStats, second: &RenderStats) {
    assert!(first.last_graph_compiled_cache_miss_count > 0);
    assert_eq!(
        second.last_graph_compiled_cache_miss_count, first.last_graph_compiled_cache_miss_count,
        "an unchanged F2 frame must not recompile the RenderGraph"
    );
    assert!(
        second.last_graph_compiled_cache_hit_count > first.last_graph_compiled_cache_hit_count,
        "an unchanged F2 frame must reuse the compiled RenderGraph: first_hits={}, second_hits={}",
        first.last_graph_compiled_cache_hit_count,
        second.last_graph_compiled_cache_hit_count,
    );
    assert_eq!(
        second.last_graph_compiled_cache_entry_count, first.last_graph_compiled_cache_entry_count,
        "steady state must not grow the compiled RenderGraph cache"
    );
    assert_eq!(
        second.last_mesh_draw_count, first.last_mesh_draw_count,
        "steady state must preserve the visible draw set"
    );
    assert_eq!(
        second.last_gpu_scene_dirty_entry_count, 0,
        "an unchanged static scene must not leave dirty GPUScene entries"
    );
    assert_eq!(
        second.last_gpu_scene_uploaded_bytes, 0,
        "an unchanged static scene must not upload GPUScene data again"
    );
}

fn export_capture_if_requested(frame: &ProductFrame) {
    let Ok(path) = std::env::var(CAPTURE_ENV) else {
        return;
    };
    let path = Path::new(&path);
    write_f2_capture_png(path, frame.width, frame.height, &frame.rgba);
    assert_f2_capture_png(path, frame.width, frame.height, &frame.rgba);
}

fn write_f2_capture_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create F2 capture output directory");
    }
    let image = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .expect("F2 capture buffer must match dimensions");
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("write F2 product capture PNG");
}

fn assert_f2_capture_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    let captured = image::open(path)
        .expect("read F2 product capture PNG")
        .to_rgba8();
    assert_eq!(
        captured.dimensions(),
        (width, height),
        "F2 product capture must preserve frame dimensions"
    );
    assert_eq!(
        captured.as_raw(),
        rgba,
        "F2 product capture must preserve RGBA pixels, including alpha and visible primitive output"
    );
}

fn unique_f2_capture_root(label: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    unique_f2_fixture_root(format!(
        "capture-{label}-{}_{}_{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed),
        unique
    ))
}

fn unique_f2_fixture_root(label: impl AsRef<str>) -> PathBuf {
    let executable = std::env::current_exe().expect("locate the F2 test executable");
    let binary_directory = executable
        .parent()
        .expect("F2 test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the F2 test binary directory");
    binary_directory
        .operation_path()
        .join("zircon-f2-fixtures")
        .join(label.as_ref())
}

struct ProductFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    stats: RenderStats,
}

struct F2Project {
    root: PathBuf,
}

impl F2Project {
    fn create() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        let root = unique_f2_fixture_root(format!(
            "basic-scene-{}_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            unique
        ));
        write_project(&root);
        Self { root }
    }

    fn assert_removable_after_sessions_drop(&self) {
        std::fs::remove_dir_all(&self.root)
            .expect("F2 project directory must be removable after runtime-session teardown");
        assert!(
            !self.root.exists(),
            "F2 project directory must not retain runtime-owned file handles after teardown"
        );
    }
}

impl Drop for F2Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn write_project(root: &Path) {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "F2BasicScene")
        .expect("render F2 product template");
    for entry in rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create F2 template directory");
        }
        std::fs::write(destination, entry.bytes).expect("write F2 template entry");
    }

    let paths = ProjectPaths::from_root(root).expect("F2 project paths");
    paths
        .ensure_derived_layout()
        .expect("F2 project derived layout");
}
