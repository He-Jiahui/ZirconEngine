use crate::core::resource::{ResourceId, ResourceState};

use crate::graphics::types::GraphicsError;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn resource_revision(&self, id: ResourceId) -> Result<u64, GraphicsError> {
        self.asset_manager()?
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.revision)
            .ok_or_else(|| GraphicsError::Asset(format!("missing resource record {id}")))
    }

    /// Runs `operation` against one coherent, ready-only revision view of the resource registry.
    ///
    /// The registry read lock is acquired only once for the complete query, avoiding one lock
    /// acquisition per static mesh while retaining the cache caller's fail-closed `None` result
    /// for pending, failed, untracked, or revision-zero resources.
    pub(crate) fn with_ready_resource_revisions<T>(
        &self,
        operation: impl FnOnce(&mut dyn FnMut(ResourceId) -> Option<u64>) -> T,
    ) -> Option<T> {
        let asset_manager = self.asset_manager().ok()?;
        let resource_manager = asset_manager.resource_manager();
        let registry = resource_manager.registry();
        let mut revision_for = |id: ResourceId| {
            registry.get(id).and_then(|record| {
                (record.state == ResourceState::Ready && record.revision != 0)
                    .then_some(record.revision)
            })
        };
        Some(operation(&mut revision_for))
    }
}
