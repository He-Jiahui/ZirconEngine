use std::cmp::Ordering as ComparisonOrdering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::artifact::RuntimeSessionArchiveSealState;
use super::error::RuntimeSessionArchiveError;
use super::slot::RuntimeSessionSlot;

pub const RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION: u32 = 1;

static NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION: AtomicU64 = AtomicU64::new(1);
static NEXT_RUNTIME_SESSION_ARCHIVE_LINEAGE: AtomicU64 = AtomicU64::new(1);

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSessionArchivePayload {
    pub(crate) format_version: u32,
    #[serde(default)]
    pub(crate) slots: Vec<RuntimeSessionSlot>,
    #[serde(skip)]
    slot_indices: BTreeMap<String, usize>,
    #[serde(skip)]
    updated_slot_indices: Vec<usize>,
    #[serde(skip)]
    tag_slot_indices: BTreeMap<String, Vec<usize>>,
}

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
            updated_slot_indices: Vec::new(),
            tag_slot_indices: BTreeMap::new(),
        };
        payload.rebuild_slot_indexes();
        payload
    }

    fn rebuild_slot_indexes(&mut self) {
        self.slot_indices.clear();
        self.updated_slot_indices.clear();
        self.tag_slot_indices.clear();
        for (slot_index, slot) in self.slots.iter().enumerate() {
            self.slot_indices.insert(slot.slot_id.clone(), slot_index);
            for tag in &slot.metadata.tags {
                self.tag_slot_indices
                    .entry(tag.clone())
                    .or_default()
                    .push(slot_index);
            }
        }
        self.updated_slot_indices.extend(0..self.slots.len());
        let slots = &self.slots;
        self.updated_slot_indices
            .sort_by(|left, right| compare_slot_update_order(&slots[*left], &slots[*right]));
        for slot_indices in self.tag_slot_indices.values_mut() {
            slot_indices
                .sort_by(|left, right| compare_slot_update_order(&slots[*left], &slots[*right]));
        }
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
            .flatten()
            .filter_map(|index| self.slots.get(*index))
    }

    fn indexed_latest_slot(&self) -> Option<&RuntimeSessionSlot> {
        self.updated_slot_indices
            .last()
            .and_then(|index| self.slots.get(*index))
    }

    fn indexed_oldest_slot(&self) -> Option<&RuntimeSessionSlot> {
        self.updated_slot_indices
            .first()
            .and_then(|index| self.slots.get(*index))
    }

    fn indexed_latest_tag_slot(&self, tag: &str) -> Option<&RuntimeSessionSlot> {
        self.tag_slot_indices
            .get(tag)
            .and_then(|slot_indices| slot_indices.last())
            .and_then(|index| self.slots.get(*index))
    }

    fn indexed_oldest_tag_slot(&self, tag: &str) -> Option<&RuntimeSessionSlot> {
        self.tag_slot_indices
            .get(tag)
            .and_then(|slot_indices| slot_indices.first())
            .and_then(|index| self.slots.get(*index))
    }
}

fn compare_slot_update_order(
    left: &RuntimeSessionSlot,
    right: &RuntimeSessionSlot,
) -> ComparisonOrdering {
    left.metadata
        .updated_at_unix_millis
        .unwrap_or(0)
        .cmp(&right.metadata.updated_at_unix_millis.unwrap_or(0))
        .then_with(|| left.slot_id.cmp(&right.slot_id))
}

pub struct RuntimeSessionArchive {
    payload: Arc<RuntimeSessionArchivePayload>,
    pub(super) state: Arc<RuntimeSessionArchiveGenerationState>,
}

pub(super) struct RuntimeSessionArchiveGenerationState {
    pub(super) generation: u64,
    pub(super) lineage: u64,
    pub(super) revision: u64,
    pub(super) counters: Arc<RuntimeSessionArchiveStageCounters>,
    pub(super) sealed: Mutex<RuntimeSessionArchiveSealState>,
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

