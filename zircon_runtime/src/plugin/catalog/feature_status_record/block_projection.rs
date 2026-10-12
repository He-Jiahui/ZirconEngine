use crate::core::framework::project::ProjectPluginFeatureSelection;

use super::super::feature_report::RuntimePluginFeatureBlock;
use super::FeatureStatus;

impl FeatureStatus {
    // 已解锁能力会从成员集合移除；在形成公开诊断前才压缩原顺序列表。
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
            provider_missing: self.provider_missing,
            unknown_feature: false,
        }
    }
}
