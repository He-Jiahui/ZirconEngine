mod gap;
mod journal;
mod limits;
mod snapshot;

pub use gap::EditorJobEventJournalGap;
pub use limits::EditorJobEventJournalLimits;
pub use snapshot::EditorJobEventJournalSnapshot;

pub(super) use journal::{EditorJobEventJournal, EditorJobEventJournalRecord};

#[cfg(test)]
mod integration_tests;
