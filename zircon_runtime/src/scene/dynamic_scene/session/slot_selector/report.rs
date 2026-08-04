use serde::{Deserialize, Serialize};

use super::super::{RuntimeSessionSlot, RuntimeSessionSlotSummary};
use super::selector::RuntimeSessionSlotSelector;

/// A generation-bound borrow of the slot selected from one archive revision.
///
/// The archive remains immutably borrowed for this handle's lifetime, so the
/// indexed row cannot be reused by a mutation before the consumer finishes.
#[derive(Debug)]
pub struct RuntimeSessionSlotSelection<'archive> {
    archive_generation: u64,
    archive_revision: u64,
    slot: &'archive RuntimeSessionSlot,
}

impl<'archive> RuntimeSessionSlotSelection<'archive> {
    pub(super) fn new(
        archive_generation: u64,
        archive_revision: u64,
        slot: &'archive RuntimeSessionSlot,
    ) -> Self {
        Self {
            archive_generation,
            archive_revision,
            slot,
        }
    }

    pub fn archive_generation(&self) -> u64 {
        self.archive_generation
    }

    pub fn archive_revision(&self) -> u64 {
        self.archive_revision
    }

    pub fn slot(&self) -> &'archive RuntimeSessionSlot {
        self.slot
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionSlotSelectionReport {
    pub selector: RuntimeSessionSlotSelector,
    pub selected_slot_id: String,
    pub summary: RuntimeSessionSlotSummary,
}
