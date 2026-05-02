use serde::{Deserialize, Serialize};

use crate::{
    asset::AssetImporterDescriptor, plugin::ComponentTypeDescriptor, plugin::ExportPackagingStrategy,
    plugin::ExportTargetPlatform, RuntimeTargetMode, plugin::UiComponentDescriptor,
};

use super::{
    PluginDependencyManifest, PluginEventCatalogManifest, PluginFeatureBundleManifest,
    PluginModuleManifest, PluginOptionManifest,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub id: String,
    pub version: String,
    #[serde(default = "default_sdk_api_version")]
    pub sdk_api_version: String,
    pub display_name: String,
    #[serde(default = "default_plugin_category")]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub supported_targets: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub supported_platforms: Vec<ExportTargetPlatform>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub asset_roots: Vec<String>,
    #[serde(default)]
    pub content_roots: Vec<String>,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub dependencies: Vec<PluginDependencyManifest>,
    #[serde(default)]
    pub options: Vec<PluginOptionManifest>,
    #[serde(default)]
    pub event_catalogs: Vec<PluginEventCatalogManifest>,
    #[serde(default)]
    pub components: Vec<ComponentTypeDescriptor>,
    #[serde(default)]
    pub ui_components: Vec<UiComponentDescriptor>,
    #[serde(default)]
    pub asset_importers: Vec<AssetImporterDescriptor>,
    #[serde(default)]
    pub optional_features: Vec<PluginFeatureBundleManifest>,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
}

fn default_plugin_category() -> String {
    "uncategorized".to_string()
}

fn default_sdk_api_version() -> String {
    "0.1.0".to_string()
}
