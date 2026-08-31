use std::collections::HashMap;
use std::sync::{Arc, MutexGuard};

use crate::event_stream::ResourceEventPublishPermit;
use crate::{
    ResourceData, ResourceEvent, ResourceEventKind, ResourceId, ResourceKind, ResourceLocator,
    ResourceMutationBatch, ResourceMutationOperation, ResourceMutationReceipt, ResourceRecord,
    ResourceRegistry, ResourceRegistryError, ResourceResult, ResourceState, RuntimeResourceState,
};

use super::resource_manager::{ResourceAuthority, ResourceManager};
use super::revision::next_ready_revision;
use super::runtime_slot::ResourceRuntimeSlot;

enum PayloadMutation {
    Keep,
    Replace(Arc<dyn ResourceData>),
    Remove,
}

// Keep repeated-operation batches from eagerly reserving O(N) staged memory.
const MAX_PREFLIGHT_INITIAL_CAPACITY: usize = 64;

fn preflight_initial_capacity(operation_count: usize) -> usize {
    operation_count.min(MAX_PREFLIGHT_INITIAL_CAPACITY)
}

struct StagedResource {
    id: ResourceId,
    before: Option<ResourceRecord>,
    record: Option<ResourceRecord>,
    removed_record_baseline: Option<ResourceRecord>,
    identity_kind: Option<ResourceKind>,
    authorized_locator: Option<ResourceLocator>,
    payload: PayloadMutation,
    runtime_state: Option<RuntimeResourceState>,
    reload_failed: bool,
}

impl StagedResource {
    fn new(id: ResourceId, before: Option<ResourceRecord>) -> Self {
        let identity_kind = before.as_ref().map(|record| record.kind);
        let authorized_locator = before.as_ref().map(|record| record.primary_locator.clone());
        Self {
            id,
            record: before.clone(),
            before,
            removed_record_baseline: None,
            identity_kind,
            authorized_locator,
            payload: PayloadMutation::Keep,
            runtime_state: None,
            reload_failed: false,
        }
    }

    fn establish_identity(&mut self, record: &ResourceRecord) {
        self.identity_kind.get_or_insert(record.kind);
        self.authorized_locator
            .get_or_insert_with(|| record.primary_locator.clone());
    }
}

struct StagedResources {
    index_by_id: HashMap<ResourceId, usize>,
    entries: Vec<StagedResource>,
}

impl StagedResources {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            index_by_id: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
        }
    }

    fn get(&self, id: ResourceId) -> Option<&StagedResource> {
        self.index_by_id.get(&id).map(|index| {
            let entry = &self.entries[*index];
            debug_assert_eq!(entry.id, id);
            entry
        })
    }

    fn get_or_insert_with(
        &mut self,
        id: ResourceId,
        before: impl FnOnce() -> Option<ResourceRecord>,
    ) -> &mut StagedResource {
        if let Some(index) = self.index_by_id.get(&id).copied() {
            let entry = &mut self.entries[index];
            debug_assert_eq!(entry.id, id);
            return entry;
        }
        let index = self.entries.len();
        self.entries.push(StagedResource::new(id, before()));
        let previous = self.index_by_id.insert(id, index);
        debug_assert!(previous.is_none());
        &mut self.entries[index]
    }

    fn into_entries(self) -> Vec<StagedResource> {
        self.entries
    }
}

pub struct PreparedResourceMutation<'a> {
    manager: &'a ResourceManager,
    commit_serial: MutexGuard<'a, ()>,
    staged: StagedResources,
    events: Vec<ResourceEvent>,
    event_publish_permit: ResourceEventPublishPermit,
}

impl PreparedResourceMutation<'_> {
    pub fn commit(self) -> ResourceMutationReceipt {
        let receipt = {
            let mut authority = self.manager.lock_authority_write();
            apply_staged(&mut authority, self.staged, self.events.len())
        };
        self.manager
            .events
            .publish_permitted(self.event_publish_permit, self.events);
        drop(self.commit_serial);
        receipt
    }
}

