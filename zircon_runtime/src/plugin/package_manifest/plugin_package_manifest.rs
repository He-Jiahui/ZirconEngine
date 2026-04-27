use serde::{Deserialize, Serialize};

use crate::{ComponentTypeDescriptor, ExportPackagingStrategy, UiComponentDescriptor};

use super::PluginModuleManifest;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub id: String,
    pub version: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub components: Vec<ComponentTypeDescriptor>,
    #[serde(default)]
    pub ui_components: Vec<UiComponentDescriptor>,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
}
