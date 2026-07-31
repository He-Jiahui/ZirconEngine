mod availability;
mod block_projection;
mod capability_wait;
mod mutation;

use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub(super) struct FeatureStatus {
    feature_id: String,
    owner_plugin_id: String,
    missing_plugins: Vec<String>,
    missing_capabilities: Vec<String>,
    missing_plugin_membership: HashSet<String>,
    missing_capability_membership: HashSet<String>,
    target_unsupported: bool,
    cycle: bool,
    invalid_owner_dependency: bool,
}

impl FeatureStatus {
    pub(super) fn new(feature_id: String, owner_plugin_id: String) -> Self {
        Self {
            feature_id,
            owner_plugin_id,
            ..Self::default()
        }
    }

    pub(super) fn missing_capabilities(&self) -> &[String] {
        &self.missing_capabilities
    }
}
