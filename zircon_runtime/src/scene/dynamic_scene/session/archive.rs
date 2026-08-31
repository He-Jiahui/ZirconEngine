use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use super::artifact::RuntimeSessionArchiveSealState;
use super::error::RuntimeSessionArchiveError;
use super::metadata::RuntimeSessionMetadata;
use super::slot::RuntimeSessionSlot;

mod secondary_index;

use self::secondary_index::{index_secondary_entries, remove_secondary_entries};

pub const RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION: u32 = 1;

static NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION: AtomicU64 = AtomicU64::new(1);
static NEXT_RUNTIME_SESSION_ARCHIVE_LINEAGE: AtomicU64 = AtomicU64::new(1);

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RuntimeSessionArchivePayload {
    pub(crate) format_version: u32,
    slots: Vec<RuntimeSessionSlot>,
    slot_indices: BTreeMap<String, usize>,
    updated_slot_indices: BTreeMap<RuntimeSessionSlotUpdateKey, usize>,
    tag_slot_indices: BTreeMap<String, BTreeMap<RuntimeSessionSlotUpdateKey, usize>>,
}

type RuntimeSessionSlotUpdateKey = (u64, String);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeSessionArchiveWirePayload {
    pub(super) format_version: u32,
    #[serde(default)]
    pub(super) slots: Vec<RuntimeSessionSlot>,
}

impl From<RuntimeSessionArchiveWirePayload> for RuntimeSessionArchivePayload {
    fn from(wire: RuntimeSessionArchiveWirePayload) -> Self {
        Self::new(wire.format_version, wire.slots)
    }
}

impl RuntimeSessionArchivePayload {
    pub(crate) fn new(format_version: u32, slots: Vec<RuntimeSessionSlot>) -> Self {
        let mut payload = Self {
            format_version,
            slots,
            slot_indices: BTreeMap::new(),
            updated_slot_indices: BTreeMap::new(),
            tag_slot_indices: BTreeMap::new(),
        };
        payload.rebuild_slot_indexes();
        payload
    }

    fn rebuild_slot_indexes(&mut self) {
        self.slot_indices.clear();
        self.updated_slot_indices.clear();
        self.tag_slot_indices.clear();
        for slot_index in 0..self.slots.len() {
            let slot_id = self.slots[slot_index].slot_id.clone();
            self.slot_indices.insert(slot_id, slot_index);
            self.index_slot_secondary_entries(slot_index);
        }
    }

    fn insert_slot(&mut self, slot: RuntimeSessionSlot) {
        let slot_index = self.slots.len();
        let slot_id = slot.slot_id.clone();
        self.slots.push(slot);
        self.slot_indices.insert(slot_id, slot_index);
        self.index_slot_secondary_entries(slot_index);
    }

    fn replace_slot(&mut self, slot_index: usize, slot: RuntimeSessionSlot) -> RuntimeSessionSlot {
        debug_assert_eq!(self.slots[slot_index].slot_id, slot.slot_id);
        let replaced = std::mem::replace(&mut self.slots[slot_index], slot);
        self.remove_slot_secondary_entries(&replaced);
        self.index_slot_secondary_entries(slot_index);
        replaced
    }

    fn remove_slot(&mut self, slot_index: usize) -> RuntimeSessionSlot {
        let removed = self.slots.swap_remove(slot_index);
        self.remove_slot_secondary_entries(&removed);
        self.slot_indices.remove(&removed.slot_id);
        if let Some(moved_slot) = self.slots.get(slot_index) {
            self.slot_indices
                .insert(moved_slot.slot_id.clone(), slot_index);
            self.index_slot_secondary_entries(slot_index);
        }
        removed
    }

    fn replace_slot_metadata(&mut self, slot_index: usize, metadata: RuntimeSessionMetadata) {
        let previous_metadata = std::mem::replace(&mut self.slots[slot_index].metadata, metadata);
        let previous_update_key = (
            previous_metadata.updated_at_unix_millis.unwrap_or(0),
            self.slots[slot_index].slot_id.clone(),
        );
        remove_secondary_entries(
            &mut self.updated_slot_indices,
            &mut self.tag_slot_indices,
            &previous_update_key,
            &previous_metadata.tags,
        );
        self.index_slot_secondary_entries(slot_index);
    }

    fn index_slot_secondary_entries(&mut self, slot_index: usize) {
        let slot = &self.slots[slot_index];
        index_secondary_entries(
            &mut self.updated_slot_indices,
            &mut self.tag_slot_indices,
            slot_index,
            slot_update_key(slot),
            &slot.metadata.tags,
        );
    }

    fn remove_slot_secondary_entries(&mut self, slot: &RuntimeSessionSlot) {
        let update_key = slot_update_key(slot);
        remove_secondary_entries(
            &mut self.updated_slot_indices,
            &mut self.tag_slot_indices,
            &update_key,
            &slot.metadata.tags,
        );
    }

