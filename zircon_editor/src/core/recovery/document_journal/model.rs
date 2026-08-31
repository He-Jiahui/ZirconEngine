/// One committed transaction that has crossed the durable journal boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentJournalAppend {
    sequence: u64,
}

impl DocumentJournalAppend {
    pub(super) const fn new(sequence: u64) -> Self {
        Self { sequence }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}
