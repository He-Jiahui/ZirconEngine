use std::sync::Arc;

use crate::{
    ResourceData, ResourceHandle, ResourceId, ResourceLease, ResourceMarker, ResourceState,
    RuntimeResourceState, lease::ResourceLeaseIdentity,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn acquire<TMarker, TData>(
        &self,
        handle: ResourceHandle<TMarker>,
    ) -> Option<ResourceLease<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let (payload, lease_identity) = {
            let mut authority = self.lock_authority_write();
            let record = authority.registry.get(handle.id())?;
            if record.kind != TMarker::KIND {
                return None;
            }
            let record_state = record.state;
            let payload = authority.payloads.get(&handle.id())?.clone();
            let payload = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
            let slot = authority.runtime.get_mut(&handle.id())?;
            let lease_identity = Arc::clone(&slot.lease_identity);
            if record_state == ResourceState::Ready {
                slot.state = RuntimeResourceState::Loaded;
            }
            authority.refresh_readiness_many([handle.id()]);
            (payload, lease_identity)
        };

        let manager = self.clone();
        Some(ResourceLease::new(
            handle.id(),
            lease_identity,
            payload,
            Arc::new(move |id, lease_identity| {
                manager.release_residency(id, lease_identity);
            }),
        ))
    }

    fn release_residency(&self, id: ResourceId, lease_identity: Arc<ResourceLeaseIdentity>) {
        let mut authority = self.lock_authority_write();
        let preserve_last_good = authority.registry.get(id).is_some_and(|record| {
            matches!(
                record.state,
                ResourceState::Reloading | ResourceState::Error
            )
        });
        let Some(slot) = authority.runtime.get_mut(&id) else {
            return;
        };
        if !Arc::ptr_eq(&slot.lease_identity, &lease_identity) {
            return;
        }
        drop(lease_identity);
        if Arc::strong_count(&slot.lease_identity) == 1 && !preserve_last_good {
            slot.state = RuntimeResourceState::Unloaded;
            authority.payloads.remove(&id);
            authority.refresh_readiness_many([id]);
        }
    }

    pub fn ref_count(&self, id: ResourceId) -> Option<usize> {
        let authority = self.lock_authority_read();
        authority
            .runtime
            .get(&id)
            .map(|slot| Arc::strong_count(&slot.lease_identity).saturating_sub(1))
            .or_else(|| authority.registry.get(id).map(|_| 0))
    }

    pub fn runtime_state(&self, id: ResourceId) -> Option<RuntimeResourceState> {
        let authority = self.lock_authority_read();
        authority
            .runtime
            .get(&id)
            .map(|slot| slot.state)
            .or_else(|| {
                authority
                    .registry
                    .get(id)
                    .map(|_| RuntimeResourceState::Unloaded)
            })
    }
}
