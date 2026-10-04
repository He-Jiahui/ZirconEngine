use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    event_stream::ResourceEventPublisher, ResourceData, ResourceEventReceiver,
    ResourceEventStreamDiagnostics, ResourceId, ResourceManagementGeneration,
    ResourceManagementGenerationIdentity, ResourceReadinessGeneration,
    ResourceReadinessGenerationIdentity, ResourceRegistry, RuntimeResourceState,
};

use super::management_projection::ResourceManagementProjection;
#[cfg(test)]
use super::readiness_projection::ResourceReadinessSourceUpdate;
use super::readiness_projection::{ResourceReadinessProjection, ResourceReadinessStagedUpdate};
use super::runtime_slot::ResourceRuntimeSlot;

#[cfg(test)]
#[path = "tests/readiness_payload.rs"]
mod readiness_payload_tests;

#[cfg(test)]
#[path = "tests/readiness_pair_profile.rs"]
mod readiness_pair_profile;

pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;
pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;

#[derive(Debug, Default)]
pub(super) struct ResourceAuthority {
    pub(super) registry: ResourceRegistry,
    pub(super) management: ResourceManagementProjection,
    pub(super) payloads: ResourcePayloadMap,
    pub(super) runtime: ResourceRuntimeMap,
    pub(super) readiness: ResourceReadinessProjection,
}

impl ResourceAuthority {
    pub(super) fn refresh_readiness_many(&mut self, ids: impl IntoIterator<Item = ResourceId>) {
        let mut ids = ids.into_iter().peekable();
        let Some(first) = ids.next() else {
            return;
        };
        let Some(second) = ids.next() else {
            self.refresh_readiness_one(first);
            return;
        };
        if ids.peek().is_none() {
            self.refresh_readiness_two(first, second);
            return;
        }

        let (remaining_lower_bound, _) = ids.size_hint();
        let mut unique_ids = Vec::with_capacity(remaining_lower_bound.saturating_add(2));
        unique_ids.push(first);
        unique_ids.push(second);
        unique_ids.extend(ids);
        unique_ids.sort_unstable();
        unique_ids.dedup();

        let Self {
            registry,
            runtime,
            payloads,
            readiness,
            ..
        } = self;
        // Compare borrowed records before cloning. Stage the changed records together so
        // projection mutation does not interleave with record allocation or source reads.
        let update_capacity = unique_ids.len();
        let mut updates = Vec::new();
        unique_ids.retain(|id| {
            if let Some(update) = Self::changed_readiness_source_update_from(
                registry, runtime, payloads, readiness, *id,
            ) {
                if updates.is_empty() {
                    updates.reserve_exact(update_capacity);
                }
                updates.push(update);
                true
            } else {
                false
            }
        });
        if updates.is_empty() {
            return;
        }
        readiness.apply_staged_many_updates(updates, unique_ids);
    }

    fn refresh_readiness_two(&mut self, first: ResourceId, second: ResourceId) {
        if first == second {
            self.refresh_readiness_one(first);
            return;
        }

        let (first, second) = if first < second {
            (first, second)
        } else {
            (second, first)
        };
        let Self {
            registry,
            runtime,
            payloads,
            readiness,
            ..
        } = self;
        let updates = [
            Self::changed_readiness_source_update_from(
                registry, runtime, payloads, readiness, first,
            ),
            Self::changed_readiness_source_update_from(
                registry, runtime, payloads, readiness, second,
            ),
        ];
        readiness.apply_staged_updates(updates.into_iter().flatten());
    }

    fn refresh_readiness_one(&mut self, id: ResourceId) {
        let Self {
            registry,
            runtime,
            payloads,
            readiness,
            ..
        } = self;
        let update =
            Self::changed_readiness_source_update_from(registry, runtime, payloads, readiness, id);
        readiness.apply_staged_updates(update);
    }

    fn changed_readiness_source_update_from(
        registry: &ResourceRegistry,
        runtime: &ResourceRuntimeMap,
        payloads: &ResourcePayloadMap,
        readiness: &ResourceReadinessProjection,
        id: ResourceId,
    ) -> Option<ResourceReadinessStagedUpdate> {
        let record = registry.get(id);
        let runtime_state = runtime
            .get(&id)
            .map(|slot| slot.state)
            .unwrap_or(RuntimeResourceState::Unloaded);
        let payload_type_id = payloads
            .get(&id)
            .map(|payload| payload.as_ref().as_any().type_id());
        readiness.stage_source_update(id, record, runtime_state, payload_type_id)
    }

