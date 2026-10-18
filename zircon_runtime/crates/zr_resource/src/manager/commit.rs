//! 资源发布的唯一在线写入链：先完整预检，再一次更新权威状态，最后按批次发布失效事件。
//! 批次按首次触及资源的顺序折叠成最终变化；状态、载荷与两个只读投影在同一权威写锁下配对。

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
    retain_last_good: Option<bool>,
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
            retain_last_good: None,
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
    source_updates: Vec<(ResourceId, Vec<ResourceId>)>,
    expected_active: Vec<(ResourceId, Option<crate::ResourceActiveVersionKey>)>,
}

impl StagedResources {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            index_by_id: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            source_updates: Vec::new(),
            expected_active: Vec::new(),
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
}

/// 已预检但尚未发布的资源批次，并持有发布串行锁；项目导入可在磁盘批次提交后再发布内存状态。
/// 丢弃即可放弃，预检不会修改权威状态；持有期间不要重入同一管理器的提交，否则会等待自己持有的锁。
pub struct PreparedResourceMutation<'a> {
    manager: &'a ResourceManager,
    commit_serial: Option<MutexGuard<'a, ()>>,
    staged: Option<StagedResources>,
    // Retain the original payload owners across every preflight/permit error.
    operations: Option<Vec<ResourceMutationOperation>>,
    events: Vec<ResourceEvent>,
    event_publish_permit: Option<ResourceEventPublishPermit>,
    reservation_token: Arc<()>,
    reserved: Vec<(ResourceId, crate::ResourceActiveVersionKey)>,
}

impl PreparedResourceMutation<'_> {
    pub fn commit(mut self) -> ResourceMutationReceipt {
        let staged = self.staged.take().expect("prepared staged resources");
        let mut retired = Vec::new();
        let receipt = {
            let mut authority = self.manager.lock_authority_write();
            apply_staged(
                &mut authority,
                &staged,
                self.events.len(),
                &self.reservation_token,
                &self.reserved,
                &mut retired,
            )
        };
        self.reserved.clear();
        self.manager.events.publish_permitted(
            self.event_publish_permit
                .take()
                .expect("prepared event permit"),
            std::mem::take(&mut self.events),
        );
        drop(self.commit_serial.take());
        // A payload can contain nested leases. Never run its final Drop under either lock.
        drop(retired);
        drop(staged);
        drop(self.operations.take());
        receipt
    }
}

impl Drop for PreparedResourceMutation<'_> {
    fn drop(&mut self) {
        let mut retired = Vec::new();
        if !self.reserved.is_empty() {
            let mut authority = self.manager.lock_authority_write();
            clear_reservations(
                &mut authority,
                &self.reservation_token,
                &self.reserved,
                &mut retired,
            );
        }
        let staged = self.staged.take();
        let operations = self.operations.take();
        drop(self.commit_serial.take());
        drop(retired);
        drop(staged);
        drop(operations);
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
        // Rust drops locals in reverse order: this final owner must precede the serial guard.
        let operations = batch.operations();
        let commit_serial = self.lock_commit_serial();
        let staged = {
            let authority = self.lock_authority_read();
            preflight(&authority, &operations)?
        };
        let events = events_for_staged(&staged);
        let event_publish_permit = self.events.prepare_publish(events.len())?;
        let reservation_token = Arc::new(());
        let reserved = {
            let mut authority = self.lock_authority_write();
            // Lease release can evict between read preflight and reservation installation.
            validate_expected_active(&authority, &staged.expected_active)?;
            let reserved = staged
                .entries
                .iter()
                .filter_map(|entry| authority.active.key(entry.id).map(|key| (entry.id, key)))
                .collect::<Vec<_>>();
            for (id, key) in &reserved {
                let slot = authority
                    .runtime
                    .get_mut(id)
                    .expect("an active payload has a slot");
                slot.prepared = Some(super::runtime_slot::PreparedResourceReservation {
                    token: Arc::clone(&reservation_token),
                    key: key.clone(),
                    deferred_last_release: false,
                });
            }
            reserved
        };
        Ok(PreparedResourceMutation {
            manager: self,
            commit_serial: Some(commit_serial),
            staged: Some(staged),
            operations: Some(operations),
            events,
            event_publish_permit: Some(event_publish_permit),
            reservation_token,
            reserved,
        })
    }
}

