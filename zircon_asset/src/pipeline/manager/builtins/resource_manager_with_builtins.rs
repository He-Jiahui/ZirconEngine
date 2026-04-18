use zircon_resource::ResourceManager;

use crate::{AssetId, AssetKind, AssetMetadata};

use super::super::resource_sync::register_project_resource;
use super::builtin_resources;

pub(in crate::pipeline::manager) fn resource_manager_with_builtins() -> ResourceManager {
    let manager = ResourceManager::new();

    for (locator_text, asset) in builtin_resources() {
        let locator = crate::AssetUri::parse(locator_text).expect("builtin locator");
        let kind = match &asset {
            crate::ImportedAsset::Texture(_) => AssetKind::Texture,
            crate::ImportedAsset::Shader(_) => AssetKind::Shader,
            crate::ImportedAsset::Material(_) => AssetKind::Material,
            crate::ImportedAsset::Scene(_) => AssetKind::Scene,
            crate::ImportedAsset::Model(_) => AssetKind::Model,
            crate::ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
            crate::ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
            crate::ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
        };
        let record = AssetMetadata::new(AssetId::from_locator(&locator), kind, locator);
        register_project_resource(&manager, record, asset);
    }

    manager
}
