use std::sync::Arc;

use crate::core::resource::{
    ResourceData, ResourceHandle, ResourceId, ResourceLease, ResourceMarker, RuntimeResourceState,
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
        let snapshot = self.snapshot::<TMarker, TData>(handle)?;
        let payload = Arc::clone(snapshot.resource());
        {
            let mut runtime = self.lock_runtime_write();
            let slot = runtime.entry(handle.id()).or_default();
            slot.ref_count += 1;
            slot.state = RuntimeResourceState::Loaded;
        }

        let manager = self.clone();
        Some(ResourceLease::new(
            handle.id(),
            payload,
            Arc::new(move |id| {
                let _ = manager.release(id);
            }),
        ))
    }

    pub fn release(&self, id: ResourceId) -> Option<usize> {
        let next_ref_count = {
            let mut runtime = self.lock_runtime_write();
            let slot = runtime.get_mut(&id)?;
            if slot.ref_count == 0 {
                return Some(0);
            }
            slot.ref_count -= 1;
            if slot.ref_count == 0 {
                slot.state = RuntimeResourceState::Unloaded;
            }
            slot.ref_count
        };

        if next_ref_count == 0 {
            self.lock_payloads_write().remove(&id);
        }

        Some(next_ref_count)
    }

    pub fn ref_count(&self, id: ResourceId) -> Option<usize> {
        self.lock_runtime_read()
            .get(&id)
            .map(|slot| slot.ref_count)
            .or_else(|| self.registry().get(id).map(|_| 0))
    }

    pub fn runtime_state(&self, id: ResourceId) -> Option<RuntimeResourceState> {
        self.lock_runtime_read()
            .get(&id)
            .map(|slot| slot.state)
            .or_else(|| {
                self.registry()
                    .get(id)
                    .map(|_| RuntimeResourceState::Unloaded)
            })
    }
}
