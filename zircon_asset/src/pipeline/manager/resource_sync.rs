use std::collections::HashSet;

use crate::{AssetId, AssetMetadata, AssetUri, ImportedAsset, ProjectManager, ResourceManager};

pub(super) fn project_locators(project: &ProjectManager) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}

pub(super) fn clear_removed_project_resources(
    resource_manager: &ResourceManager,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) {
    let current = project_locators(project);
    for locator in previous_locators.difference(&current) {
        let _ = resource_manager.remove_by_locator(locator);
    }
}

pub(super) fn register_project_resource(
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

pub(super) fn store_runtime_payload(
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
