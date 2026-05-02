use crate::core::resource::ResourceManager;
use crate::core::resource::ResourceRecord;

use crate::asset::ImportedAsset;

pub(in crate::asset::pipeline::manager) fn register_project_resource(
    resource_manager: &ResourceManager,
    metadata: ResourceRecord,
    imported: ImportedAsset,
) {
    match imported {
        ImportedAsset::Data(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Texture(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Shader(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Material(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::MaterialGraph(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Sound(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Font(asset) => {
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
        ImportedAsset::PhysicsMaterial(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::NavMesh(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::NavigationSettings(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Terrain(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::TerrainLayerStack(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::TileSet(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::TileMap(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Prefab(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::AnimationSkeleton(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::AnimationClip(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::AnimationSequence(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::AnimationGraph(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::AnimationStateMachine(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
    }
}
