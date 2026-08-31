use serde::{Deserialize, Serialize};

pub(super) const JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES: usize =
    std::mem::size_of::<EditorJobEventJournalGap>();

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorJobEventJournalGap {
    dropped_lifecycle_events: u64,
    first_dropped_sequence: u64,
    last_dropped_sequence: u64,
}

impl EditorJobEventJournalGap {
    pub(super) const fn single(sequence: u64) -> Self {
        Self {
            dropped_lifecycle_events: 1,
            first_dropped_sequence: sequence,
            last_dropped_sequence: sequence,
        }
    }

    pub(super) fn merge(&mut self, other: Self) {
        self.dropped_lifecycle_events = self
            .dropped_lifecycle_events
            .saturating_add(other.dropped_lifecycle_events);
        self.first_dropped_sequence = self
            .first_dropped_sequence
            .min(other.first_dropped_sequence);
        self.last_dropped_sequence = self.last_dropped_sequence.max(other.last_dropped_sequence);
    }

    pub const fn dropped_lifecycle_events(&self) -> u64 {
        self.dropped_lifecycle_events
    }

    pub const fn first_dropped_sequence(&self) -> u64 {
        self.first_dropped_sequence
    }

    pub const fn last_dropped_sequence(&self) -> u64 {
        self.last_dropped_sequence
    }
}
