use serde::{Deserialize, Serialize};

/// The durable transaction span represented by an autosave snapshot.
///
/// The P1-10 durable journal does not yet link autosave commit records to
/// journal coverage, so current captures record that absence explicitly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutosaveJournalRange {
    #[default]
    Unavailable,
}
