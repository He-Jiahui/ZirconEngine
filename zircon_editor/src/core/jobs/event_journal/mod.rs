#[path = "gap.rs"]
mod gap;
#[path = "journal.rs"]
mod journal;
#[path = "limits.rs"]
mod limits;
#[path = "snapshot.rs"]
mod snapshot;

pub use gap::EditorJobEventJournalGap;
pub use limits::EditorJobEventJournalLimits;
pub use snapshot::EditorJobEventJournalSnapshot;

pub(crate) use journal::{EditorJobEventJournal, EditorJobEventJournalRecord};

#[cfg(test)]
#[path = "tests/integration_tests.rs"]
mod integration_tests;
