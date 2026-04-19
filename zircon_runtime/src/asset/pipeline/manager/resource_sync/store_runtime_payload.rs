use crate::core::resource::ResourceManager;

use crate::asset::{AssetId, ImportedAsset};

pub(in crate::asset::pipeline::manager) fn store_runtime_payload(
    resource_manager: &ResourceManager,
    id: AssetId,
    imported: ImportedAsset,
) {
    match imported {
        ImportedAsset::Texture(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Shader(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Material(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Scene(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Model(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::UiLayout(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::UiWidget(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::UiStyle(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::PhysicsMaterial(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::AnimationSkeleton(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::AnimationClip(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::AnimationSequence(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::AnimationGraph(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::AnimationStateMachine(asset) => {
            resource_manager.store_payload(id, asset);
        }
    }
}