fn preflight(
    authority: &ResourceAuthority,
    operations: &[ResourceMutationOperation],
) -> ResourceResult<StagedResources> {
    let registry = &authority.registry;
    let mut expanded = Vec::new();
    let mut reconciled = Vec::new();
    for operation in operations {
        match operation {
            ResourceMutationOperation::PublishSource(publication) => {
                expand_source_publication(authority, publication, &mut expanded)?;
            }
            ResourceMutationOperation::ReconcileSources(candidate_ids) => {
                let candidate_ids = candidate_ids
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>();
                for (owner, _) in authority.active.sources() {
                    if !candidate_ids.contains(&owner) {
                        let publication = crate::ResourceSourcePublication::capture(
                            owner,
                            Vec::new(),
                            Vec::new(),
                            crate::ResourceSourceOutcome::Removed,
                            &authority.active,
                        );
                        expand_source_publication(authority, &publication, &mut expanded)?;
                        reconciled.push(publication);
                    }
                }
            }
            operation => expanded.push(operation.clone()),
        }
    }
    let initial_capacity = preflight_initial_capacity(operations.len());
    let mut staged = StagedResources::with_capacity(initial_capacity);
    let mut locators =
        HashMap::<ResourceLocator, Option<ResourceId>>::with_capacity(initial_capacity);

    for operation in expanded {
        match operation {
            ResourceMutationOperation::PublishSource(_)
            | ResourceMutationOperation::ReconcileSources(_) => {
                unreachable!("source operations expanded")
            }
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
    for operation in operations {
        if let ResourceMutationOperation::PublishSource(publication) = operation {
            apply_source_overlay(authority, &mut staged, publication)?;
        }
    }
    for publication in &reconciled {
        apply_source_overlay(authority, &mut staged, publication)?;
    }
    // Both full-GC entry points declare candidate membership. A failed root may have no
    // catalog children, but its explicit source-unit active members remain last-good.
    if operations
        .iter()
        .any(|operation| matches!(operation, ResourceMutationOperation::ReconcileSources(_)))
    {
        let handled = operations
            .iter()
            .filter_map(|operation| match operation {
                ResourceMutationOperation::PublishSource(publication) => Some(publication.owner),
                _ => None,
            })
            .chain(reconciled.iter().map(|publication| publication.owner))
            .collect::<std::collections::HashSet<_>>();
        for (owner, members) in authority.active.sources() {
            // The complete source outcome already owns removal/retention. In particular,
            // generic candidate GC must not resurrect retired or successfully omitted members.
            if handled.contains(&owner) {
                continue;
            }
            if effective_record(registry, &staged, owner)
                .is_some_and(|record| record.state == ResourceState::Error)
                || members.iter().any(|id| {
                    authority
                        .runtime
                        .get(id)
                        .is_some_and(|slot| slot.retain_last_good)
                })
            {
                for id in members {
                    if authority.active.row(*id).is_some_and(|row| {
                        matches!(row.record.kind, ResourceKind::Model | ResourceKind::Mesh)
                    }) {
                        let entry = staged_entry(&mut staged, registry, *id);
                        entry.payload = PayloadMutation::Keep;
                        entry.retain_last_good = Some(true);
                        entry.runtime_state = Some(RuntimeResourceState::Error);
                        staged
                            .expected_active
                            .push((*id, authority.active.key(*id)));
                    }
                }
            }
        }
    }
    Ok(staged)
}

fn validate_expected_active(
    authority: &ResourceAuthority,
    expected: &[(ResourceId, Option<crate::ResourceActiveVersionKey>)],
) -> ResourceResult<()> {
    for (id, expected) in expected {
        if authority.active.key(*id).as_ref() != expected.as_ref() {
            return Err(ResourceRegistryError::ActiveVersionConflict { id: id.to_string() });
        }
    }
    Ok(())
}

fn source_error(owner: ResourceId, id: ResourceId) -> ResourceRegistryError {
    ResourceRegistryError::InvalidSourcePublication {
        owner: owner.to_string(),
        id: id.to_string(),
    }
}

fn expand_source_publication(
    authority: &ResourceAuthority,
    publication: &crate::ResourceSourcePublication,
    operations: &mut Vec<ResourceMutationOperation>,
) -> ResourceResult<()> {
    use crate::ResourceSourceOutcome;
    validate_expected_active(authority, &publication.expected_active)?;
    if publication.outcome == ResourceSourceOutcome::Removed && !publication.members.is_empty() {
        return Err(source_error(publication.owner, publication.owner));
    }
    let mut ids = std::collections::HashSet::new();
    for member in &publication.members {
        let id = member.record.id;
        if matches!(
            publication.outcome,
            ResourceSourceOutcome::CompleteSuccess | ResourceSourceOutcome::Unchanged
        ) && member.record.state != ResourceState::Ready
        {
            return Err(source_error(publication.owner, id));
        }
        if !ids.insert(id)
            || authority
                .active
                .source_owner(id)
                .is_some_and(|owner| owner != publication.owner)
        {
            return Err(source_error(publication.owner, id));
        }
        if let Some(active) = authority.active.row(id) {
            if active.record.kind != member.record.kind {
                return Err(ResourceRegistryError::KindConflict {
                    id: id.to_string(),
                    current_kind: active.record.kind,
                    requested_kind: member.record.kind,
                });
            }
        }
    }
    if publication.outcome != ResourceSourceOutcome::Removed && !ids.contains(&publication.owner) {
        return Err(source_error(publication.owner, publication.owner));
    }
    let mut previous = publication.previous_members.clone();
    previous.extend_from_slice(authority.active.source_members(publication.owner));
    previous.sort_unstable();
    previous.dedup();
    for id in previous {
        if publication.outcome == ResourceSourceOutcome::Removed || !ids.contains(&id) {
            if let Some(record) = authority.registry.get(id) {
                operations.push(ResourceMutationOperation::Remove {
                    locator: record.primary_locator.clone(),
                    expected_kind: None,
                });
            }
        }
    }
    if publication.outcome != ResourceSourceOutcome::Removed {
        for member in &publication.members {
            if let Some(payload) = &member.payload {
                if !matches!(member.record.kind, ResourceKind::Model | ResourceKind::Mesh) {
                    operations.push(ResourceMutationOperation::UpsertReady {
                        record: member.record.clone(),
                        payload: Arc::clone(payload),
                        recover_from_error: true,
                    });
                    continue;
                }
            }
            operations.push(ResourceMutationOperation::UpsertLazy(member.record.clone()));
        }
    }
    Ok(())
}

fn apply_source_overlay(
    authority: &ResourceAuthority,
    staged: &mut StagedResources,
    publication: &crate::ResourceSourcePublication,
) -> ResourceResult<()> {
    use crate::ResourceSourceOutcome;
    staged
        .expected_active
        .extend(publication.expected_active.iter().cloned());
    let mut members = publication
        .members
        .iter()
        .map(|member| member.record.id)
        .collect::<Vec<_>>();
    let mut previous = publication.previous_members.clone();
    previous.extend_from_slice(authority.active.source_members(publication.owner));
    previous.sort_unstable();
    previous.dedup();
    match publication.outcome {
        ResourceSourceOutcome::Failed => {
            members.extend(previous.iter().copied());
            for id in &previous {
                if authority.active.row(*id).is_some_and(|row| {
                    matches!(row.record.kind, ResourceKind::Model | ResourceKind::Mesh)
                }) {
                    let entry = staged_entry(staged, &authority.registry, *id);
                    entry.payload = PayloadMutation::Keep;
                    entry.retain_last_good = Some(true);
                    entry.runtime_state = Some(RuntimeResourceState::Error);
                }
            }
        }
        ResourceSourceOutcome::CompleteSuccess | ResourceSourceOutcome::Unchanged => {
            for member in &publication.members {
                if !matches!(member.record.kind, ResourceKind::Model | ResourceKind::Mesh) {
                    continue;
                }
                let entry = staged_entry(staged, &authority.registry, member.record.id);
                entry.retain_last_good = Some(false);
                if let Some(active) = authority.active.row(member.record.id) {
                    if let Some(payload) = &member.payload {
                        entry.payload = PayloadMutation::Replace(Arc::clone(payload));
                        entry.runtime_state = Some(RuntimeResourceState::Loaded);
                    } else if entry.record.as_ref().is_some_and(|record| {
                        record.state == ResourceState::Ready
                            && record.revision == active.record.revision
                    }) {
                        entry.payload = PayloadMutation::Keep;
                        entry.runtime_state = Some(RuntimeResourceState::Loaded);
                    } else {
                        return Err(source_error(publication.owner, member.record.id));
                    }
                } else {
                    // Owned importer bytes do not turn an existing cold requirement into residency.
                    entry.payload = PayloadMutation::Remove;
                    entry.runtime_state = Some(RuntimeResourceState::Unloaded);
                }
            }
        }
        ResourceSourceOutcome::Removed => members.clear(),
    }
    if publication.outcome != ResourceSourceOutcome::Failed {
        for id in previous {
            if !members.contains(&id) {
                let entry = staged_entry(staged, &authority.registry, id);
                entry.record = None;
                entry.payload = PayloadMutation::Remove;
                entry.retain_last_good = Some(false);
                entry.runtime_state = None;
            }
        }
    }
    members.sort_unstable();
    members.dedup();
    staged.source_updates.push((publication.owner, members));
    Ok(())
}

fn clear_reservations(
    authority: &mut ResourceAuthority,
    token: &Arc<()>,
    reserved: &[(ResourceId, crate::ResourceActiveVersionKey)],
    retired: &mut Vec<Arc<dyn ResourceData>>,
) {
    let mut removed = Vec::new();
    for (id, key) in reserved {
        if authority.active.key(*id).as_ref() != Some(key) {
            continue;
        }
        let preserve = authority.registry.get(*id).is_some_and(|record| {
            matches!(
                record.state,
                ResourceState::Reloading | ResourceState::Error
            )
        });
        let Some(slot) = authority.runtime.get_mut(id) else {
            continue;
        };
        let matches = slot.prepared.as_ref().is_some_and(|reservation| {
            Arc::ptr_eq(&reservation.token, token) && reservation.key == *key
        });
        if !matches {
            continue;
        }
        let reservation = slot.prepared.take().expect("matching reservation");
        if reservation.deferred_last_release
            && Arc::strong_count(&slot.lease_identity) == 1
            && !slot.retain_last_good
            && !preserve
        {
            slot.state = RuntimeResourceState::Unloaded;
            if let Some(payload) = authority.payloads.remove(id) {
                retired.push(payload);
            }
            removed.push((*id, None));
        }
    }
    let changed = removed.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    authority.active = crate::ResourceActiveGeneration::updated(&authority.active, removed, &[]);
    authority.refresh_readiness_many(changed);
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

// 删除后保留最后有效记录作为再登记基线，延续同批次版本与显式重载准入。
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

// 先移除发生变化的旧定位符，再写入最终记录，让同一批次的显式迁移共享最终命名空间。
// 投影和回执必须在权威写锁内取得，调用端才能按一次发布结果更新后续视图。
fn apply_staged(
    authority: &mut ResourceAuthority,
    staged: &StagedResources,
    published_event_count: usize,
    token: &Arc<()>,
    reserved: &[(ResourceId, crate::ResourceActiveVersionKey)],
    retired: &mut Vec<Arc<dyn ResourceData>>,
) -> ResourceMutationReceipt {
    for entry in &staged.entries {
        if entry.before.is_some() && entry.before != entry.record {
            authority.registry.remove_by_id(entry.id);
        }
    }
    for entry in &staged.entries {
        if entry.before != entry.record {
            if let Some(record) = &entry.record {
                authority.registry.insert_unchecked(record.clone());
            }
        }
    }
    authority.management.apply_delta(
        staged
            .entries
            .iter()
            .filter(|entry| entry.before.is_some() && entry.record.is_none())
            .map(|entry| entry.id),
        staged
            .entries
            .iter()
            .filter(|entry| entry.before != entry.record)
            .filter_map(|entry| entry.record.as_ref()),
    );
    let mut active_updates = Vec::new();
    for entry in &staged.entries {
        let id = entry.id;
        let remove = matches!(entry.payload, PayloadMutation::Remove)
            || (entry.record.is_none() && entry.retain_last_good != Some(true));
        if remove {
            if let Some(payload) = authority.payloads.remove(&id) {
                retired.push(payload);
            }
            if authority.active.row(id).is_some() {
                active_updates.push((id, None));
            }
            if entry.record.is_none() {
                authority.runtime.remove(&id);
            } else {
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
            continue;
        }
        match &entry.payload {
            PayloadMutation::Replace(payload) => {
                let record = entry
                    .record
                    .as_ref()
                    .expect("an installed payload has a record");
                if let Some(previous) = authority.payloads.insert(id, Arc::clone(payload)) {
                    retired.push(previous);
                }
                active_updates.push((
                    id,
                    Some(Arc::new(crate::ResourceActiveRow {
                        record: Arc::new(record.clone()),
                        key: crate::ResourceActiveVersionKey::new(id),
                    })),
                ));
                authority.runtime.insert(
                    id,
                    ResourceRuntimeSlot {
                        state: entry.runtime_state.unwrap_or(RuntimeResourceState::Loaded),
                        ..ResourceRuntimeSlot::default()
                    },
                );
            }
            PayloadMutation::Keep => {
                if let Some(slot) = authority.runtime.get_mut(&id) {
                    if let Some(retain) = entry.retain_last_good {
                        slot.retain_last_good = retain;
                    }
                    if let Some(state) = entry.runtime_state {
                        slot.state = state;
                    }
                } else if let Some(state) = entry.runtime_state {
                    authority.runtime.insert(
                        id,
                        ResourceRuntimeSlot {
                            state,
                            ..ResourceRuntimeSlot::default()
                        },
                    );
                }
                if entry.retain_last_good != Some(true) {
                    if let (Some(record), Some(active)) = (&entry.record, authority.active.row(id))
                    {
                        if record.state == ResourceState::Ready && record != active.record.as_ref()
                        {
                            active_updates.push((
                                id,
                                Some(Arc::new(crate::ResourceActiveRow {
                                    record: Arc::new(record.clone()),
                                    key: active.key.clone(),
                                })),
                            ));
                        }
                    }
                }
            }
            PayloadMutation::Remove => unreachable!("handled removal"),
        }
    }
    authority.active = crate::ResourceActiveGeneration::updated(
        &authority.active,
        active_updates,
        &staged.source_updates,
    );
    // Retention is installed before a last external release deferred by this reservation.
    clear_reservations(authority, token, reserved, retired);
    authority.refresh_readiness_many(staged.entries.iter().map(|entry| entry.id));
    let mut records = HashMap::with_capacity(staged.entries.len());
    let mut removed = HashMap::with_capacity(staged.entries.len());
    for entry in &staged.entries {
        if let Some(record) = &entry.record {
            records.insert(record.id, record.clone());
        } else if let Some(record) = &entry.before {
            removed.insert(record.id, record.clone());
        }
    }
    let projections = crate::ResourceProjectionSnapshot::new(
        authority.management.generation(),
        authority.readiness.generation(),
        Arc::clone(&authority.active),
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

// 事件只描述批次前后目录记录的净差异；仅替换载荷或改变驻留不会自动产生目录事件。
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
#[path = "commit/tests/optimization_tests.rs"]
mod optimization_tests;
