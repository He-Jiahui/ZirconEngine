use std::collections::{HashMap, HashSet};

use crate::RuntimeTargetMode;

use super::super::feature_capabilities::feature_declares_capability_for_target;
use super::super::feature_definitions::FeatureDefinition;
use super::FeatureStatus;

impl FeatureStatus {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn is_waiting_for_feature_capability(
        &self,
        definitions: &HashMap<String, FeatureDefinition>,
        unresolved_feature_ids: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> bool {
        !self.missing_capabilities.is_empty()
            && self.missing_plugins.is_empty()
            && !self.target_unsupported
            && !self.invalid_owner_dependency
            && self.missing_capabilities.iter().all(|capability| {
                definitions.iter().any(|(key, candidate)| {
                    unresolved_feature_ids.contains(key)
                        && feature_declares_capability_for_target(candidate, capability, target)
                })
            })
    }
}
