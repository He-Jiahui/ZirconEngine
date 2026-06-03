use super::FeatureStatus;

impl FeatureStatus {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn is_available(&self) -> bool {
        self.missing_plugins.is_empty()
            && self.missing_capabilities.is_empty()
            && !self.target_unsupported
            && !self.cycle
            && !self.invalid_owner_dependency
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn is_immediately_blocked(
        &self,
    ) -> bool {
        !self.missing_plugins.is_empty() || self.target_unsupported || self.invalid_owner_dependency
    }
}
