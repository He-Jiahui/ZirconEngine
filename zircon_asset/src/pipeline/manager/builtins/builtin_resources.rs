use crate::load::mesh::generate_cube_mesh;
use crate::{
    AlphaMode, AssetUri, ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset, ShaderAsset,
};

use super::{builtin_pbr_wgsl, builtin_reference, editor_icon_builtin_resources};

pub(in crate::pipeline::manager) fn builtin_resources() -> Vec<(&'static str, ImportedAsset)> {
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
