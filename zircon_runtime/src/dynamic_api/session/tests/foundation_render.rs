use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{project_asset_manager_handle, AssetUri, ProjectPaths};
use crate::core::framework::input::{InputButton, InputEvent};
use crate::core::framework::render::RenderStats;
use crate::core::manager::resolve_manager_service;
use crate::core::resource::ResourceState;
use crate::runtime_diagnostics::collect_runtime_diagnostics;
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
// The persisted one-unit cube face projects to about 11 x 11 pixels here; require over half.
const MIN_VISIBLE_PRIMITIVE_PIXELS: usize = 64;

#[path = "foundation_render/f2_evidence.rs"]
mod f2_evidence;
#[path = "foundation_render/ui_input_frame_profile.rs"]
mod ui_input_frame_profile;
use self::f2_evidence::{
    assert_f2_capture_png, write_f2_capture_png, FrameEvidence, FramePixelCounts, FrameSlot,
    ProductFrame,
};

#[test]
fn render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down() {
    let project = F2Project::create();
    let config = RuntimeProjectConfig::from_root(project.root.clone())
        .expect("F2 project root should resolve before session creation");
    let evidence = FrameEvidence::from_current_environment()
        .expect("initialize F2 managed product evidence capture");
    let reference_frames = capture_cube_free_reference_frames(&project.cube_free_root, 2);

    let (first, second_gpu_upload_bytes) = {
        let mut session =
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Runtime, Some(config.clone()))
                .expect("F2 runtime session should load the persisted project");
        assert_template_assets_ready(&session);
        assert_input_ingress(&mut session);
        session.tick_frame().expect("F2 runtime tick");

        let first = capture_product_frame(&mut session);
        let first_counts = assert_basic_scene_frame(&first, &reference_frames[0], "first launch");
        assert_product_diagnostics(&session);
        evidence.record_frame(&first, FrameSlot::FirstLaunch, first_counts);

        let second = capture_product_frame(&mut session);
        let second_counts =
            assert_basic_scene_frame(&second, &reference_frames[1], "unchanged second frame");
        evidence.record_frame(&second, FrameSlot::UnchangedSecondFrame, second_counts);
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
    let restarted_counts = assert_basic_scene_frame(
        &restarted,
        &reference_frames[0],
        "second launch after teardown",
    );
    evidence.record_frame(
        &restarted,
        FrameSlot::SecondLaunchAfterTeardown,
        restarted_counts,
    );
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
fn identical_spatially_varying_background_has_no_visible_primitive_pixels() {
    let background = [
        4, 8, 12, 255, 32, 48, 64, 255, 96, 80, 64, 255, 160, 176, 192, 255,
    ];
    assert_ne!(&background[0..4], &background[4..8]);
    assert_ne!(&background[4..8], &background[8..12]);
    assert_eq!(
        count_rgba_pixels_different_from_reference(&background, &background),
        0,
        "a spatially varying but identical background must contribute zero primitive pixels"
    );

    let mut with_primitive = background;
    with_primitive[4..8].copy_from_slice(&[220, 180, 140, 255]);
    assert_eq!(
        count_rgba_pixels_different_from_reference(&with_primitive, &background),
        1,
        "the reference comparison must count a pixel changed by a visible primitive"
    );
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

fn assert_basic_scene_frame(
    frame: &ProductFrame,
    cube_free_reference: &ProductFrame,
    label: &str,
) -> FramePixelCounts {
    assert_eq!((frame.width, frame.height), (CAPTURE_WIDTH, CAPTURE_HEIGHT));
    assert_eq!(
        (cube_free_reference.width, cube_free_reference.height),
        (frame.width, frame.height),
        "{label} and its cube-free reference must use the same viewport"
    );
    assert_eq!(
        frame.rgba.len(),
        (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize,
        "{label} must return a complete RGBA frame"
    );
    assert_eq!(
        cube_free_reference.rgba.len(),
        frame.rgba.len(),
        "{label} and its cube-free reference must return complete RGBA frames"
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
    assert_eq!(
        cube_free_reference.stats.last_mesh_draw_count, 0,
        "{label} reference scene must preserve Camera and Sun while omitting the Cube mesh"
    );
    assert!(
        cube_free_reference.stats.last_graph_executed_pass_count > 0,
        "{label} cube-free reference must execute the RenderGraph"
    );
    assert!(
        cube_free_reference.stats.last_directional_light_count > 0,
        "{label} cube-free reference must preserve the persisted directional light"
    );
    let visible_primitive_pixels =
        count_rgba_pixels_different_from_reference(&frame.rgba, &cube_free_reference.rgba);
    assert!(
        visible_primitive_pixels >= MIN_VISIBLE_PRIMITIVE_PIXELS,
        "{label} must show the persisted Cube changing at least {MIN_VISIBLE_PRIMITIVE_PIXELS} pixels relative to the same-scene cube-free capture, visible_primitive_pixels={visible_primitive_pixels}"
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

    FramePixelCounts {
        primitive_pixels: visible_primitive_pixels,
        non_transparent_pixels,
    }
}

fn capture_cube_free_reference_frames(
    project_root: &Path,
    frame_count: usize,
) -> Vec<ProductFrame> {
    assert!(frame_count > 0, "F2 reference capture must include a frame");
    let config = RuntimeProjectConfig::from_root(project_root.to_path_buf())
        .expect("F2 cube-free reference project root should resolve");
    let mut session =
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Runtime, Some(config))
            .expect("F2 runtime session should load the persisted cube-free reference scene");
    session
        .tick_frame()
        .expect("F2 cube-free reference runtime tick");
    assert_product_diagnostics(&session);

    (0..frame_count)
        .map(|_| {
            let frame = capture_product_frame(&mut session);
            assert_eq!(
                frame.stats.last_mesh_draw_count, 0,
                "F2 cube-free reference must not submit a mesh"
            );
            assert!(
                frame.stats.last_directional_light_count > 0,
                "F2 cube-free reference must retain the persisted directional light"
            );
            frame
        })
        .collect()
}

fn count_rgba_pixels_different_from_reference(actual: &[u8], reference: &[u8]) -> usize {
    assert_eq!(actual.len(), reference.len());
    assert_eq!(actual.len() % 4, 0, "RGBA reference must have whole pixels");
    actual
        .chunks_exact(4)
        .zip(reference.chunks_exact(4))
        .filter(|(actual, reference)| *actual != *reference)
        .count()
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

struct F2Project {
    root: PathBuf,
    cube_free_root: PathBuf,
}

impl F2Project {
    fn create() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        let unique = format!(
            "basic-scene-{}_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            unique
        );
        let root = unique_f2_fixture_root(&unique);
        let cube_free_root = unique_f2_fixture_root(format!("{unique}-cube-free"));
        let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "F2BasicScene")
            .expect("render F2 product template");
        write_project(&root, &rendered, true);
        write_project(&cube_free_root, &rendered, false);
        Self {
            root,
            cube_free_root,
        }
    }

    fn assert_removable_after_sessions_drop(&self) {
        for root in [&self.root, &self.cube_free_root] {
            std::fs::remove_dir_all(root)
                .expect("F2 project directory must be removable after runtime-session teardown");
            assert!(
                !root.exists(),
                "F2 project directory must not retain runtime-owned file handles after teardown"
            );
        }
    }
}

impl Drop for F2Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.cube_free_root);
    }
}

