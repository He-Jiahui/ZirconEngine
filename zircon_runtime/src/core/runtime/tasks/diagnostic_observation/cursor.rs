#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskDiagnosticCursor {
    source_id: u64,
    next_observation_sequence: u64,
}

impl TaskDiagnosticCursor {
    pub(super) const fn new(source_id: u64, next_observation_sequence: u64) -> Self {
        Self {
            source_id,
            next_observation_sequence,
        }
    }

    pub const fn source_id(self) -> u64 {
        self.source_id
    }

    pub const fn next_observation_sequence(self) -> u64 {
        self.next_observation_sequence
    }
}