    #[cfg(test)]
    fn readiness_source_update(&self, id: ResourceId) -> ResourceReadinessSourceUpdate {
        Self::readiness_source_update_from(&self.registry, &self.runtime, &self.payloads, id)
    }

    #[cfg(test)]
    fn readiness_source_update_from(
        registry: &ResourceRegistry,
        runtime: &ResourceRuntimeMap,
        payloads: &ResourcePayloadMap,
        id: ResourceId,
    ) -> ResourceReadinessSourceUpdate {
        ResourceReadinessSourceUpdate {
            id,
            record: registry.get(id).cloned(),
            runtime_state: runtime
                .get(&id)
                .map(|slot| slot.state)
                .unwrap_or(RuntimeResourceState::Unloaded),
            payload_type_id: payloads
                .get(&id)
                .map(|payload| payload.as_ref().as_any().type_id()),
        }
    }
}

#[derive(Debug)]
pub struct ResourceRegistryReadGuard<'a> {
    guard: RwLockReadGuard<'a, ResourceAuthority>,
}

impl Deref for ResourceRegistryReadGuard<'_> {
    type Target = ResourceRegistry;

    fn deref(&self) -> &Self::Target {
        &self.guard.registry
    }
}

/// Exact management/readiness pair captured under one Resource authority lock.
#[derive(Clone, Debug)]
pub struct ResourceProjectionSnapshot {
    management: Arc<ResourceManagementGeneration>,
    readiness: Arc<ResourceReadinessGeneration>,
}

impl ResourceProjectionSnapshot {
    pub(super) fn new(
        management: Arc<ResourceManagementGeneration>,
        readiness: Arc<ResourceReadinessGeneration>,
    ) -> Self {
        Self {
            management,
            readiness,
        }
    }

    pub fn management(&self) -> &Arc<ResourceManagementGeneration> {
        &self.management
    }

    pub fn readiness(&self) -> &Arc<ResourceReadinessGeneration> {
        &self.readiness
    }

    pub fn management_identity(&self) -> ResourceManagementGenerationIdentity {
        self.management.identity()
    }

    pub fn readiness_identity(&self) -> ResourceReadinessGenerationIdentity {
        self.readiness.identity()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    pub(super) authority: Arc<RwLock<ResourceAuthority>>,
    commit_serial: Arc<Mutex<()>>,
    pub(super) events: ResourceEventPublisher,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> ResourceEventReceiver {
        self.events.subscribe()
    }

    pub fn event_stream_diagnostics(&self) -> ResourceEventStreamDiagnostics {
        self.events.diagnostics()
    }

    pub fn projection_snapshot(&self) -> ResourceProjectionSnapshot {
        let authority = self
            .authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ResourceProjectionSnapshot::new(
            authority.management.generation(),
            authority.readiness.generation(),
        )
    }

    pub fn management_generation(&self) -> Arc<ResourceManagementGeneration> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .management
            .generation()
    }

    pub fn readiness_generation(&self) -> Arc<ResourceReadinessGeneration> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .readiness
            .generation()
    }

    pub(super) fn lock_registry_read(&self) -> ResourceRegistryReadGuard<'_> {
        ResourceRegistryReadGuard {
            guard: self
                .authority
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub(super) fn lock_authority_read(&self) -> RwLockReadGuard<'_, ResourceAuthority> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_authority_write(&self) -> RwLockWriteGuard<'_, ResourceAuthority> {
        self.authority
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_commit_serial(&self) -> MutexGuard<'_, ()> {
        self.commit_serial
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn commit_gate_is_locked_for_test(&self) -> bool {
        self.commit_serial.try_lock().is_err()
    }

    pub fn registry(&self) -> ResourceRegistryReadGuard<'_> {
        self.lock_registry_read()
    }

    #[cfg(test)]
    fn poison_event_stream_for_test(&self) {
        self.events.poison_state();
    }

    #[cfg(test)]
    pub(crate) fn set_event_next_sequence_for_test(&self, next_sequence: Option<u64>) {
        self.events.set_next_sequence_for_test(next_sequence);
    }
}

#[cfg(test)]
#[path = "tests/resource_manager.rs"]
mod tests;
