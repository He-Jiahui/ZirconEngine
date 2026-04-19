use crate::core::resource::ResourceManager;
use crate::core::resource::ResourceRecord;

use crate::asset::{AssetId, AssetKind};

use super::super::resource_sync::register_project_resource;
use super::builtin_resources;

pub(in crate::asset::pipeline::manager) fn resource_manager_with_builtins() -> ResourceManager {
    let manager = ResourceManager::new();

    for (locator_text, asset) in builtin_resources() {
        let locator = crate::asset::AssetUri::parse(locator_text).expect("builtin locator");
        let kind = match &asset {
            crate::asset::ImportedAsset::Texture(_) => AssetKind::Texture,
            crate::asset::ImportedAsset::Shader(_) => AssetKind::Shader,
            crate::asset::ImportedAsset::Material(_) => AssetKind::Material,
            crate::asset::ImportedAsset::Scene(_) => AssetKind::Scene,
            crate::asset::ImportedAsset::Model(_) => AssetKind::Model,
            crate::asset::ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
            crate::asset::ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
            crate::asset::ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
            crate::asset::ImportedAsset::PhysicsMaterial(_) => AssetKind::PhysicsMaterial,
            crate::asset::ImportedAsset::AnimationSkeleton(_) => AssetKind::AnimationSkeleton,
            crate::asset::ImportedAsset::AnimationClip(_) => AssetKind::AnimationClip,
            crate::asset::ImportedAsset::AnimationSequence(_) => AssetKind::AnimationSequence,
            crate::asset::ImportedAsset::AnimationGraph(_) => AssetKind::AnimationGraph,
            crate::asset::ImportedAsset::AnimationStateMachine(_) => AssetKind::AnimationStateMachine,
        };
        let record = ResourceRecord::new(AssetId::from_locator(&locator), kind, locator);
        register_project_resource(&manager, record, asset);
    }

    manager
}
