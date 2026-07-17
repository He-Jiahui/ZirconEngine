use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::test_support::{
    write_checker_png, write_default_material, write_static_lit_default_scene, write_triangle_obj,
};
use crate::asset::{AssetUri, ProjectManifest, ProjectPaths};
use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::framework::input::InputEvent;
use crate::core::framework::render::RenderStats;
use image::ImageFormat;
use zircon_runtime_interface::project::RelPath;
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
const GPU_SCENE_WGSL: &str =
    include_str!("../../../graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

#[test]
fn render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down() {
    let project = F2Project::create();
    let config = RuntimeProjectConfig::from_root(project.root.clone());

    let (first, second_gpu_upload_bytes) = {
        let mut session =
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Runtime, Some(config.clone()))
                .expect("F2 runtime session should load the persisted project");
        assert_input_ingress(&mut session);
        session.tick_frame().expect("F2 runtime tick");

        let first = capture_product_frame(&mut session);
        assert_basic_scene_frame(&first, "first launch");
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

fn assert_input_ingress(session: &mut RuntimeDynamicSession) {
    let viewport = ZrRuntimeViewportHandle::new(1);
    let size = ZrRuntimeViewportSizeV1::new(CAPTURE_WIDTH, CAPTURE_HEIGHT);
    let pointer = [CAPTURE_WIDTH as f32 * 0.5, CAPTURE_HEIGHT as f32 * 0.5];
    let events = [
        ZrRuntimeEventV1::viewport_resized(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
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
}

fn capture_product_frame(session: &mut RuntimeDynamicSession) -> ProductFrame {
    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(CAPTURE_WIDTH, CAPTURE_HEIGHT),
        ))
        .expect("F2 WGPU frame capture");
    let rgba = if frame.rgba.data.is_null() || frame.rgba.len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(frame.rgba.data.cast_const(), frame.rgba.len) }.to_vec()
    };
    let free = frame.rgba.free.expect("F2 frame must own a free callback");
    let free_status = unsafe { free(frame.rgba) };
    assert!(
        free_status.is_ok(),
        "F2 frame buffer must release cleanly: {free_status:?}"
    );
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
    let primitive_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[1] > 170 && pixel[0] < 120 && pixel[2] < 150 && pixel[3] == 255)
        .count();
    assert!(
        primitive_pixels > 100,
        "{label} must contain the green persisted primitive, primitive_pixels={primitive_pixels}"
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
}

fn assert_steady_state_performance(first: &RenderStats, second: &RenderStats) {
    assert!(first.last_graph_compiled_cache_miss_count > 0);
    assert_eq!(
        second.last_graph_compiled_cache_miss_count, first.last_graph_compiled_cache_miss_count,
        "an unchanged F2 frame must not recompile the RenderGraph"
    );
    assert_eq!(
        second.last_graph_compiled_cache_hit_count,
        first.last_graph_compiled_cache_hit_count + 1,
        "an unchanged F2 frame must reuse the compiled RenderGraph"
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
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("create F2 capture output directory");
    }
    let image = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("F2 capture buffer must match dimensions");
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("write F2 product capture PNG");
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
        let root = std::env::temp_dir().join(format!(
            "zircon_f2_basic_scene_{}_{}_{}",
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
    let paths = ProjectPaths::from_root(root).expect("F2 project paths");
    let asset_root = paths.asset_root(&RelPath::project_assets());
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .expect("F2 project layout");
    ProjectManifest::new(
        "F2BasicScene",
        AssetUri::parse("res://scenes/main.scene.toml").expect("F2 scene URI"),
        1,
    )
    .save(paths.manifest_path())
    .expect("F2 project manifest");

    write_f2_mesh_wgsl(asset_root.join("shaders/pbr.wgsl"));
    write_checker_png(asset_root.join("textures/checker.png"));
    write_triangle_obj(asset_root.join("models/triangle.obj"));
    write_default_material(asset_root.join("materials/grid.zmaterial"));
    write_static_lit_default_scene(asset_root.join("scenes/main.scene.toml"));
}

fn write_f2_mesh_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create F2 shader directory");
    }
    let shader = format!(
        r#"{GPU_SCENE_WGSL}

struct SceneUniform {{
    view_proj: mat4x4<f32>,
}};

struct MaterialPropertyUniform {{
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
}};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;
@group(2) @binding(1) var albedo_tex: texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
}};

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {{
    var output: VertexOutput;
    let world = zr_world_from_local(instance_index) * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    output.tint = zr_gpu_scene_tint(instance_index);
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>(0.05, 0.9, 0.2, alpha) * input.tint;
}}
"#
    );
    std::fs::write(path, shader).expect("write F2 mesh shader");
}