impl ResourceManager {
    pub fn commit(&self, batch: ResourceMutationBatch) -> ResourceResult<ResourceMutationReceipt> {
        Ok(self.prepare_commit(batch)?.commit())
    }

    pub(crate) fn prepare_commit(
        &self,
        batch: ResourceMutationBatch,
    ) -> ResourceResult<PreparedResourceMutation<'_>> {
        let commit_serial = self.lock_commit_serial();
        let staged = {
            let authority = self.lock_authority_read();
            preflight(&authority.registry, batch.operations())?
        };
        let events = events_for_staged(&staged);
        let event_publish_permit = self.events.prepare_publish(events.len())?;
        Ok(PreparedResourceMutation {
            manager: self,
            commit_serial,
            staged,
            events,
            event_publish_permit,
        })
    }
}

fn preflight(
    registry: &ResourceRegistry,
    operations: Vec<ResourceMutationOperation>,
) -> ResourceResult<StagedResources> {
    let initial_capacity = preflight_initial_capacity(operations.len());
    let mut staged = StagedResources::with_capacity(initial_capacity);
    let mut locators =
        HashMap::<ResourceLocator, Option<ResourceId>>::with_capacity(initial_capacity);

    for operation in operations {
        match operation {
            ResourceMutationOperation::UpsertLazy(mut record) => {
                validate_upsert(registry, &staged, &locators, &record)?;
                let previous = upsert_baseline_record(registry, &staged, record.id);
                let invalidate_payload = match previous.as_ref() {
                    Some(previous) => {
                        let previous_state = previous.state;
                        let previous_revision = previous.revision;
                        if record.state == ResourceState::Ready {
                            record.revision = next_ready_revision(previous, &record)?;
                        } else {
                            record.revision = previous_revision;
                        }
                        if record.state == ResourceState::Ready {
                            record.revision != previous_revision
                                || previous_state != ResourceState::Ready
                        } else {
                            previous_state == ResourceState::Ready
                        }
                    }
                    None => {
                        if record.state == ResourceState::Ready && record.revision == 0 {
                            record.revision = 1;
                        }
                        true
                    }
                };
                locators.insert(record.primary_locator.clone(), Some(record.id));
                let entry = staged_entry(&mut staged, registry, record.id);
                entry.establish_identity(&record);
                entry.record = Some(record.clone());
                if invalidate_payload {
                    entry.payload = PayloadMutation::Remove;
                }
                entry.runtime_state = match record.state {
                    ResourceState::Error => Some(RuntimeResourceState::Error),
                    ResourceState::Reloading => Some(RuntimeResourceState::Reloading),
                    ResourceState::Pending => Some(RuntimeResourceState::Unloaded),
                    ResourceState::Ready if invalidate_payload => {
                        Some(RuntimeResourceState::Unloaded)
                    }
                    ResourceState::Ready => None,
                };
                entry.reload_failed = false;
            }
            ResourceMutationOperation::UpsertReady {
                mut record,
                payload,
                recover_from_error,
            } => {
                validate_upsert(registry, &staged, &locators, &record)?;
                let previous = upsert_baseline_record(registry, &staged, record.id);
                if previous
                    .as_ref()
                    .is_some_and(|previous| previous.state == ResourceState::Error)
                    && !recover_from_error
                {
                    return Err(invalid_transition(
                        record.id,
                        ResourceState::Error,
                        ResourceState::Ready,
                    ));
                }
                record.state = ResourceState::Ready;
                record.revision = match previous.as_ref() {
                    Some(previous) => next_ready_revision(previous, &record)?,
                    None => 1,
                };
                locators.insert(record.primary_locator.clone(), Some(record.id));
                let entry = staged_entry(&mut staged, registry, record.id);
                entry.establish_identity(&record);
                entry.record = Some(record);
                entry.payload = PayloadMutation::Replace(payload);
                entry.runtime_state = Some(RuntimeResourceState::Loaded);
                entry.reload_failed = false;
            }
            ResourceMutationOperation::StorePayload {
                id,
                expected_revision,
                payload,
            } => {
                let Some(record) = effective_record(registry, &staged, id) else {
                    return Err(ResourceRegistryError::RevisionConflict {
                        id: id.to_string(),
                        expected_revision,
                        actual_revision: None,
                    });
                };
                if record.revision != expected_revision {
                    return Err(ResourceRegistryError::RevisionConflict {
                        id: id.to_string(),
                        expected_revision,
                        actual_revision: Some(record.revision),
                    });
                }
                if record.state != ResourceState::Ready {
                    return Err(invalid_transition(id, record.state, ResourceState::Ready));
                }
                let entry = staged_entry(&mut staged, registry, id);
                entry.payload = PayloadMutation::Replace(payload);
                entry.runtime_state = Some(RuntimeResourceState::Loaded);
            }
            ResourceMutationOperation::StartReload { id, diagnostics } => {
                let Some(mut record) = effective_record(registry, &staged, id) else {
                    return Err(ResourceRegistryError::MissingRecordForId { id: id.to_string() });
                };
                if !matches!(
                    record.state,
                    ResourceState::Ready | ResourceState::Reloading | ResourceState::Error
                ) {
                    return Err(invalid_transition(
                        id,
                        record.state,
                        ResourceState::Reloading,
                    ));
                }
                record.state = ResourceState::Reloading;
                record.diagnostics = diagnostics;
                let entry = staged_entry(&mut staged, registry, id);
                entry.record = Some(record);
                entry.runtime_state = Some(RuntimeResourceState::Reloading);
                entry.reload_failed = false;
            }
            ResourceMutationOperation::FailReload { id, diagnostics } => {
                let Some(mut record) = effective_record(registry, &staged, id) else {
                    return Err(ResourceRegistryError::MissingRecordForId { id: id.to_string() });
                };
                if !matches!(
                    record.state,
                    ResourceState::Pending | ResourceState::Reloading | ResourceState::Error
                ) {
                    return Err(invalid_transition(id, record.state, ResourceState::Error));
                }
                record.state = ResourceState::Error;
                record.diagnostics = diagnostics;
                let entry = staged_entry(&mut staged, registry, id);
                entry.record = Some(record);
                entry.runtime_state = Some(RuntimeResourceState::Error);
                entry.reload_failed = true;
            }
            ResourceMutationOperation::Rename { from, to } => {
                let Some(id) = effective_id(registry, &locators, &from) else {
                    return Err(ResourceRegistryError::MissingRecordForLocator {
                        locator: from.to_string(),
                    });
                };
                if let Some(existing_id) = effective_id(registry, &locators, &to) {
                    if existing_id != id {
                        return Err(locator_occupied(&to, existing_id, id));
                    }
                }
                let Some(mut record) = effective_record(registry, &staged, id) else {
                    return Err(ResourceRegistryError::MissingRecordForId { id: id.to_string() });
                };
                locators.insert(from, None);
                locators.insert(to.clone(), Some(id));
                record.primary_locator = to;
                let entry = staged_entry(&mut staged, registry, id);
                entry.authorized_locator = Some(record.primary_locator.clone());
                entry.record = Some(record);
                entry.reload_failed = false;
            }
            ResourceMutationOperation::Remove {
                locator,
                expected_kind,
            } => {
                let Some(id) = effective_id(registry, &locators, &locator) else {
                    continue;
                };
                let current = effective_record(registry, &staged, id)
                    .expect("a locator owner has an effective record");
                if let Some(expected_kind) = expected_kind {
                    if current.kind != expected_kind {
                        return Err(ResourceRegistryError::KindConflict {
                            id: id.to_string(),
                            current_kind: current.kind,
                            requested_kind: expected_kind,
                        });
                    }
                }
                let entry = staged_entry(&mut staged, registry, id);
                entry.removed_record_baseline =
                    (entry.before.as_ref() != Some(&current)).then_some(current);
                entry.record = None;
                entry.payload = PayloadMutation::Remove;
                entry.runtime_state = None;
                entry.reload_failed = false;
                locators.insert(locator, None);
            }
        }
    }
    Ok(staged)
}

