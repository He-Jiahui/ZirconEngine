use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::FeatureStatus;

impl FeatureStatus {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn is_waiting_for_feature_capability(
        &self,
        projection: &RuntimePluginCatalogProjection,
        unresolved_feature_ids: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> bool {
        !self.missing_capability_membership.is_empty()
            && self.missing_plugins.is_empty()
            && !self.target_unsupported
            && !self.invalid_owner_dependency
            && !self.provider_missing
            && self.missing_capability_membership.iter().all(|capability| {
                projection.capability_has_unresolved_provider(
                    capability,
                    unresolved_feature_ids,
                    target,
                )
            })
    }
}
