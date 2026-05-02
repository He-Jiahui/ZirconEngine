use crate::{plugin::ExportPackagingStrategy, RuntimePluginId, RuntimeTargetMode};

use super::{ProjectPluginFeatureSelection, ProjectPluginSelection};

impl ProjectPluginSelection {
    pub fn runtime_plugin(id: RuntimePluginId, enabled: bool, required: bool) -> Self {
        Self {
            id: id.key().to_string(),
            enabled,
            required,
            target_modes: Vec::new(),
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        }
    }

    pub fn with_runtime_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.runtime_crate = Some(crate_name.into());
        self
    }

    pub fn with_editor_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.editor_crate = Some(crate_name.into());
        self
    }

    pub fn with_packaging(mut self, packaging: ExportPackagingStrategy) -> Self {
        self.packaging = packaging;
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_feature(mut self, feature: ProjectPluginFeatureSelection) -> Self {
        self.features.push(feature);
        self
    }
}
