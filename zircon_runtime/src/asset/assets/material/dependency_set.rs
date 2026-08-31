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
    collect_direct_references(material, Clone::clone)
}

impl MaterialAsset {
    pub(crate) fn direct_reference_locators(&self) -> Vec<ResourceLocator> {
        collect_direct_references(self, |reference| reference.locator.clone())
    }
}

fn collect_direct_references<T: PartialEq>(
    material: &MaterialAsset,
    mut project: impl FnMut(&AssetReference) -> T,
) -> Vec<T> {
    let texture_slots = material.all_texture_slots();
    let capacity = 1usize
        .saturating_add(texture_slots.len())
        .saturating_add(usize::from(material.parent.is_some()));
    let mut references = Vec::with_capacity(capacity);
    references.push(project(&material.shader));
    for (_, texture) in texture_slots {
        let texture = project(texture);
        if !references[1..].contains(&texture) {
            references.push(texture);
        }
    }
    if let Some(parent) = material.parent.as_ref() {
        references.push(project(parent));
    }
    references
}

#[cfg(test)]
mod tests {
    use super::MaterialAsset;

    #[test]
    fn direct_reference_projections_share_order_texture_dedup_and_parent() {
        let material = MaterialAsset::from_toml_str(
            r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[parent]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://materials/parent.zmaterial"

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000003"
url = "res://textures/shared.png"

[textures.normal]
uuid = "00000000-0000-0000-0000-000000000003"
url = "res://textures/shared.png"
"#,
        )
        .expect("material dependency fixture");

        let references = material.direct_references();
        let locators = material.direct_reference_locators();

        assert_eq!(references.len(), 3);
        assert_eq!(
            locators,
            references
                .iter()
                .map(|reference| reference.locator.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(locators[0].to_string(), "res://shaders/pbr.zshader");
        assert_eq!(locators[1].to_string(), "res://textures/shared.png");
        assert_eq!(locators[2].to_string(), "res://materials/parent.zmaterial");
    }
}
