use crate::asset::load::mesh::generate_cube_mesh;
use crate::asset::{
    AlphaMode, AssetUri, ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset,
    ShaderAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::ShaderAssetKind;

use super::{builtin_pbr_wgsl, builtin_reference};

pub(in crate::asset::pipeline::manager) fn builtin_resources() -> Vec<(&'static str, ImportedAsset)>
{
    let mesh = generate_cube_mesh();
    let resources = vec![
        (
            "builtin://cube",
            ImportedAsset::Model(ModelAsset {
                uri: AssetUri::parse("builtin://cube").expect("builtin cube uri"),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: mesh.vertices.clone(),
                    indices: mesh.indices.clone(),
                    mesh: None,
                    mesh_sdf: None,
                    virtual_geometry: None,
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
                    mesh: None,
                    mesh_sdf: None,
                    virtual_geometry: None,
                }],
            }),
        ),
        (
            "builtin://material/default",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Default".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                parent: None,
                options: Default::default(),
                queue: None,
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
                property_values: Default::default(),
                texture_slots: Default::default(),
                validation_diagnostics: Vec::new(),
            }),
        ),
        (
            "builtin://missing-material",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Missing".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                parent: None,
                options: Default::default(),
                queue: None,
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
                property_values: Default::default(),
                texture_slots: Default::default(),
                validation_diagnostics: Vec::new(),
            }),
        ),
        (
            "builtin://shader/pbr.wgsl",
            ImportedAsset::Shader(ShaderAsset {
                uri: AssetUri::parse("builtin://shader/pbr.wgsl").expect("builtin shader uri"),
                kind: ShaderAssetKind::Surface,
                source_language: ShaderSourceLanguage::Wgsl,
                source: builtin_pbr_wgsl().to_string(),
                wgsl_source: builtin_pbr_wgsl().to_string(),
                import_path: None,
                entry_points: Vec::new(),
                dependencies: Vec::new(),
                source_files: Vec::new(),
                imports: Vec::new(),
                shader_defs: Vec::new(),
                property_schema: Vec::new(),
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: Some("standard_pbr".to_string()),
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
            }),
        ),
    ];
    resources
}

#[cfg(test)]
mod tests {
    use crate::asset::{ImportedAsset, ShaderSurfaceSourceContract};

    use super::builtin_resources;

    #[test]
    fn builtin_pbr_shader_publishes_only_the_surface_material_contract() {
        let shader = builtin_resources()
            .into_iter()
            .find_map(|(locator, asset)| (locator == "builtin://shader/pbr.wgsl").then_some(asset))
            .and_then(|asset| match asset {
                ImportedAsset::Shader(shader) => Some(shader),
                _ => None,
            })
            .expect("builtin PBR shader");

        assert!(shader.entry_points.is_empty());
        assert_eq!(
            shader.surface_source_contract(),
            Ok(Some(ShaderSurfaceSourceContract::MaterialFunction))
        );
    }
}
