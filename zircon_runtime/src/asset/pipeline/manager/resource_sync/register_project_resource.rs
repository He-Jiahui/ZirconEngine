use crate::core::resource::ResourceManager;
use crate::core::resource::{ResourceRecord, ResourceState};

use crate::asset::ImportedAsset;

pub(in crate::asset::pipeline::manager) fn register_project_resource(
    resource_manager: &ResourceManager,
    metadata: ResourceRecord,
    imported: ImportedAsset,
) {
    prepare_ready_registration(resource_manager, &metadata);

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
        ImportedAsset::Mesh(asset) => {
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
        ImportedAsset::UiTheme(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiIcon(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiV2View(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiV2Component(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::UiV2Style(asset) => {
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

fn prepare_ready_registration(resource_manager: &ResourceManager, metadata: &ResourceRecord) {
    let previous_state = resource_manager
        .registry()
        .get(metadata.id())
        .map(|record| record.state);
    if matches!(previous_state, Some(ResourceState::Error)) {
        resource_manager.start_reload(metadata.id(), Vec::new());
    }
}