    pub(super) fn lineage(&self) -> u64 {
        self.state.lineage
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

    pub(super) fn payload_arc(&self) -> Arc<RuntimeSessionArchivePayload> {
        Arc::clone(&self.payload)
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

    pub(in crate::scene::dynamic_scene::session) fn rebuild_slot_indexes(&mut self) {
        Arc::make_mut(&mut self.payload).rebuild_slot_indexes();
    }

    pub(in crate::scene::dynamic_scene::session) fn commit_staged_slot_rows<'slot>(
        &mut self,
        replacements: Vec<(usize, RuntimeSessionSlot)>,
        inserts: Vec<RuntimeSessionSlot>,
        removed_slot_ids: impl IntoIterator<Item = &'slot str>,
    ) {
        let removed_slot_ids = removed_slot_ids.into_iter().collect::<BTreeSet<_>>();
        let payload = &mut **self;
        // Reserve before changing a row so allocation failure cannot expose a
        // partial batch through the archive's authoritative payload.
        payload.slots.reserve(inserts.len());
        for (slot_index, slot) in replacements {
            payload.slots[slot_index] = slot;
        }
        payload.slots.extend(inserts);
        if !removed_slot_ids.is_empty() {
            payload
                .slots
                .retain(|slot| !removed_slot_ids.contains(slot.slot_id.as_str()));
        }
        payload
            .slots
            .sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
        payload.rebuild_slot_indexes();
    }

    pub(in crate::scene::dynamic_scene::session) fn commit_slot_upsert(
        &mut self,
        slot: RuntimeSessionSlot,
    ) {
        let payload = &mut **self;
        match payload.indexed_slot_index(&slot.slot_id) {
            Some(slot_index) => payload.slots[slot_index] = slot,
            None => {
                let slot_index = payload
                    .slots
                    .binary_search_by(|existing| existing.slot_id.cmp(&slot.slot_id))
                    .unwrap_or_else(|slot_index| slot_index);
                payload.slots.insert(slot_index, slot);
            }
        }
        payload.rebuild_slot_indexes();
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

        let payload = &mut **self;
        let mut slot = payload.slots.remove(slot_index);
        debug_assert_eq!(slot.slot_id, source_slot_id);
        slot.slot_id = destination_slot_id;
        let slot_index = payload
            .slots
            .binary_search_by(|existing| existing.slot_id.cmp(&slot.slot_id))
            .unwrap_or_else(|slot_index| slot_index);
        payload.slots.insert(slot_index, slot);
        payload.rebuild_slot_indexes();
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

impl DerefMut for RuntimeSessionArchive {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state = next_revision_state(&self.state);
        Arc::make_mut(&mut self.payload)
    }
}

fn new_lineage_state() -> Arc<RuntimeSessionArchiveGenerationState> {
    Arc::new(RuntimeSessionArchiveGenerationState {
        generation: NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION.fetch_add(1, Ordering::AcqRel),
        lineage: NEXT_RUNTIME_SESSION_ARCHIVE_LINEAGE.fetch_add(1, Ordering::AcqRel),
        revision: 1,
        counters: Arc::new(RuntimeSessionArchiveStageCounters::default()),
        sealed: Mutex::new(RuntimeSessionArchiveSealState::Open),
    })
}

fn next_revision_state(
    current: &RuntimeSessionArchiveGenerationState,
) -> Arc<RuntimeSessionArchiveGenerationState> {
    Arc::new(RuntimeSessionArchiveGenerationState {
        generation: NEXT_RUNTIME_SESSION_ARCHIVE_GENERATION.fetch_add(1, Ordering::AcqRel),
        lineage: current.lineage,
        revision: current.revision.saturating_add(1),
        counters: Arc::new(RuntimeSessionArchiveStageCounters::default()),
        sealed: Mutex::new(RuntimeSessionArchiveSealState::Open),
    })
}
