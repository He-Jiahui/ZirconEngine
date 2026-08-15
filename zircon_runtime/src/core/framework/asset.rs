use std::sync::Arc;

use crate::core::resource::{
    ResourceEventReceiver, ResourceManagementGeneration, ResourceRecord, ResourceState,
};

/// Minimal invalidation stamp for caches that do not need a cloned resource record.
///
/// State is part of the identity because reload recovery may reach `Ready` without changing the
/// content revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceCacheIdentity {
    pub revision: u64,
    pub state: ResourceState,
}

pub trait ResourceManager: Send + Sync {
    fn resolve_resource_id(&self, locator: &str) -> Option<String>;
    fn resource_status(&self, locator: &str) -> Option<ResourceRecord>;
    fn resource_management_generation(&self) -> Arc<ResourceManagementGeneration>;
    fn resource_revision(&self, locator: &str) -> Option<u64>;
    fn resource_cache_identity(&self, locator: &str) -> Option<ResourceCacheIdentity> {
        self.resource_status(locator)
            .map(|record| ResourceCacheIdentity {
                revision: record.revision,
                state: record.state,
            })
    }
    fn subscribe_resource_changes(&self) -> ResourceEventReceiver;
}
