use zircon_resource::ResourceManager;

use crate::{AssetMetadata, ImportedAsset};

pub(in crate::pipeline::manager) fn register_project_resource(
    resource_manager: &ResourceManager,
    metadata: AssetMetadata,
    imported: ImportedAsset,
) {
    match imported {
        ImportedAsset::Texture(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Shader(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Material(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Scene(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Model(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiLayout(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiWidget(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiStyle(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
    }
}
