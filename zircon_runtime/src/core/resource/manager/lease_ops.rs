use std::sync::Arc;

use crate::core::resource::{
    ResourceData, ResourceHandle, ResourceId, ResourceLease, ResourceMarker, ResourceState,
    RuntimeResourceState,
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
        let (payload, residency_token) = {
            let mut authority = self.lock_authority_write();
            let record = authority.registry.get(handle.id())?;
            if record.kind != TMarker::KIND {
                return None;
            }
            let record_state = record.state;
            let payload = authority.payloads.get(&handle.id())?.clone();
            let payload = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
            let slot = authority.runtime.get_mut(&handle.id())?;
            slot.ref_count += 1;
            if record_state == ResourceState::Ready {
                slot.state = RuntimeResourceState::Loaded;
            }
            let residency_token = slot.residency_token;
            authority.refresh_readiness_many([handle.id()]);
            (payload, residency_token)
        };

        let manager = self.clone();
        Some(ResourceLease::new(
            handle.id(),
            residency_token,
            payload,
            Arc::new(move |id, token| {
                manager.release_residency(id, token);
            }),
        ))
    }

    fn release_residency(&self, id: ResourceId, residency_token: u64) {
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
        if slot.residency_token != residency_token || slot.ref_count == 0 {
            return;
        }
        slot.ref_count -= 1;
        if slot.ref_count == 0 && !preserve_last_good {
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
            .map(|slot| slot.ref_count)
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
