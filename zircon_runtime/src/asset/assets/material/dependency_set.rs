use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialDependencySet;

use super::MaterialAsset;

pub fn material_dependency_set(material: &MaterialAsset) -> RenderMaterialDependencySet {
    let mut dependencies = RenderMaterialDependencySet::new(material.shader.clone());
    for (_, texture) in material.all_texture_slots() {
        dependencies.push_texture(texture.clone());
    }
    dependencies
}

pub fn direct_references(material: &MaterialAsset) -> Vec<AssetReference> {
    material_dependency_set(material).all_references()
}
