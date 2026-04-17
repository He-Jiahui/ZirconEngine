use std::collections::HashMap;

use crate::{AssetReference, ImportedAsset, ResourceId};

pub(super) fn direct_references(imported: &ImportedAsset) -> Vec<AssetReference> {
    let mut references = Vec::new();
    match imported {
        ImportedAsset::Material(material) => {
            references.push(material.shader.clone());
            references.extend(
                [
                    material.base_color_texture.clone(),
                    material.normal_texture.clone(),
                    material.metallic_roughness_texture.clone(),
                    material.occlusion_texture.clone(),
                    material.emissive_texture.clone(),
                ]
                .into_iter()
                .flatten(),
            );
        }
        ImportedAsset::Scene(scene) => {
            for entity in &scene.entities {
                if let Some(mesh) = &entity.mesh {
                    references.push(mesh.model.clone());
                    references.push(mesh.material.clone());
                }
            }
        }
        ImportedAsset::UiLayout(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiWidget(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiStyle(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::Texture(_) | ImportedAsset::Shader(_) | ImportedAsset::Model(_) => {}
    }

    dedup_references(references)
}

fn dedup_references(references: Vec<AssetReference>) -> Vec<AssetReference> {
    let mut seen = HashMap::<ResourceId, AssetReference>::new();
    for reference in references {
        let id = ResourceId::from_asset_uuid_label(reference.uuid, reference.locator.label());
        seen.entry(id).or_insert(reference);
    }
    let mut deduped = seen.into_values().collect::<Vec<_>>();
    deduped.sort_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()));
    deduped
}
