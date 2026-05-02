use serde::{Deserialize, Serialize};

use crate::plugin::ExportPackagingStrategy;

use super::{PluginFeatureDependency, PluginModuleManifest};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFeatureBundleManifest {
    pub id: String,
    pub display_name: String,
    pub owner_plugin_id: String,
    #[serde(default)]
    pub dependencies: Vec<PluginFeatureDependency>,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default)]
    pub enabled_by_default: bool,
}

impl PluginFeatureBundleManifest {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        owner_plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            owner_plugin_id: owner_plugin_id.into(),
            dependencies: Vec::new(),
            modules: Vec::new(),
            capabilities: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
            enabled_by_default: false,
        }
    }

    pub fn with_dependency(mut self, dependency: PluginFeatureDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_runtime_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_editor_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.default_packaging = packaging.into_iter().collect();
        self
    }

    pub fn enabled_by_default(mut self, enabled: bool) -> Self {
        self.enabled_by_default = enabled;
        self
    }
}
