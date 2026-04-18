use zircon_resource::ResourceManager;

use crate::{AssetId, ImportedAsset};

pub(in crate::pipeline::manager) fn store_runtime_payload(
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
    }
}
