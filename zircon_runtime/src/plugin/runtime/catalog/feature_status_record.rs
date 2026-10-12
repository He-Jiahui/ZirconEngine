#[path = "feature_status_record/availability.rs"]
mod availability;
#[path = "../../catalog/feature_status_record/block_projection.rs"]
mod block_projection;
#[path = "../../catalog/feature_status_record/capability_wait.rs"]
mod capability_wait;
#[path = "feature_status_record/mutation.rs"]
mod mutation;

use std::collections::HashSet;

// 同时保留诊断显示顺序和成员集合：能力解锁时只改集合，最终投影再过滤显示列表。
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
    provider_missing: bool,
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
