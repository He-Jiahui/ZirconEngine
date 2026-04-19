use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_asset::assets::{
    AlphaMode, MaterialAsset, SceneAsset, SceneCameraAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_asset::{AssetReference, AssetUri};
use zircon_framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderMeshSnapshot,
    RenderOverlayExtract, RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
};
use zircon_math::{Transform, UVec2, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::components::{default_render_layer_mask, Mobility};
use zircon_scene::world::World;

use crate::{runtime::WgpuRenderFramework, SceneRenderer};

#[test]
fn directory_project_scene_renders_non_background_frame_with_gizmo_overlay() {
    let root = unique_temp_project_root("graphics_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
        "res://shaders/pbr.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/grid.material.toml",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(320, 240)),
    });

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(320, 240)).unwrap();

    let background = [20_u8, 23_u8, 28_u8, 255_u8];
    assert!(frame.rgba.chunks_exact(4).any(|pixel| pixel != background));
    assert!(frame
        .rgba
        .chunks_exact(4)
        .any(|pixel| { pixel[3] == 255 && (pixel[0] > 200 || pixel[1] > 200 || pixel[2] > 200) }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn directory_project_material_shader_drives_pipeline_color_output() {
    let root = unique_temp_project_root("graphics_shader_pipeline");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsShaderPipeline",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(paths.assets_root().join("shaders").join("flat_green.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/flat_green.material.toml",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut snapshot = world.to_render_snapshot();
    snapshot.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer.render(snapshot, UVec2::new(320, 240)).unwrap();

    let green_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255
                && pixel[1] > 160
                && pixel[1] > pixel[0] + 50
                && pixel[1] > pixel[2] + 30
        })
        .count();
    assert!(
        green_pixels > 32,
        "expected project shader to dominate visible pixels, found {green_pixels}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn wire_only_mode_reduces_filled_surface_pixels() {
    let root = unique_temp_project_root("graphics_wire_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsWireOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(paths.assets_root().join("shaders").join("flat_green.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
        "res://shaders/flat_green.wgsl",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/grid.material.toml",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

    let mut shaded = world.to_render_snapshot();
    shaded.preview.skybox_enabled = false;
    shaded.preview.fallback_skybox = FallbackSkyboxKind::None;
    shaded.overlays = RenderOverlayExtract {
        display_mode: DisplayMode::Shaded,
        ..RenderOverlayExtract::default()
    };

    let mut wire_only = shaded.clone();
    wire_only.overlays.display_mode = DisplayMode::WireOnly;

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let shaded_frame = renderer.render(shaded, UVec2::new(320, 240)).unwrap();
    let wire_frame = renderer.render(wire_only, UVec2::new(320, 240)).unwrap();

    let background: [u8; 4] = wire_frame.rgba[..4].try_into().unwrap();
    let shaded_surface_pixels = shaded_frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();
    let wire_surface_pixels = wire_frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != background)
        .count();

    assert!(
        shaded_surface_pixels > 0 && wire_surface_pixels < shaded_surface_pixels,
        "wire-only mode should suppress most filled surface pixels ({wire_surface_pixels} vs {shaded_surface_pixels})"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn history_resolve_blends_previous_scene_color_when_enabled() {
    let root = unique_temp_project_root("graphics_history_resolve");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsHistoryResolve",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.02, 0.92, 0.1],
    );
    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_black.wgsl"),
        [0.0, 0.0, 0.0],
    );
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
        "res://shaders/flat_green.wgsl",
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_black.material.toml"),
        "res://shaders/flat_black.wgsl",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let black_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_black.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let history_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            history_viewport,
            RenderQualityProfile::new("history-only")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    submit_snapshot(
        &server,
        history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                transform: fullscreen_quad_transform(),
                model,
                material: green_material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
            Vec::new(),
            viewport_size,
        ),
    );
    let _ = server.capture_frame(history_viewport).unwrap();
    let history_frame = submit_snapshot(
        &server,
        history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                transform: fullscreen_quad_transform(),
                model,
                material: black_material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
            Vec::new(),
            viewport_size,
        ),
    );

    let no_history_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            no_history_viewport,
            RenderQualityProfile::new("no-history")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let no_history_frame = submit_snapshot(
        &server,
        no_history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                transform: fullscreen_quad_transform(),
                model,
                material: black_material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
            Vec::new(),
            viewport_size,
        ),
    );

    let history_green_pixels = dominant_green_pixels(&history_frame.rgba);
    let no_history_green_pixels = dominant_green_pixels(&no_history_frame.rgba);
    assert!(
        history_green_pixels > no_history_green_pixels + 64,
        "expected history resolve to preserve prior frame color; green pixels with history={history_green_pixels}, without history={no_history_green_pixels}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn ssao_quality_profile_darkens_scene_when_enabled() {
    let root = unique_temp_project_root("graphics_ssao");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsSsao",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_gray.wgsl"),
        [0.72, 0.72, 0.72],
    );
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_gray.material.toml"),
        "res://shaders/flat_gray.wgsl",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_gray.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let snapshot = build_snapshot(
        vec![
            RenderMeshSnapshot {
                node_id: 1,
                transform: fullscreen_quad_transform(),
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            },
            RenderMeshSnapshot {
                node_id: 2,
                transform: offset_quad_transform(),
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            },
        ],
        Vec::new(),
        viewport_size,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let ao_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            ao_viewport,
            RenderQualityProfile::new("ao-on")
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let ao_frame = submit_snapshot(&server, ao_viewport, snapshot.clone());

    let no_ao_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            no_ao_viewport,
            RenderQualityProfile::new("ao-off")
                .with_clustered_lighting(false)
                .with_history_resolve(false)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();
    let no_ao_frame = submit_snapshot(&server, no_ao_viewport, snapshot);

    let ao_luma = average_luma(&ao_frame.rgba);
    let no_ao_luma = average_luma(&no_ao_frame.rgba);
    assert!(
        ao_luma + 5.0 < no_ao_luma,
        "expected SSAO-enabled output to be darker; ao luma={ao_luma:.2}, no-ao luma={no_ao_luma:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn clustered_lighting_quality_profile_applies_runtime_tile_lighting() {
    let root = unique_temp_project_root("graphics_clustered_lighting");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsClusteredLighting",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_white.wgsl"),
        [0.55, 0.55, 0.55],
    );
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_white.material.toml"),
        "res://shaders/flat_white.wgsl",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let lights = vec![RenderDirectionalLightSnapshot {
        node_id: 7,
        direction: Vec3::new(-0.65, -0.35, -1.0).normalize_or_zero(),
        color: Vec3::new(1.0, 0.48, 0.2),
        intensity: 3.5,
    }];
    let snapshot = build_snapshot(
        vec![RenderMeshSnapshot {
            node_id: 1,
            transform: fullscreen_quad_transform(),
            model,
            material,
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        }],
        lights,
        viewport_size,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let clustered_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            clustered_viewport,
            RenderQualityProfile::new("clustered-on")
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let clustered_frame = submit_snapshot(&server, clustered_viewport, snapshot.clone());

    let flat_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            flat_viewport,
            RenderQualityProfile::new("clustered-off")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let flat_frame = submit_snapshot(&server, flat_viewport, snapshot);

    let clustered_red = average_channel(&clustered_frame.rgba, 0);
    let flat_red = average_channel(&flat_frame.rgba, 0);
    assert!(
        clustered_red > flat_red + 5.0,
        "expected clustered lighting to tint the frame; clustered red={clustered_red:.2}, flat red={flat_red:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path() {
    let root = unique_temp_project_root("graphics_deferred_runtime");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsDeferredRuntime",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_green_wgsl(paths.assets_root().join("shaders").join("flat_green.wgsl"));
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material_with_base_color_and_texture(
        paths
            .assets_root()
            .join("materials")
            .join("forward_green.material.toml"),
        "res://shaders/flat_green.wgsl",
        [1.0, 0.08, 0.08, 1.0],
        "res://textures/white.png",
    );
    write_scene(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/forward_green.material.toml",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/forward_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let snapshot = build_snapshot(
        vec![RenderMeshSnapshot {
            node_id: 1,
            transform: fullscreen_quad_transform(),
            model,
            material,
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        viewport_size,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let forward_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(forward_viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            forward_viewport,
            RenderQualityProfile::new("forward-clean")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let forward_frame = submit_snapshot(&server, forward_viewport, snapshot.clone());

    let deferred_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(deferred_viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .set_quality_profile(
            deferred_viewport,
            RenderQualityProfile::new("deferred-clean")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();
    let deferred_frame = submit_snapshot(&server, deferred_viewport, snapshot);

    let forward_red = average_channel(&forward_frame.rgba, 0);
    let forward_green = average_channel(&forward_frame.rgba, 1);
    let deferred_red = average_channel(&deferred_frame.rgba, 0);
    let deferred_green = average_channel(&deferred_frame.rgba, 1);

    assert!(
        forward_green > forward_red + 25.0,
        "forward baseline should remain project-shader green; red={forward_red:.2}, green={forward_green:.2}"
    );
    assert!(
        deferred_red > deferred_green + 20.0,
        "deferred runtime should shade through GBuffer material decode instead of the project shader; red={deferred_red:.2}, green={deferred_green:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn write_valid_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#,
    )
    .unwrap();
}

fn write_flat_green_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>(0.05, 0.9, 0.2, alpha);
}
"#,
    )
    .unwrap();
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            r#"
struct SceneUniform {{
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
}};
struct ModelUniform {{
    model: mat4x4<f32>,
    tint: vec4<f32>,
}};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {{
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * model_data.tint;
}}
"#,
            color[0], color[1], color[2]
        ),
    )
    .unwrap();
}

fn write_checker_png(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
    .save_with_format(path, ImageFormat::Png)
    .unwrap();
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_triangle_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 1.0 1.0 0.0
v -1.0 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vt 1.0 0.0
vt 0.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
",
    )
    .unwrap();
}

fn write_material(path: PathBuf, shader_uri: &str) {
    write_material_with_base_color_and_texture(
        path,
        shader_uri,
        [0.8, 0.8, 0.8, 1.0],
        "res://textures/checker.png",
    );
}

fn write_material_with_base_color_and_texture(
    path: PathBuf,
    shader_uri: &str,
    base_color: [f32; 4],
    base_color_texture: &str,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference(shader_uri),
        base_color,
        base_color_texture: Some(asset_reference(base_color_texture)),
        normal_texture: None,
        metallic: 0.1,
        roughness: 0.8,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_scene(path: PathBuf, material_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "Camera".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 2.0, 5.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                }),
                mesh: None,
                directional_light: None,
            },
            SceneEntityAsset {
                entity: 2,
                name: "Triangle".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    material: asset_reference(material_uri),
                }),
                directional_light: None,
            },
        ],
    };
    fs::write(path, scene.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}

fn build_snapshot(
    meshes: Vec<RenderMeshSnapshot>,
    lights: Vec<RenderDirectionalLightSnapshot>,
    viewport_size: UVec2,
) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes,
            lights,
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
    }
}

fn fullscreen_quad_transform() -> Transform {
    Transform {
        scale: Vec3::new(1.8, 1.8, 1.0),
        ..Transform::default()
    }
}

fn offset_quad_transform() -> Transform {
    Transform {
        translation: Vec3::new(0.18, -0.14, 0.32),
        scale: Vec3::new(1.1, 1.1, 1.0),
        ..Transform::default()
    }
}

fn submit_snapshot(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    snapshot: RenderSceneSnapshot,
) -> zircon_framework::render::CapturedFrame {
    server
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot),
        )
        .unwrap();
    server
        .capture_frame(viewport)
        .unwrap()
        .expect("frame should be available after submission")
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255 && pixel[1] > 20 && pixel[1] > pixel[0] + 8 && pixel[1] > pixel[2] + 8
        })
        .count()
}

fn average_luma(rgba: &[u8]) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }
    let total = rgba
        .chunks_exact(4)
        .map(|pixel| 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }
    let total = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}
