use std::collections::HashMap;

use zircon_runtime::asset::assets::{AnimationGraphNodeAsset, ImportedAsset};
use zircon_runtime::asset::{AssetId, AssetReference};

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
                | AnimationGraphNodeAsset::Additive { .. }
                | AnimationGraphNodeAsset::Mask { .. }
                | AnimationGraphNodeAsset::Output { .. } => None,
            }));
        }
        ImportedAsset::AnimationStateMachine(asset) => {
            references.extend(asset.states.iter().map(|state| state.graph.clone()));
        }
        ImportedAsset::MaterialGraph(_)
        | ImportedAsset::Scene(_)
        | ImportedAsset::Terrain(_)
        | ImportedAsset::TerrainLayerStack(_)
        | ImportedAsset::TileSet(_)
        | ImportedAsset::TileMap(_)
        | ImportedAsset::Prefab(_)
        | ImportedAsset::UiIcon(_)
        | ImportedAsset::UiV2View(_)
        | ImportedAsset::UiV2Component(_)
        | ImportedAsset::UiV2Style(_) => {
            references.extend(imported.direct_references());
        }
        ImportedAsset::UiLayout(asset) => {
            references.extend(zircon_runtime::asset::assets::ui_asset_references(
                &asset.document,
            ));
        }
        ImportedAsset::UiWidget(asset) => {
            references.extend(zircon_runtime::asset::assets::ui_asset_references(
                &asset.document,
            ));
        }
        ImportedAsset::UiStyle(asset) => {
            references.extend(zircon_runtime::asset::assets::ui_asset_references(
                &asset.document,
            ));
        }
        ImportedAsset::Data(_)
        | ImportedAsset::Texture(_)
        | ImportedAsset::Shader(_)
        | ImportedAsset::Model(_)
        | ImportedAsset::Mesh(_)
        | ImportedAsset::Sound(_)
        | ImportedAsset::Font(_)
        | ImportedAsset::PhysicsMaterial(_)
        | ImportedAsset::NavMesh(_)
        | ImportedAsset::NavigationSettings(_)
        | ImportedAsset::AnimationSkeleton(_)
        | ImportedAsset::AnimationSequence(_)
        | ImportedAsset::UiTheme(_) => {}
    }

    dedup_references(references)
}

fn dedup_references(references: Vec<AssetReference>) -> Vec<AssetReference> {
    let mut seen = HashMap::<AssetId, AssetReference>::new();
    for reference in references {
        let id = AssetId::from_asset_uuid(reference.uuid);
        seen.entry(id).or_insert(reference);
    }
    let mut deduped = seen.into_values().collect::<Vec<_>>();
    deduped.sort_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()));
    deduped
}

#[cfg(test)]
mod tests {
    use super::direct_references;
    use zircon_runtime::asset::assets::{
        FontAsset, ImportedAsset, UiIconAsset, UiIconSource, UiIconSourceKind, UiThemeAsset,
    };
    use zircon_runtime_interface::ui::style::UiThemeDocument;

    #[test]
    fn font_assets_do_not_report_direct_references() {
        let imported = ImportedAsset::Font(FontAsset {
            source: "res://fonts/default.ttf".to_string(),
            family: None,
            render_mode: None,
            face_index: 0,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: Vec::new(),
            render_strategy: Default::default(),
            metadata: None,
        });

        assert!(direct_references(&imported).is_empty());
    }

    #[test]
    fn ui_theme_assets_do_not_report_direct_references() {
        let imported = ImportedAsset::UiTheme(UiThemeAsset {
            document: UiThemeDocument::dark(),
        });

        assert!(direct_references(&imported).is_empty());
    }

    #[test]
    fn ui_icon_assets_report_external_source_reference() {
        let imported = ImportedAsset::UiIcon(UiIconAsset {
            source: UiIconSource {
                kind: UiIconSourceKind::SvgAsset,
                text: None,
                uri: Some("res://ui/icons/run.svg".to_string()),
            },
            default_size: 18.0,
            semantic_id: "icons/run".to_string(),
        });

        let references = direct_references(&imported);

        assert_eq!(
            references
                .iter()
                .map(|reference| reference.locator.to_string())
                .collect::<Vec<_>>(),
            vec!["res://ui/icons/run.svg"]
        );
    }
}
