use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::artifact::RuntimeSessionArchiveSealState;
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
        Self {
            format_version: wire.format_version,
            slots: wire.slots,
        }
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
    pub(super) counters: Arc<RuntimeSessionArchiveStageCounters>,
    pub(super) sealed: Mutex<RuntimeSessionArchiveSealState>,
}

#[derive(Default)]
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
        Self::from_deserialized_payload(RuntimeSessionArchivePayload {
            format_version,
            slots,
        })
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