fn validate_upsert(
    registry: &ResourceRegistry,
    staged: &StagedResources,
    locators: &HashMap<ResourceLocator, Option<ResourceId>>,
    requested: &ResourceRecord,
) -> ResourceResult<()> {
    if let Some(identity) = staged.get(requested.id) {
        if let Some(current_kind) = identity.identity_kind {
            if current_kind != requested.kind {
                return Err(ResourceRegistryError::KindConflict {
                    id: requested.id.to_string(),
                    current_kind,
                    requested_kind: requested.kind,
                });
            }
        }
        if let Some(current_locator) = &identity.authorized_locator {
            if current_locator != &requested.primary_locator {
                return Err(ResourceRegistryError::ExplicitRenameRequired {
                    id: requested.id.to_string(),
                    current_locator: current_locator.to_string(),
                    requested_locator: requested.primary_locator.to_string(),
                });
            }
        }
    } else if let Some(current) = registry.get(requested.id) {
        if current.kind != requested.kind {
            return Err(ResourceRegistryError::KindConflict {
                id: requested.id.to_string(),
                current_kind: current.kind,
                requested_kind: requested.kind,
            });
        }
        if current.primary_locator != requested.primary_locator {
            return Err(ResourceRegistryError::ExplicitRenameRequired {
                id: requested.id.to_string(),
                current_locator: current.primary_locator.to_string(),
                requested_locator: requested.primary_locator.to_string(),
            });
        }
    }
    if let Some(existing_id) = effective_id(registry, locators, &requested.primary_locator) {
        if existing_id != requested.id {
            return Err(locator_occupied(
                &requested.primary_locator,
                existing_id,
                requested.id,
            ));
        }
    }
    Ok(())
}

