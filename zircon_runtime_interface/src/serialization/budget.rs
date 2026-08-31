/// Caller-owned ceiling for one serialization attempt.
///
/// The budget is intentionally explicit and has no `Default` implementation:
/// each archive owner must choose a limit appropriate for its wire contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerializationBudget {
    max_output_bytes: usize,
}

impl SerializationBudget {
    pub const fn new(max_output_bytes: usize) -> Self {
        Self { max_output_bytes }
    }

    pub const fn max_output_bytes(self) -> usize {
        self.max_output_bytes
    }
}
