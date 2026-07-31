use crate::core::framework::project::ProjectPluginFeatureSelection;

use super::super::feature_report::RuntimePluginFeatureBlock;
use super::FeatureStatus;

impl FeatureStatus {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn into_block(
        mut self,
        selection: &ProjectPluginFeatureSelection,
    ) -> RuntimePluginFeatureBlock {
        self.missing_capabilities
            .retain(|capability| self.missing_capability_membership.contains(capability));
        RuntimePluginFeatureBlock {
            feature_id: self.feature_id,
            owner_plugin_id: self.owner_plugin_id,
            required: selection.required,
            missing_plugins: self.missing_plugins,
            missing_capabilities: self.missing_capabilities,
            target_unsupported: self.target_unsupported,
            cycle: self.cycle,
            invalid_owner_dependency: self.invalid_owner_dependency,
            unknown_feature: false,
        }
    }
}
