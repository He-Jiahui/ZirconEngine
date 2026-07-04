use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{
    AlphaMode, MaterialAsset, SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{
    AssetMetaDocument, AssetSourceUnit, ProjectManager, ProjectManifest, ProjectPaths,
};
use crate::asset::{AssetKind, AssetReference, AssetUri, AssetUuid};
use crate::core::framework::render::{
    CapturedFrame, DisplayMode, EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract,
    ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportHandle, RenderWorldSnapshotHandle, SampledEquirectangularEnvironment,
    SceneViewportExtractRequest, ShaderAssetKind, ViewportCameraSnapshot, ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::ResourceHandle;
use crate::scene::world::World;
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::graphics::{SceneRenderer, WgpuRenderFramework};

const GPU_SCENE_TEST_WGSL: &str =
    include_str!("../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

mod project_scenes;
mod render_quality;

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

fn project_asset_manager_with_first_wave_plugin_importers() -> Arc<ProjectAssetManager> {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    asset_manager
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
    let shader_body = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
};

struct MaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;
@group(2) @binding(1) var albedo_tex: texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let world = zr_world_from_local(instance_index) * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    output.tint = zr_gpu_scene_tint(instance_index);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>(0.05, 0.9, 0.2, alpha) * input.tint;
}
"#;
    fs::write(path, format!("{GPU_SCENE_TEST_WGSL}\n{shader_body}")).unwrap();
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            "{GPU_SCENE_TEST_WGSL}\n{}",
            format!(
                r#"
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
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * input.tint;
}}
"#,
                color[0], color[1], color[2]
            )
        ),
    )
    .unwrap();
}

fn write_material_sphere_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let shader_body = r#"
const MATERIAL_SPHERE_RADIUS: f32 = 1.35;

fn material_sphere_normal(input: ZrVertexOutput) -> vec3<f32> {
    let projected = input.position_ws.xy / MATERIAL_SPHERE_RADIUS;
    let radius_sq = clamp(dot(projected, projected), 0.0, 1.0);
    return zr_normalize_or_zero(vec3<f32>(
        projected.x,
        projected.y,
        sqrt(max(1.0 - radius_sq, 0.0)),
    ));
}

fn material_sphere_color(input: ZrVertexOutput) -> vec3<f32> {
    let normal = material_sphere_normal(input);
    let light = zr_normalize_or_zero(vec3<f32>(0.35, 0.55, 0.75));
    let diffuse = max(dot(normal, light), 0.0);
    let facing = clamp(normal.z * 0.5 + 0.5, 0.0, 1.0);
    let rim = pow(1.0 - facing, 2.0);
    let tex = zr_sample_base_color(input.uv0);
    let base = zr_mat_base_color().rgb * tex.rgb * input.tint.rgb * input.color.rgb;
    return base * (0.18 + diffuse * 0.82) + vec3<f32>(0.12, 0.18, 0.32) * rim;
}

fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let tex = zr_sample_base_color(input.uv0);
    var surface = zr_surface_from_base_color(
        vec4<f32>(material_sphere_color(input), tex.a * input.tint.a * input.color.a),
    );
    surface.normal_ws = material_sphere_normal(input);
    surface.metallic = clamp(zr_mat_metallic(), 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness(), 0.04, 1.0);
    surface.occlusion = 1.0;
    surface.unlit = 1.0;
    surface.shading_model_id = 2u;
    return surface;
}
"#;
    fs::write(path, shader_body).unwrap();
}

fn write_material_sphere_zshader(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"kind = "surface"
version = 2
name = "Runtime Material Sphere"
shading_model = "standard_pbr"
wgsl_files = ["material_sphere.wgsl"]

[[properties]]
name = "base_color"
kind = "vec4"
default = [0.78, 0.30, 0.18, 1.0]

[[properties]]
name = "metallic"
kind = "float"
default = 0.1

[[properties]]
name = "roughness"
kind = "float"
default = 0.78

[[texture_slots]]
name = "base_color"
kind = "texture_2d"
default = "white"
"#,
    )
    .unwrap();
}

fn write_compound_shader_meta(paths: &ProjectPaths, shader_uri: &str, shader_name: &str) {
    let uri = AssetUri::parse(shader_uri).unwrap();
    let meta_path = paths
        .assets_root()
        .join("shaders")
        .join(format!("{shader_name}.zmeta"));
    let mut meta = AssetMetaDocument::new(AssetUuid::new(), uri, AssetKind::Shader);
    meta.unit = AssetSourceUnit::Compound;
    meta.save(meta_path).unwrap();
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

fn write_uv_sphere_obj(path: PathBuf, rings: usize, segments: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut obj = String::new();
    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let x = radius * phi.cos();
            let z = radius * phi.sin();
            writeln!(&mut obj, "v {x:.6} {y:.6} {z:.6}").unwrap();
            writeln!(
                &mut obj,
                "vt {:.6} {:.6}",
                segment as f32 / segments as f32,
                ring as f32 / rings as f32
            )
            .unwrap();
            writeln!(&mut obj, "vn {x:.6} {y:.6} {z:.6}").unwrap();
        }
    }
    for ring in 0..rings {
        for segment in 0..segments {
            let a = ring * (segments + 1) + segment + 1;
            let b = a + 1;
            let c = a + segments + 1;
            let d = c + 1;
            writeln!(&mut obj, "f {a}/{a}/{a} {c}/{c}/{c} {b}/{b}/{b}").unwrap();
            writeln!(&mut obj, "f {b}/{b}/{b} {c}/{c}/{c} {d}/{d}/{d}").unwrap();
        }
    }
    fs::write(path, obj).unwrap();
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
        parent: None,
        options: Default::default(),
        queue: None,
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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
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
                    post_process_settings: None,
                    ..SceneCameraAsset::default()
                }),
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
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
                    mesh: None,
                    material: asset_reference(material_uri),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            },
        ],
    };
    fs::write(path, scene.to_toml_string().unwrap()).unwrap();
}

fn write_material_sphere_scene(path: PathBuf, material_uri: &str) {
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
                    translation: [0.0, 0.0, 4.2],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 0.7853982,
                    z_near: 0.1,
                    z_far: 100.0,
                    post_process_settings: None,
                    ..SceneCameraAsset::default()
                }),
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            },
            SceneEntityAsset {
                entity: 2,
                name: "Material Sphere".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.35, 1.35, 1.35],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/material_sphere.obj"),
                    mesh: None,
                    material: asset_reference(material_uri),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
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
            directional_lights: lights,
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
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
) -> crate::core::framework::render::CapturedFrame {
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

fn average_channel_in_region(
    frame: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
    channel: usize,
) -> f32 {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut total = 0.0;
    let mut count = 0.0;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4 + channel;
            total += frame.rgba[index] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}