    fn indexed_slot(&self, slot_id: &str) -> Option<&RuntimeSessionSlot> {
        self.slot_indices
            .get(slot_id)
            .and_then(|index| self.slots.get(*index))
    }

    fn indexed_slot_index(&self, slot_id: &str) -> Option<usize> {
        self.slot_indices.get(slot_id).copied()
    }

    fn indexed_tag_slots(&self, tag: &str) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.tag_slot_indices
            .get(tag)
            .into_iter()
            .flat_map(|slot_indices| slot_indices.values())
            .filter_map(|index| self.slots.get(*index))
    }

    fn indexed_latest_slot(&self) -> Option<&RuntimeSessionSlot> {
        self.updated_slot_indices
            .last_key_value()
            .and_then(|(_, index)| self.slots.get(*index))
    }

    fn indexed_oldest_slot(&self) -> Option<&RuntimeSessionSlot> {
        self.updated_slot_indices
            .first_key_value()
            .and_then(|(_, index)| self.slots.get(*index))
    }

    fn indexed_slots_by_update(&self) -> impl DoubleEndedIterator<Item = &RuntimeSessionSlot> {
        self.updated_slot_indices
            .values()
            .filter_map(|index| self.slots.get(*index))
    }

    fn indexed_latest_tag_slot(&self, tag: &str) -> Option<&RuntimeSessionSlot> {
        self.tag_slot_indices
            .get(tag)
            .and_then(|slot_indices| slot_indices.last_key_value())
            .and_then(|(_, index)| self.slots.get(*index))
    }

    fn indexed_oldest_tag_slot(&self, tag: &str) -> Option<&RuntimeSessionSlot> {
        self.tag_slot_indices
            .get(tag)
            .and_then(|slot_indices| slot_indices.first_key_value())
            .and_then(|(_, index)| self.slots.get(*index))
    }

    pub(super) fn canonical_slots(&self) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.slot_indices
            .values()
            .filter_map(|slot_index| self.slots.get(*slot_index))
    }

    pub(super) fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    fn dense_slots(&self) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.slots.iter()
    }
}

fn slot_update_key(slot: &RuntimeSessionSlot) -> RuntimeSessionSlotUpdateKey {
    (
        slot.metadata.updated_at_unix_millis.unwrap_or(0),
        slot.slot_id.clone(),
    )
}

impl Serialize for RuntimeSessionArchivePayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RuntimeSessionArchivePayload", 2)?;
        state.serialize_field("format_version", &self.format_version)?;
        let slots = self.canonical_slots().collect::<Vec<_>>();
        state.serialize_field("slots", &slots)?;
        state.end()
    }
}

impl PartialEq for RuntimeSessionArchivePayload {
    fn eq(&self, other: &Self) -> bool {
        self.format_version == other.format_version
            && self.canonical_slots().eq(other.canonical_slots())
    }
}

pub struct RuntimeSessionArchive {
    payload: Arc<RuntimeSessionArchivePayload>,
    pub(super) state: Arc<RuntimeSessionArchiveGenerationState>,
}

pub(super) struct RuntimeSessionArchiveGenerationState {
    pub(super) generation: u64,
    pub(super) lineage: u64,
    pub(super) revision: u64,
    pub(super) publication: Arc<RuntimeSessionArchivePublicationState>,
    pub(super) counters: Arc<RuntimeSessionArchiveStageCounters>,
    pub(super) validation_gate: Mutex<()>,
    pub(super) sealed: Mutex<RuntimeSessionArchiveSealState>,
}

#[derive(Debug)]
pub(super) struct RuntimeSessionArchivePublicationState {
    next_revision: AtomicU64,
    pub(super) published_revision: AtomicU64,
    pub(super) gate: Mutex<()>,
}

impl Default for RuntimeSessionArchivePublicationState {
    fn default() -> Self {
        Self {
            next_revision: AtomicU64::new(1),
            published_revision: AtomicU64::new(0),
            gate: Mutex::new(()),
        }
    }
}

impl RuntimeSessionArchivePublicationState {
    fn allocate_revision(&self) -> u64 {
        self.next_revision
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |revision| {
                revision.checked_add(1)
            })
            .expect("runtime session archive lineage revision exhausted")
            + 1
    }
}

#[derive(Debug, Default)]
pub(super) struct RuntimeSessionArchiveStageCounters {
    pub(super) capture_count: AtomicUsize,
    pub(super) normalize_count: AtomicUsize,
    pub(super) validate_count: AtomicUsize,
    pub(super) serialize_count: AtomicUsize,
    pub(super) internal_json_roundtrip_count: AtomicUsize,
    pub(super) normalized: AtomicBool,
    pub(super) validated: AtomicBool,
}