fn effective_record(
    registry: &ResourceRegistry,
    staged: &StagedResources,
    id: ResourceId,
) -> Option<ResourceRecord> {
    staged
        .get(id)
        .map(|entry| entry.record.clone())
        .unwrap_or_else(|| registry.get(id).cloned())
}

fn upsert_baseline_record(
    registry: &ResourceRegistry,
    staged: &StagedResources,
    id: ResourceId,
) -> Option<ResourceRecord> {
    effective_record(registry, staged, id).or_else(|| {
        staged.get(id).and_then(|entry| {
            entry
                .removed_record_baseline
                .as_ref()
                .or(entry.before.as_ref())
                .cloned()
        })
    })
}

fn effective_id(
    registry: &ResourceRegistry,
    staged: &HashMap<ResourceLocator, Option<ResourceId>>,
    locator: &ResourceLocator,
) -> Option<ResourceId> {
    staged
        .get(locator)
        .copied()
        .unwrap_or_else(|| registry.id_for_locator(locator))
}

fn staged_entry<'a>(
    staged: &'a mut StagedResources,
    registry: &ResourceRegistry,
    id: ResourceId,
) -> &'a mut StagedResource {
    staged.get_or_insert_with(id, || registry.get(id).cloned())
}

fn apply_staged(
    authority: &mut ResourceAuthority,
    staged: StagedResources,
    published_event_count: usize,
) -> ResourceMutationReceipt {
    let mut staged = staged.into_entries();
    staged.retain(|entry| entry.before.is_some() || entry.record.is_some());

    for entry in &staged {
        if entry.before.is_some() && entry.before != entry.record {
            authority
                .registry
                .remove_by_id(entry.before.as_ref().unwrap().id);
        }
    }
    for entry in &staged {
        if entry.before != entry.record {
            if let Some(record) = &entry.record {
                authority.registry.insert_unchecked(record.clone());
            }
        }
    }

    let removed_ids = staged.iter().filter_map(|entry| {
        (entry.before.is_some() && entry.record.is_none())
            .then(|| entry.before.as_ref().unwrap().id)
    });
    let changed_records = staged
        .iter()
        .filter(|entry| entry.before != entry.record)
        .filter_map(|entry| entry.record.as_ref());
    authority
        .management
        .apply_delta(removed_ids, changed_records);

    for entry in &staged {
        let id = entry.id;
        if entry.record.is_none() {
            authority.payloads.remove(&id);
            authority.runtime.remove(&id);
            continue;
        }
        match &entry.payload {
            PayloadMutation::Keep => {
                if let Some(state) = entry.runtime_state {
                    let slot = authority.runtime.entry(id).or_default();
                    slot.state = state;
                }
            }
            PayloadMutation::Replace(payload) => {
                authority.payloads.insert(id, payload.clone());
                authority.runtime.insert(
                    id,
                    ResourceRuntimeSlot {
                        state: entry.runtime_state.unwrap_or(RuntimeResourceState::Loaded),
                        ..ResourceRuntimeSlot::default()
                    },
                );
            }
            PayloadMutation::Remove => {
                authority.payloads.remove(&id);
                authority.runtime.insert(
                    id,
                    ResourceRuntimeSlot {
                        state: entry
                            .runtime_state
                            .unwrap_or(RuntimeResourceState::Unloaded),
                        ..ResourceRuntimeSlot::default()
                    },
                );
            }
        }
    }

    authority.refresh_readiness_many(staged.iter().filter_map(|entry| {
        entry
            .record
            .as_ref()
            .or(entry.before.as_ref())
            .map(|record| record.id)
    }));

    let mut records = HashMap::with_capacity(staged.len());
    let mut removed = HashMap::with_capacity(staged.len());
    for entry in staged {
        if let Some(record) = entry.record {
            records.insert(record.id, record);
        } else if let Some(record) = entry.before {
            removed.insert(record.id, record);
        }
    }
    let projections = crate::ResourceProjectionSnapshot::new(
        authority.management.generation(),
        authority.readiness.generation(),
    );
    ResourceMutationReceipt::new(records, removed, projections, published_event_count)
}

