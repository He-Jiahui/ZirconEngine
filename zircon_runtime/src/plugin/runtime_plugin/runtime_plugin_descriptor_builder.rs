use crate::{
    plugin::ExportPackagingStrategy, plugin::PluginFeatureBundleManifest, RuntimePluginId,
    RuntimeTargetMode,
};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            display_name: display_name.into(),
            category: "runtime".to_string(),
            runtime_id,
            crate_name: crate_name.into(),
            enabled_by_default: true,
            required_by_default: false,
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            optional_features: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    pub fn with_required_by_default(mut self, required: bool) -> Self {
        self.required_by_default = required;
        self
    }

    pub fn with_enabled_by_default(mut self, enabled: bool) -> Self {
        self.enabled_by_default = enabled;
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_optional_feature(mut self, feature: PluginFeatureBundleManifest) -> Self {
        self.optional_features.push(feature);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.default_packaging = packaging.into_iter().collect();
        self
    }
}
