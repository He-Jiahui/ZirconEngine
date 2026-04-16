use crate::load::mesh::generate_cube_mesh;
use crate::{
    AlphaMode, AssetId, AssetKind, AssetMetadata, AssetReference, AssetUri, ImportedAsset,
    MaterialAsset, ModelAsset, ModelPrimitiveAsset, ResourceManager, ShaderAsset, TextureAsset,
};

use super::resource_sync::register_project_resource;

pub(super) fn resource_manager_with_builtins() -> ResourceManager {
    let manager = ResourceManager::new();

    for (locator_text, asset) in builtin_resources() {
        let locator = AssetUri::parse(locator_text).expect("builtin locator");
        let kind = match &asset {
            ImportedAsset::Texture(_) => AssetKind::Texture,
            ImportedAsset::Shader(_) => AssetKind::Shader,
            ImportedAsset::Material(_) => AssetKind::Material,
            ImportedAsset::Scene(_) => AssetKind::Scene,
            ImportedAsset::Model(_) => AssetKind::Model,
            ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
            ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
            ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
        };
        let record = AssetMetadata::new(AssetId::from_locator(&locator), kind, locator);
        register_project_resource(&manager, record, asset);
    }

    manager
}

pub(super) fn builtin_resources() -> Vec<(&'static str, ImportedAsset)> {
    let mesh = generate_cube_mesh();
    let mut resources = vec![
        (
            "builtin://cube",
            ImportedAsset::Model(ModelAsset {
                uri: AssetUri::parse("builtin://cube").expect("builtin cube uri"),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: mesh.vertices.clone(),
                    indices: mesh.indices.clone(),
                }],
            }),
        ),
        (
            "builtin://missing-model",
            ImportedAsset::Model(ModelAsset {
                uri: AssetUri::parse("builtin://missing-model").expect("missing model uri"),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: mesh.vertices,
                    indices: mesh.indices,
                }],
            }),
        ),
        (
            "builtin://material/default",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Default".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0, 0.0, 0.0],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
            }),
        ),
        (
            "builtin://missing-material",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Missing".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                base_color: [1.0, 0.0, 1.0, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0, 0.0, 0.0],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
            }),
        ),
        (
            "builtin://shader/pbr.wgsl",
            ImportedAsset::Shader(ShaderAsset {
                uri: AssetUri::parse("builtin://shader/pbr.wgsl").expect("builtin shader uri"),
                source: builtin_pbr_wgsl().to_string(),
            }),
        ),
    ];
    resources.extend(editor_icon_builtin_resources());
    resources
}

fn editor_icon_builtin_resources() -> Vec<(&'static str, ImportedAsset)> {
    BUILTIN_EDITOR_ICON_LOCATORS
        .iter()
        .copied()
        .map(|locator| {
            (
                locator,
                ImportedAsset::Texture(TextureAsset {
                    uri: AssetUri::parse(locator).expect("builtin editor icon uri"),
                    width: 1,
                    height: 1,
                    rgba: vec![255, 255, 255, 255],
                }),
            )
        })
        .collect()
}

const BUILTIN_EDITOR_ICON_LOCATORS: &[&str] = &[
    "builtin://editor/icons/folder-open-outline.svg",
    "builtin://editor/icons/albums-outline.svg",
    "builtin://editor/icons/list-outline.svg",
    "builtin://editor/icons/options-outline.svg",
    "builtin://editor/icons/cube-outline.svg",
    "builtin://editor/icons/game-controller-outline.svg",
    "builtin://editor/icons/terminal-outline.svg",
    "builtin://editor/icons/layers-outline.svg",
    "builtin://editor/icons/grid-outline.svg",
    "builtin://editor/icons/scan-outline.svg",
    "builtin://editor/icons/move-outline.svg",
    "builtin://editor/icons/sync-outline.svg",
    "builtin://editor/icons/resize-outline.svg",
    "builtin://editor/icons/play-outline.svg",
    "builtin://editor/icons/color-fill-outline.svg",
    "builtin://editor/icons/locate-outline.svg",
    "builtin://editor/icons/ellipse-outline.svg",
];

fn builtin_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("builtin asset reference"))
}

pub(super) fn builtin_pbr_wgsl() -> &'static str {
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
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((model.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * model.tint;
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let lighting = 0.15 + ndotl;
    return vec4<f32>(albedo.rgb * scene.light_color.rgb * lighting, albedo.a);
}
"#
}
