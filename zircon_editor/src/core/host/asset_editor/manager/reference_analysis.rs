use std::collections::HashMap;

use zircon_runtime::asset::{AssetId, AssetReference};
use zircon_runtime::asset::assets::{AnimationGraphNodeAsset, ImportedAsset};

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
        ImportedAsset::AnimationClip(asset) => {
            references.push(asset.skeleton.clone());
        }
        ImportedAsset::AnimationGraph(asset) => {
            references.extend(asset.nodes.iter().filter_map(|node| match node {
                AnimationGraphNodeAsset::Clip { clip, .. } => Some(clip.clone()),
                AnimationGraphNodeAsset::Blend { .. }
                | AnimationGraphNodeAsset::Output { .. } => None,
            }));
        }
        ImportedAsset::AnimationStateMachine(asset) => {
            references.extend(asset.states.iter().map(|state| state.graph.clone()));
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
            references.extend(zircon_runtime::asset::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiWidget(asset) => {
            references.extend(zircon_runtime::asset::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiStyle(asset) => {
            references.extend(zircon_runtime::asset::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::Texture(_)
        | ImportedAsset::Shader(_)
        | ImportedAsset::Model(_)
        | ImportedAsset::PhysicsMaterial(_)
        | ImportedAsset::AnimationSkeleton(_)
        | ImportedAsset::AnimationSequence(_) => {}
    }

    dedup_references(references)
}

fn dedup_references(references: Vec<AssetReference>) -> Vec<AssetReference> {
    let mut seen = HashMap::<AssetId, AssetReference>::new();
    for reference in references {
        let id = AssetId::from_asset_uuid_label(reference.uuid, reference.locator.label());
        seen.entry(id).or_insert(reference);
    }
    let mut deduped = seen.into_values().collect::<Vec<_>>();
    deduped.sort_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()));
    deduped
}
