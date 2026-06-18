use crate::builtin::RuntimeTargetMode;
use crate::{
    plugin::CapabilityStatusManifest, plugin::ExportPackagingStrategy,
    plugin::PluginFeatureBundleManifest, plugin::PluginMaturity,
};

use super::super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
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

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_sets = system_sets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_anchors = system_anchors.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.maturity = maturity;
        self
    }

    pub fn with_capability_status(mut self, status: CapabilityStatusManifest) -> Self {
        self.capability_statuses.push(status);
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