fn events_for_staged(staged: &StagedResources) -> Vec<ResourceEvent> {
    staged
        .entries
        .iter()
        .filter_map(event_for_staged_resource)
        .collect()
}

fn event_for_staged_resource(entry: &StagedResource) -> Option<ResourceEvent> {
    let (kind, record, previous_locator) = match (&entry.before, &entry.record) {
        (None, Some(record)) => (ResourceEventKind::Added, record, None),
        (Some(previous), None) => (ResourceEventKind::Removed, previous, None),
        (Some(previous), Some(record)) if previous != record => {
            let locator_changed = previous.primary_locator != record.primary_locator;
            let kind = if entry.reload_failed && record.state == ResourceState::Error {
                ResourceEventKind::ReloadFailed
            } else if locator_changed {
                ResourceEventKind::Renamed
            } else {
                ResourceEventKind::Updated
            };
            (
                kind,
                record,
                locator_changed.then(|| previous.primary_locator.clone()),
            )
        }
        _ => return None,
    };
    Some(ResourceEvent {
        kind,
        resource_kind: record.kind,
        id: record.id,
        locator: Some(record.primary_locator.clone()),
        previous_locator,
        revision: record.revision,
    })
}

fn locator_occupied(
    locator: &ResourceLocator,
    existing_id: ResourceId,
    requested_id: ResourceId,
) -> ResourceRegistryError {
    ResourceRegistryError::LocatorOccupied {
        locator: locator.to_string(),
        existing_id: existing_id.to_string(),
        requested_id: requested_id.to_string(),
    }
}

fn invalid_transition(
    id: ResourceId,
    current_state: ResourceState,
    requested_state: ResourceState,
) -> ResourceRegistryError {
    ResourceRegistryError::InvalidStateTransition {
        id: id.to_string(),
        current_state,
        requested_state,
    }
}

#[cfg(test)]
#[path = "commit/optimization_tests.rs"]
mod optimization_tests;
