use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialDependencySet;

use super::MaterialAsset;

pub fn material_dependency_set(material: &MaterialAsset) -> RenderMaterialDependencySet {
    let mut dependencies = RenderMaterialDependencySet::new(material.shader.clone());
    for texture in [
        material.base_color_texture.as_ref(),
        material.normal_texture.as_ref(),
        material.metallic_roughness_texture.as_ref(),
        material.occlusion_texture.as_ref(),
        material.emissive_texture.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        dependencies.push_texture(texture.clone());
    }
    dependencies
}

pub fn direct_references(material: &MaterialAsset) -> Vec<AssetReference> {
    material_dependency_set(material).all_references()
}
