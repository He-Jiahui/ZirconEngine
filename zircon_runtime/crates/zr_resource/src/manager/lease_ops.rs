use std::sync::Arc;

use crate::{
    lease::ResourceLeaseIdentity, ResourceData, ResourceHandle, ResourceId, ResourceLease,
    ResourceMarker, ResourceState, RuntimeResourceState,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    /// 同时检查句柄种类与载荷类型，并为当前驻留身份建立租约；重载或失败时仍可租用最后有效载荷。
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
            let active = authority.active.row(handle.id())?;
            if active.record.kind != TMarker::KIND {
                return None;
            }
            let record_state = authority.registry.get(handle.id()).map(|record| record.state);
            let payload = authority.payloads.get(&handle.id())?.clone();
            let payload = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
            let slot = authority.runtime.get_mut(&handle.id())?;
            let lease_identity = Arc::clone(&slot.lease_identity);
            if record_state == Some(ResourceState::Ready) {
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
        let retired = {
            let mut authority = self.lock_authority_write();
            let preserve_last_good = authority.registry.get(id).is_some_and(|record| {
                matches!(record.state, ResourceState::Reloading | ResourceState::Error)
            });
            let Some(slot) = authority.runtime.get_mut(&id) else { return; };
            if !Arc::ptr_eq(&slot.lease_identity, &lease_identity) { return; }
            drop(lease_identity);
            if Arc::strong_count(&slot.lease_identity) != 1 { return; }
            if let Some(reservation) = &mut slot.prepared {
                reservation.deferred_last_release = true;
                return;
            }
            if preserve_last_good || slot.retain_last_good { return; }
            slot.state = RuntimeResourceState::Unloaded;
            let retired = authority.payloads.remove(&id);
            authority.active = crate::ResourceActiveGeneration::updated(
                &authority.active, vec![(id, None)], &[],
            );
            authority.refresh_readiness_many([id]);
            retired
        };
        // A payload may drop nested resource leases.
        drop(retired);
    }

    /// 统计当前驻留身份的租约，不计载荷 Arc 或旧代租约；已登记但无运行槽为零，目录缺失为 None。
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
