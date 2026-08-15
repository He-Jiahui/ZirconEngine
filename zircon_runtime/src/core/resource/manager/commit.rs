use std::collections::HashMap;
use std::sync::{Arc, MutexGuard};

use crate::core::resource::{
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

struct StagedResource {
    before: Option<ResourceRecord>,
    record: Option<ResourceRecord>,
    identity_kind: Option<ResourceKind>,
    authorized_locator: Option<ResourceLocator>,
    payload: PayloadMutation,
    runtime_state: Option<RuntimeResourceState>,
    reload_failed: bool,
    order: usize,
}

impl StagedResource {
    fn new(before: Option<ResourceRecord>, order: usize) -> Self {
        let identity_kind = before.as_ref().map(|record| record.kind);
        let authorized_locator = before.as_ref().map(|record| record.primary_locator.clone());
        Self {
            record: before.clone(),
            before,
            identity_kind,
            authorized_locator,
            payload: PayloadMutation::Keep,
            runtime_state: None,
            reload_failed: false,
            order,
        }
    }

    fn establish_identity(&mut self, record: &ResourceRecord) {
        self.identity_kind.get_or_insert(record.kind);
        self.authorized_locator
            .get_or_insert_with(|| record.primary_locator.clone());
    }
}

pub(crate) struct PreparedResourceMutation<'a> {
    manager: &'a ResourceManager,
    commit_serial: MutexGuard<'a, ()>,
    staged: HashMap<ResourceId, StagedResource>,
}

impl PreparedResourceMutation<'_> {
    pub(crate) fn commit(self) -> ResourceMutationReceipt {
        let events_and_receipt = {
            let mut authority = self.manager.lock_authority_write();
            apply_staged(&mut authority, self.staged)
        };
        let (events, receipt) = events_and_receipt;
        for event in events {
            self.manager.publish_event(event);
        }
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
        Ok(PreparedResourceMutation {
            manager: self,
            commit_serial,
            staged,
        })
    }
}

fn preflight(
    registry: &ResourceRegistry,
    operations: Vec<ResourceMutationOperation>,
) -> ResourceResult<HashMap<ResourceId, StagedResource>> {
    let mut staged = HashMap::<ResourceId, StagedResource>::new();
    let mut locators = HashMap::<ResourceLocator, Option<ResourceId>>::new();
    let mut next_order = 0;

    for operation in operations {
        match operation {
            ResourceMutationOperation::UpsertLazy(mut record) => {
                validate_upsert(registry, &staged, &locators, &record)?;
                let previous = effective_record(registry, &staged, record.id);
                let invalidate_payload = match previous.as_ref() {
                    Some(previous) => {
                        let previous_state = previous.state;
                        let previous_revision = previous.revision;
                        if record.state == ResourceState::Ready {
                            record.revision = next_ready_revision(previous, &record);
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
                let entry = staged_entry(&mut staged, registry, record.id, &mut next_order);
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
                let previous = effective_record(registry, &staged, record.id);
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
                    Some(previous) => next_ready_revision(previous, &record),
                    None => 1,
                };
                locators.insert(record.primary_locator.clone(), Some(record.id));
                let entry = staged_entry(&mut staged, registry, record.id, &mut next_order);
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
                let entry = staged_entry(&mut staged, registry, id, &mut next_order);
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
                let entry = staged_entry(&mut staged, registry, id, &mut next_order);
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
                let entry = staged_entry(&mut staged, registry, id, &mut next_order);
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
                let entry = staged_entry(&mut staged, registry, id, &mut next_order);
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
                let entry = staged_entry(&mut staged, registry, id, &mut next_order);
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
    staged: &HashMap<ResourceId, StagedResource>,
    locators: &HashMap<ResourceLocator, Option<ResourceId>>,
    requested: &ResourceRecord,
) -> ResourceResult<()> {
    if let Some(identity) = staged.get(&requested.id) {
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
    staged: &HashMap<ResourceId, StagedResource>,
    id: ResourceId,
) -> Option<ResourceRecord> {
    staged
        .get(&id)
        .map(|entry| entry.record.clone())
        .unwrap_or_else(|| registry.get(id).cloned())
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
    staged: &'a mut HashMap<ResourceId, StagedResource>,
    registry: &ResourceRegistry,
    id: ResourceId,
    next_order: &mut usize,
) -> &'a mut StagedResource {
    staged.entry(id).or_insert_with(|| {
        let order = *next_order;
        *next_order += 1;
        StagedResource::new(registry.get(id).cloned(), order)
    })
}

fn apply_staged(
    authority: &mut ResourceAuthority,
    staged: HashMap<ResourceId, StagedResource>,
) -> (Vec<ResourceEvent>, ResourceMutationReceipt) {
    let mut staged = staged.into_values().collect::<Vec<_>>();
    staged.retain(|entry| entry.before.is_some() || entry.record.is_some());
    staged.sort_by_key(|entry| entry.order);

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
        let id = entry
            .record
            .as_ref()
            .or(entry.before.as_ref())
            .expect("a staged resource has a before or after record")
            .id;
        if entry.record.is_none() {
            authority.payloads.remove(&id);
            authority.runtime.remove(&id);
            continue;
        }
        match &entry.payload {
            PayloadMutation::Keep => {
                if let Some(state) = entry.runtime_state {
                    let needs_token = authority
                        .runtime
                        .get(&id)
                        .is_none_or(|slot| slot.residency_token == 0);
                    let token = needs_token.then(|| authority.allocate_residency_token());
                    let slot = authority.runtime.entry(id).or_default();
                    if let Some(token) = token {
                        slot.residency_token = token;
                    }
                    slot.state = state;
                }
            }
            PayloadMutation::Replace(payload) => {
                let token = authority.allocate_residency_token();
                authority.payloads.insert(id, payload.clone());
                authority.runtime.insert(
                    id,
                    ResourceRuntimeSlot {
                        residency_token: token,
                        ref_count: 0,
                        state: entry.runtime_state.unwrap_or(RuntimeResourceState::Loaded),
                    },
                );
            }
            PayloadMutation::Remove => {
                let token = authority.allocate_residency_token();
                authority.payloads.remove(&id);
                authority.runtime.insert(
                    id,
                    ResourceRuntimeSlot {
                        residency_token: token,
                        ref_count: 0,
                        state: entry
                            .runtime_state
                            .unwrap_or(RuntimeResourceState::Unloaded),
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

    let events = staged
        .iter()
        .filter_map(event_for_staged_resource)
        .collect::<Vec<_>>();
    let mut records = HashMap::new();
    let mut removed = HashMap::new();
    for entry in staged {
        if let Some(record) = entry.record {
            records.insert(record.id, record);
        } else if let Some(record) = entry.before {
            removed.insert(record.id, record);
        }
    }
    let receipt = ResourceMutationReceipt::new(
        records,
        removed,
        authority.management.generation().sequence(),
        authority.readiness.generation().sequence(),
        events.len(),
    );
    (events, receipt)
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
