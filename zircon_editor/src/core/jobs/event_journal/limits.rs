use std::time::Duration;

use super::gap::JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;

const DEFAULT_JOB_EVENT_JOURNAL_ENTRIES: usize = 4_096;
const DEFAULT_JOB_EVENT_JOURNAL_RETAINED_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_JOB_EVENT_JOURNAL_AGE: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorJobEventJournalLimits {
    max_entries: usize,
    max_retained_bytes: usize,
    max_oldest_age: Duration,
}

impl EditorJobEventJournalLimits {
    pub const fn new(max_entries: usize, max_retained_bytes: usize) -> Self {
        Self {
            max_entries,
            max_retained_bytes,
            max_oldest_age: DEFAULT_JOB_EVENT_JOURNAL_AGE,
        }
    }

    pub const fn with_max_oldest_age(mut self, max_oldest_age: Duration) -> Self {
        self.max_oldest_age = max_oldest_age;
        self
    }

    pub const fn max_entries(self) -> usize {
        self.max_entries
    }

    pub const fn max_retained_bytes(self) -> usize {
        self.max_retained_bytes
    }

    pub const fn max_oldest_age(self) -> Duration {
        self.max_oldest_age
    }

    pub(super) fn normalized(self) -> Self {
        Self {
            max_entries: self.max_entries.max(1),
            max_retained_bytes: self
                .max_retained_bytes
                .max(JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES),
            max_oldest_age: self.max_oldest_age,
        }
    }
}

impl Default for EditorJobEventJournalLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_JOB_EVENT_JOURNAL_ENTRIES,
            DEFAULT_JOB_EVENT_JOURNAL_RETAINED_BYTES,
        )
    }
}
