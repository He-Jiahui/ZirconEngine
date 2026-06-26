use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{AlphaMode, MaterialAsset, ShaderAsset, ShaderSourceLanguage};
use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderCameraClear,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderOverlayExtract, RenderPipelineHandle, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderViewportHandle, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::scene::components::{default_render_layer_mask, Mobility};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::graphics::RenderFeatureDescriptor;
use crate::graphics::{
    offline_bake_frame, OfflineBakeSettings, RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::plugin_render_feature_fixtures::default_rendering_feature_descriptors;

const GPU_SCENE_TEST_WGSL: &str =
    include_str!("../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

mod particles;
mod postprocess;
mod queue_override;
mod transparent3d;

#[test]
fn offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering() {
    let fixture = RenderFixture::new("graphics_m4_offline_bake", [0.5, 0.5, 0.5, 1.0]);
    let base_extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: RenderLayerSet::from_legacy_mask(default_render_layer_mask()),
        }],
        vec![RenderDirectionalLightSnapshot {
            node_id: 7,
            light_id: 7,
            layer_mask: RenderLayerSet::from_legacy_mask(default_render_layer_mask()),
            direction: Vec3::new(-0.4, -0.4, -1.0).normalize_or_zero(),
            color: Vec3::new(1.0, 0.62, 0.28),
            intensity: 3.2,
            shadow: None,
        }],
        |_extract| {},
    );

    let bake_output = offline_bake_frame(
        &base_extract,
        &OfflineBakeSettings {
            ambient_scale: 0.24,
            reflection_probe_scale: 0.8,
            max_reflection_probes: 1,
        },
    );
    assert!(
        bake_output.baked_lighting.intensity > 0.0,
        "offline bake should produce non-zero baked lighting"
    );
    assert!(
        !bake_output.reflection_probes.is_empty(),
        "offline bake should produce at least one reflection probe"
    );

    let mut baked_extract = base_extract.clone();
    baked_extract.lighting.baked_lighting = Some(bake_output.baked_lighting);
    baked_extract.lighting.reflection_probes = bake_output.reflection_probes;

    let server = fixture.server();
    let baked_frame = fixture.render_extract(
        &server,
        baked_extract,
        RenderQualityProfile::new("baked-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let unbaked_frame = fixture.render_extract(
        &server,
        base_extract,
        RenderQualityProfile::new("baked-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_baked_lighting(false)
            .with_reflection_probes(false),
    );

    let baked_red = average_channel(&baked_frame.rgba, 0);
    let unbaked_red = average_channel(&unbaked_frame.rgba, 0);
    assert!(
        baked_red > unbaked_red + 8.0,
        "expected baked lighting and probes to change the frame; baked red={baked_red:.2}, unbaked red={unbaked_red:.2}"
    );
}

struct RenderFixture {
    root: PathBuf,
    asset_manager: Arc<ProjectAssetManager>,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    viewport_size: UVec2,
}

impl RenderFixture {
    fn new(label: &str, base_color: [f32; 4]) -> Self {
        Self::new_with_alpha_mode(label, base_color, AlphaMode::Opaque)
    }

    fn new_with_alpha_mode(label: &str, base_color: [f32; 4], alpha_mode: AlphaMode) -> Self {
        let root = unique_temp_project_root(label);
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            label,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        write_flat_color_wgsl(
            paths.assets_root().join("shaders").join("flat_color.wgsl"),
            [base_color[0], base_color[1], base_color[2]],
        );
        write_solid_png(
            paths.assets_root().join("textures").join("white.png"),
            [255, 255, 255, 255],
        );
        write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
        write_material_with_base_color_and_texture(
            paths
                .assets_root()
                .join("materials")
                .join("flat_color.zmaterial"),
            "res://shaders/flat_color.wgsl",
            base_color,
            "res://textures/white.png",
            alpha_mode,
        );

        let asset_manager = Arc::new(ProjectAssetManager::default());
        asset_manager
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        asset_manager
            .open_project(root.to_string_lossy().as_ref())
            .unwrap();
        let mut project = ProjectManager::open(&root).unwrap();
        project.scan_and_import().unwrap();

        let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
        let material = resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/flat_color.zmaterial",
        );

        Self {
            root,
            asset_manager,
            model,
            material,
            viewport_size: UVec2::new(160, 120),
        }
    }

    fn server(&self) -> WgpuRenderFramework {
        WgpuRenderFramework::new_with_plugin_render_features(
            self.asset_manager.clone(),
            default_rendering_feature_descriptors(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
    }

    fn builtin_server(&self) -> WgpuRenderFramework {
        WgpuRenderFramework::new(self.asset_manager.clone()).unwrap()
    }

    fn server_with_render_features(
        &self,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> WgpuRenderFramework {
        let mut features = default_rendering_feature_descriptors();
        features.extend(render_features);
        WgpuRenderFramework::new_with_plugin_render_features(
            self.asset_manager.clone(),
            features,
            render_pass_executors,
            Vec::new(),
        )
        .unwrap()
    }

    fn frame_extract<F>(
        &self,
        meshes: Vec<RenderMeshSnapshot>,
        lights: Vec<RenderDirectionalLightSnapshot>,
        configure: F,
    ) -> RenderFrameExtract
    where
        F: FnOnce(&mut RenderFrameExtract),
    {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            build_snapshot(meshes, lights, self.viewport_size),
        );
        configure(&mut extract);
        extract
    }

    fn insert_texture_sampling_material(
        &self,
        material_uri: &str,
        base_color_texture_uri: &str,
    ) -> ResourceHandle<MaterialMarker> {
        let shader_uri = AssetUri::parse("res://shaders/sample_texture.wgsl").unwrap();
        let shader_id = ResourceId::from_locator(&shader_uri);
        self.asset_manager
            .assets::<ShaderAsset>()
            .insert(
                ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone()),
                sample_texture_shader(shader_uri),
            )
            .expect("sample texture shader insert");

        let material_uri = AssetUri::parse(material_uri).unwrap();
        let material_id = ResourceId::from_locator(&material_uri);
        self.asset_manager
            .assets::<MaterialAsset>()
            .insert(
                ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
                MaterialAsset {
                    name: Some("SampleOutputTarget".to_string()),
                    shader: asset_reference("res://shaders/sample_texture.wgsl"),
                    base_color: [1.0, 1.0, 1.0, 1.0],
                    base_color_texture: Some(asset_reference(base_color_texture_uri)),
                    normal_texture: None,
                    metallic: 0.0,
                    roughness: 1.0,
                    metallic_roughness_texture: None,
                    occlusion_texture: None,
                    emissive: [0.0, 0.0, 0.0],
                    emissive_texture: None,
                    alpha_mode: AlphaMode::Opaque,
                    double_sided: false,
                    property_values: Default::default(),
                    texture_slots: Default::default(),
                    validation_diagnostics: Vec::new(),
                },
            )
            .expect("sample texture material insert");
        ResourceHandle::new(material_id)
    }

    fn render_extract(
        &self,
        server: &WgpuRenderFramework,
        extract: RenderFrameExtract,
        profile: RenderQualityProfile,
    ) -> crate::core::framework::render::CapturedFrame {
        let viewport = server
            .create_viewport(RenderViewportDescriptor::new(self.viewport_size))
            .unwrap();
        server
            .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
            .unwrap();
        server.set_quality_profile(viewport, profile).unwrap();
        let frame = submit_extract(server, viewport, extract);
        server.destroy_viewport(viewport).unwrap();
        frame
    }
}

impl Drop for RenderFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
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
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
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

fn centered_quad_transform(scale: f32) -> Transform {
    Transform {
        scale: Vec3::new(scale, scale, 1.0),
        ..Transform::default()
    }
}

fn submit_extract(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> crate::core::framework::render::CapturedFrame {
    server.submit_frame_extract(viewport, extract).unwrap();
    server
        .capture_frame(viewport)
        .unwrap()
        .expect("frame should be available after submission")
}

fn ring_luma(rgba: &[u8], viewport_size: UVec2, inner_radius: f32, outer_radius: f32) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    let center_x = viewport_size.x as f32 * 0.5;
    let center_y = viewport_size.y as f32 * 0.5;
    let normalizer = viewport_size.x.min(viewport_size.y) as f32 * 0.5;
    for y in 0..viewport_size.y as usize {
        for x in 0..viewport_size.x as usize {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            let radius = (dx * dx + dy * dy).sqrt() / normalizer.max(1.0);
            if radius < inner_radius || radius > outer_radius {
                continue;
            }
            let index = (y * viewport_size.x as usize + x) * 4;
            let pixel = &rgba[index..index + 4];
            total += 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn dominant_red_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[0] > 72 && pixel[0] > pixel[1] + 32)
        .count()
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[1] > 72 && pixel[1] > pixel[0] + 32)
        .count()
}

fn dominant_blue_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[2] > 72 && pixel[2] > pixel[0] + 32)
        .count()
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
    frame: &crate::core::framework::render::CapturedFrame,
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
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

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

fn sample_texture_shader(uri: AssetUri) -> ShaderAsset {
    ShaderAsset {
        uri,
        source_language: ShaderSourceLanguage::Wgsl,
        source: sample_texture_wgsl_source(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn sample_texture_wgsl_source() -> String {
    format!(
        "{GPU_SCENE_TEST_WGSL}\n{}",
        r#"
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
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

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
    return textureSample(albedo_tex, albedo_sampler, input.uv) * input.tint;
}
"#
    )
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
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

fn write_material_with_base_color_and_texture(
    path: PathBuf,
    shader_uri: &str,
    base_color: [f32; 4],
    base_color_texture: &str,
    alpha_mode: AlphaMode,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("FlatColor".to_string()),
        shader: asset_reference(shader_uri),
        base_color,
        base_color_texture: Some(asset_reference(base_color_texture)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
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
