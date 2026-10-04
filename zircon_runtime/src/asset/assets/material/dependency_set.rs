use std::collections::HashSet;
use std::hash::Hash;

use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialDependencySet;
use crate::core::resource::ResourceLocator;

use super::MaterialAsset;

pub fn material_dependency_set(material: &MaterialAsset) -> RenderMaterialDependencySet {
    let mut dependencies = RenderMaterialDependencySet::new(material.shader.clone());
    for (_, texture) in material.all_texture_slots() {
        dependencies.push_texture(texture.clone());
    }
    dependencies
}

pub fn direct_references(material: &MaterialAsset) -> Vec<AssetReference> {
    collect_direct_references(material, Clone::clone, asset_reference_key)
}

impl MaterialAsset {
    pub(crate) fn direct_reference_locators(&self) -> Vec<ResourceLocator> {
        collect_direct_references(
            self,
            |reference| reference.locator.clone(),
            reference_locator,
        )
    }
}

fn asset_reference_key(reference: &AssetReference) -> &AssetReference {
    reference
}

fn reference_locator(reference: &AssetReference) -> &ResourceLocator {
    &reference.locator
}

fn collect_direct_references<'a, T, K: Eq + Hash + 'a>(
    material: &'a MaterialAsset,
    mut project: impl FnMut(&AssetReference) -> T,
    key: impl Fn(&'a AssetReference) -> &'a K,
) -> Vec<T> {
    let texture_slots = material.all_texture_slots();
    let capacity = 1usize
        .saturating_add(texture_slots.len())
        .saturating_add(usize::from(material.parent.is_some()));
    let mut references = Vec::with_capacity(capacity);
    references.push(project(&material.shader));
    let mut texture_keys = HashSet::with_capacity(texture_slots.len());
    for (_, texture) in texture_slots {
        if texture_keys.insert(key(texture)) {
            references.push(project(texture));
        }
    }
    if let Some(parent) = material.parent.as_ref() {
        references.push(project(parent));
    }
    references
}

#[cfg(test)]
#[path = "tests/dependency_set.rs"]
mod tests;

#[cfg(test)]
#[path = "dependency_set/tests/optimization_batch_hy_runtime608_tests.rs"]
mod optimization_batch_hy_runtime608_tests;