impl RuntimeSessionArchive {
    pub(in crate::scene::dynamic_scene::session) fn from_payload(
        format_version: u32,
        slots: Vec<RuntimeSessionSlot>,
    ) -> Self {
        Self::from_deserialized_payload(RuntimeSessionArchivePayload::new(format_version, slots))
    }

    pub(super) fn from_deserialized_payload(payload: RuntimeSessionArchivePayload) -> Self {
        Self {
            payload: Arc::new(payload),
            state: new_lineage_state(),
        }
    }

    pub fn generation(&self) -> u64 {
        self.state.generation
    }

    pub fn revision(&self) -> u64 {
        self.state.revision
    }

    pub(in crate::scene::dynamic_scene::session) fn record_capture(&self) {
        self.state
            .counters
            .capture_count
            .store(1, Ordering::Release);
    }

    pub(in crate::scene::dynamic_scene::session) fn record_normalized(&self) {
        if !self.state.counters.normalized.swap(true, Ordering::AcqRel) {
            self.state
                .counters
                .normalize_count
                .fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(in crate::scene::dynamic_scene::session) fn record_validated(&self) {
        if !self.state.counters.validated.swap(true, Ordering::AcqRel) {
            self.state
                .counters
                .validate_count
                .fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(in crate::scene::dynamic_scene::session) fn has_current_validation_ticket(&self) -> bool {
        self.state.counters.validated.load(Ordering::Acquire)
    }

    pub(super) fn payload_arc(&self) -> Arc<RuntimeSessionArchivePayload> {
        Arc::clone(&self.payload)
    }

    fn payload_mut(&mut self) -> &mut RuntimeSessionArchivePayload {
        self.state = next_revision_state(&self.state);
        Arc::make_mut(&mut self.payload)
    }

    pub(in crate::scene::dynamic_scene::session) fn normalize_slot_metadata_rows(&mut self) {
        let payload = self.payload_mut();
        for slot in &mut payload.slots {
            slot.metadata.normalize();
        }
        payload.rebuild_slot_indexes();
    }

    pub(in crate::scene::dynamic_scene::session) fn iter_canonical_slots(
        &self,
    ) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.payload.canonical_slots()
    }

    pub(in crate::scene::dynamic_scene::session) fn iter_dense_slot_rows(
        &self,
    ) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.payload.dense_slots()
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_slot(
        &self,
        slot_id: &str,
    ) -> Option<&RuntimeSessionSlot> {
        self.payload.indexed_slot(slot_id)
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_slot_index(
        &self,
        slot_id: &str,
    ) -> Option<usize> {
        self.payload.indexed_slot_index(slot_id)
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_tag_slots(
        &self,
        tag: &str,
    ) -> impl Iterator<Item = &RuntimeSessionSlot> {
        self.payload.indexed_tag_slots(tag)
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_latest_slot(
        &self,
    ) -> Option<&RuntimeSessionSlot> {
        self.payload.indexed_latest_slot()
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_oldest_slot(
        &self,
    ) -> Option<&RuntimeSessionSlot> {
        self.payload.indexed_oldest_slot()
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_slots_by_update(
        &self,
    ) -> impl DoubleEndedIterator<Item = &RuntimeSessionSlot> {
        self.payload.indexed_slots_by_update()
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_latest_tag_slot(
        &self,
        tag: &str,
    ) -> Option<&RuntimeSessionSlot> {
        self.payload.indexed_latest_tag_slot(tag)
    }

    pub(in crate::scene::dynamic_scene::session) fn indexed_oldest_tag_slot(
        &self,
        tag: &str,
    ) -> Option<&RuntimeSessionSlot> {
        self.payload.indexed_oldest_tag_slot(tag)
    }

    pub(in crate::scene::dynamic_scene::session) fn replace_slot_metadata(
        &mut self,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> bool {
        let Some(slot_index) = self.indexed_slot_index(slot_id) else {
            return false;
        };
        let payload = self.payload_mut();
        payload.replace_slot_metadata(slot_index, metadata);
        true
    }

    pub(in crate::scene::dynamic_scene::session) fn remove_indexed_slot(
        &mut self,
        slot_id: &str,
    ) -> Option<RuntimeSessionSlot> {
        let slot_index = self.indexed_slot_index(slot_id)?;
        let payload = self.payload_mut();
        Some(payload.remove_slot(slot_index))
    }

    pub(in crate::scene::dynamic_scene::session) fn commit_staged_slot_rows<'slot>(
        &mut self,
        replacements: Vec<RuntimeSessionSlot>,
        inserts: Vec<RuntimeSessionSlot>,
        removed_slot_ids: impl IntoIterator<Item = &'slot str>,
    ) {
        let removed_slot_ids = removed_slot_ids.into_iter().collect::<BTreeSet<_>>();
        let payload = self.payload_mut();
        // Grow the dense rows before applying the batch. Secondary maps are
        // repaired by the same per-row primitives used by single-slot commits.
        payload.slots.reserve(inserts.len());
        for slot_id in &removed_slot_ids {
            if let Some(slot_index) = payload.indexed_slot_index(slot_id) {
                let _ = payload.remove_slot(slot_index);
            }
        }
        for slot in replacements {
            if removed_slot_ids.contains(slot.slot_id.as_str()) {
                continue;
            }
            if let Some(slot_index) = payload.indexed_slot_index(&slot.slot_id) {
                let _ = payload.replace_slot(slot_index, slot);
            } else {
                debug_assert!(false, "staged replacement must retain its target row");
            }
        }
        for slot in inserts {
            if removed_slot_ids.contains(slot.slot_id.as_str()) {
                continue;
            }
            debug_assert!(payload.indexed_slot(&slot.slot_id).is_none());
            payload.insert_slot(slot);
        }
    }

    pub(in crate::scene::dynamic_scene::session) fn commit_slot_upsert(
        &mut self,
        slot: RuntimeSessionSlot,
    ) {
        let slot_index = self.indexed_slot_index(&slot.slot_id);
        let payload = self.payload_mut();
        match slot_index {
            Some(slot_index) => {
                let _ = payload.replace_slot(slot_index, slot);
            }
            None => payload.insert_slot(slot),
        }
    }

    pub(in crate::scene::dynamic_scene::session) fn commit_slot_rename(
        &mut self,
        source_slot_id: &str,
        slot_index: usize,
        destination_slot_id: String,
    ) -> Result<(), RuntimeSessionArchiveError> {
        if source_slot_id != destination_slot_id
            && self.indexed_slot(&destination_slot_id).is_some()
        {
            return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                slot_id: destination_slot_id,
            });
        }

        let payload = self.payload_mut();
        let mut slot = payload.remove_slot(slot_index);
        debug_assert_eq!(slot.slot_id, source_slot_id);
        slot.slot_id = destination_slot_id;
        payload.insert_slot(slot);
        Ok(())
    }
}

impl Clone for RuntimeSessionArchive {
    fn clone(&self) -> Self {
        Self {
            payload: Arc::clone(&self.payload),
            state: Arc::clone(&self.state),
        }
    }
}

impl fmt::Debug for RuntimeSessionArchive {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeSessionArchive")
            .field("generation", &self.generation())
            .field("payload", &self.payload)
            .finish()
    }
}

impl PartialEq for RuntimeSessionArchive {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload
    }
}

impl Deref for RuntimeSessionArchive {
    type Target = RuntimeSessionArchivePayload;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

fn new_lineage_state() -> Arc<RuntimeSessionArchiveGenerationState> {
    Arc::new(RuntimeSessionArchiveGenerationState {
        generation: NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION.fetch_add(1, Ordering::AcqRel),
        lineage: NEXT_RUNTIME_SESSION_ARCHIVE_LINEAGE.fetch_add(1, Ordering::AcqRel),
        revision: 1,
        publication: Arc::new(RuntimeSessionArchivePublicationState::default()),
        counters: Arc::new(RuntimeSessionArchiveStageCounters::default()),
        validation_gate: Mutex::new(()),
        sealed: Mutex::new(RuntimeSessionArchiveSealState::Open),
    })
}

fn next_revision_state(
    current: &RuntimeSessionArchiveGenerationState,
) -> Arc<RuntimeSessionArchiveGenerationState> {
    Arc::new(RuntimeSessionArchiveGenerationState {
        generation: NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION.fetch_add(1, Ordering::AcqRel),
        lineage: current.lineage,
        revision: current.publication.allocate_revision(),
        publication: Arc::clone(&current.publication),
        counters: Arc::new(RuntimeSessionArchiveStageCounters::default()),
        validation_gate: Mutex::new(()),
        sealed: Mutex::new(RuntimeSessionArchiveSealState::Open),
    })
}

#[cfg(test)]
mod tests {
    use super::RuntimeSessionArchive;

    #[test]
    fn invalid_runtime_session_archive_generation_caches_its_seal_rejection() {
        let archive = RuntimeSessionArchive::from_payload(u32::MAX, Vec::new());

        let first = archive
            .sealed_artifact()
            .expect_err("unsupported format must reject the generation");
        let second = archive
            .sealed_artifact()
            .expect_err("deterministic validation rejection must be cached");

        assert_eq!(first.to_string(), second.to_string());
        let diagnostics = archive.artifact_diagnostics();
        assert_eq!(diagnostics.validate_count, 0);
        assert_eq!(diagnostics.serialize_count, 0);
    }
}