fn write_project(
    root: &Path,
    rendered: &zircon_runtime_interface::project::RenderedProjectTemplate,
    include_cube: bool,
) {
    for entry in &rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create F2 template directory");
        }
        let bytes = if !include_cube && entry.path.as_str() == "assets/scenes/main.scene.toml" {
            cube_free_scene_bytes(&entry.bytes)
        } else {
            entry.bytes.clone()
        };
        std::fs::write(destination, bytes).expect("write F2 template entry");
    }

    let paths = ProjectPaths::from_root(root).expect("F2 project paths");
    paths
        .ensure_derived_layout()
        .expect("F2 project derived layout");
}

fn cube_free_scene_bytes(scene_bytes: &[u8]) -> Vec<u8> {
    let source = std::str::from_utf8(scene_bytes).expect("F2 template scene must be UTF-8");
    let mut scene = toml::from_str::<toml::Value>(source).expect("F2 template scene must be TOML");
    let entities = scene
        .get_mut("entities")
        .and_then(toml::Value::as_array_mut)
        .expect("F2 template scene must contain entities");
    let original_entity_count = entities.len();
    let camera = entities
        .iter()
        .find(|entity| entity.get("name").and_then(toml::Value::as_str) == Some("Camera"))
        .expect("F2 template scene must contain the persisted Camera")
        .clone();
    let sun = entities
        .iter()
        .find(|entity| entity.get("name").and_then(toml::Value::as_str) == Some("Sun"))
        .expect("F2 template scene must contain the persisted Sun")
        .clone();
    entities.retain(|entity| entity.get("name").and_then(toml::Value::as_str) != Some("Cube"));
    assert_eq!(
        entities.len() + 1,
        original_entity_count,
        "F2 cube-free scene variant must remove exactly the persisted Cube"
    );
    assert_eq!(
        entities
            .iter()
            .find(|entity| entity.get("name").and_then(toml::Value::as_str) == Some("Camera")),
        Some(&camera),
        "F2 cube-free reference must preserve the complete persisted Camera entity"
    );
    assert_eq!(
        entities
            .iter()
            .find(|entity| entity.get("name").and_then(toml::Value::as_str) == Some("Sun")),
        Some(&sun),
        "F2 cube-free reference must preserve the complete persisted Sun entity"
    );
    toml::to_string_pretty(&scene)
        .expect("encode F2 cube-free reference scene")
        .into_bytes()
}
