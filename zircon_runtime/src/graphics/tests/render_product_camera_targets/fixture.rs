use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::mesh::build_snapshot;
use crate::asset::assets::{AlphaMode, MaterialAsset, ShaderAsset, ShaderSourceLanguage};
use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetReference, AssetUri, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT, TextureAsset,
    TextureAssetDescriptor,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderImageColorSpace, RenderImageFallbackKind,
    RenderImageUsage, RenderMeshSnapshot, RenderPipelineHandle, RenderQualityProfile,
    RenderSamplerDescriptor, RenderViewportDescriptor, RenderViewportHandle,
    RenderWorldSnapshotHandle, ShaderAssetKind,
};
use crate::core::math::UVec2;
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;
use image::{ImageBuffer, ImageFormat, Rgba};

const GPU_SCENE_TEST_WGSL: &str =
    include_str!("../../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

pub(super) struct RenderFixture {
    root: PathBuf,
    asset_manager: Arc<ProjectAssetManager>,
    pub(super) model: ResourceHandle<ModelMarker>,
    pub(super) material: ResourceHandle<MaterialMarker>,
    pub(super) viewport_size: UVec2,
}

impl RenderFixture {
    pub(super) fn new(label: &str, base_color: [f32; 4]) -> Self {
        let root = unique_temp_project_root(label);
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            label,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        write_flat_color_wgsl(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("shaders")
                .join("flat_color.wgsl"),
            [base_color[0], base_color[1], base_color[2]],
        );
        write_solid_png(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("textures")
                .join("white.png"),
            [255, 255, 255, 255],
        );
        write_quad_obj(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("models")
                .join("quad.obj"),
        );
        write_material_with_base_color_and_texture(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("materials")
                .join("flat_color.zmaterial"),
            "res://shaders/flat_color.wgsl",
            base_color,
            "res://textures/white.png",
            AlphaMode::Opaque,
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
        insert_sample_texture_shader(&asset_manager);

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

    pub(super) fn configured_framework(
        &self,
        profile: RenderQualityProfile,
    ) -> (WgpuRenderFramework, RenderViewportHandle) {
        let framework = WgpuRenderFramework::new_for_test(self.asset_manager.clone()).unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(self.viewport_size))
            .unwrap();
        framework
            .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
            .unwrap();
        framework.set_quality_profile(viewport, profile).unwrap();
        (framework, viewport)
    }

    pub(super) fn frame_extract(&self, meshes: Vec<RenderMeshSnapshot>) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            build_snapshot(meshes, self.viewport_size),
        )
    }

    pub(super) fn insert_srgb_render_target_texture(&self, uri: &str, size: UVec2) -> ResourceId {
        let texture_uri = AssetUri::parse(uri).unwrap();
        let texture_id = ResourceId::from_locator(&texture_uri);
        self.asset_manager
            .assets::<TextureAsset>()
            .insert(
                ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
                TextureAsset::new_rgba8(
                    texture_uri,
                    size.x,
                    size.y,
                    vec![0; (size.x * size.y * 4) as usize],
                )
                .with_descriptor(srgb_render_target_texture_descriptor()),
            )
            .expect("texture insert");
        texture_id
    }

    pub(super) fn insert_linear_render_target_texture(&self, uri: &str, size: UVec2) -> ResourceId {
        let texture_uri = AssetUri::parse(uri).unwrap();
        let texture_id = ResourceId::from_locator(&texture_uri);
        self.asset_manager
            .assets::<TextureAsset>()
            .insert(
                ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
                TextureAsset::new_rgba8(
                    texture_uri,
                    size.x,
                    size.y,
                    vec![0; (size.x * size.y * 4) as usize],
                )
                .with_descriptor(render_target_texture_descriptor()),
            )
            .expect("linear texture insert");
        texture_id
    }

    pub(super) fn insert_texture_sampling_material(
        &self,
        material_uri: &str,
        base_color_texture_uri: &str,
    ) -> ResourceHandle<MaterialMarker> {
        let material_uri = AssetUri::parse(material_uri).unwrap();
        let material_id = ResourceId::from_locator(&material_uri);
        self.asset_manager
            .assets::<MaterialAsset>()
            .insert(
                ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
                MaterialAsset {
                    name: Some("SampleOutputTarget".to_string()),
                    shader: asset_reference("res://shaders/sample_texture.wgsl"),
                    parent: None,
                    options: Default::default(),
                    queue: None,
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
}

impl Drop for RenderFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn insert_sample_texture_shader(asset_manager: &ProjectAssetManager) {
    let shader_uri = AssetUri::parse("res://shaders/sample_texture.wgsl").unwrap();
    let shader_id = ResourceId::from_locator(&shader_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone()),
            ShaderAsset {
                uri: shader_uri,
                kind: ShaderAssetKind::Surface,
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
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: None,
                render_state: Default::default(),
                queue: None,
                disabled_passes: Vec::new(),
                resources: Vec::new(),
                material_property_layout: Default::default(),
                material_option_table: Default::default(),
                generated_material_wgsl: String::new(),
                editor: Default::default(),
                pipeline_layout: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("sample texture shader insert");
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
    return textureSample(albedo_tex, albedo_sampler, input.uv) * input.tint;
}
"#
    )
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
        parent: None,
        options: Default::default(),
        queue: None,
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

fn render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![
            RenderImageUsage::RenderTarget,
            RenderImageUsage::Sampled,
            RenderImageUsage::CopySrc,
        ],
        fallback: RenderImageFallbackKind::MissingImage,
        ..TextureAssetDescriptor::default()
    }
}

fn srgb_render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Srgb,
        ..render_target_texture_descriptor()
    }
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

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let process_id = std::process::id();
    let sequence = NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_camera_targets_{label}_{process_id}_{sequence}_{unique}"
    ))
}
