use super::{PendingEditId, PendingEditIntent, PendingEditQueueSummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingEditExitDecision {
    Apply,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEditDecisionPrompt {
    pub pending_count: usize,
    pub payload_bytes: usize,
    pub oldest_age: Option<std::time::Duration>,
}

impl PendingEditDecisionPrompt {
    pub const fn new(summary: PendingEditQueueSummary) -> Self {
        Self {
            pending_count: summary.pending_count,
            payload_bytes: summary.payload_bytes,
            oldest_age: summary.oldest_age,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PendingEditApplyFailure<E> {
    pub intent: PendingEditIntent,
    pub error: E,
}

#[derive(Debug, PartialEq)]
pub struct PendingEditApplyReport<E> {
    pub applied: Vec<PendingEditId>,
    pub failures: Vec<PendingEditApplyFailure<E>>,
    pub budget_exhausted: bool,
    pub remaining: PendingEditQueueSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingEditDiscardReport {
    pub discarded_count: usize,
    pub discarded_payload_bytes: usize,
    pub remaining: PendingEditQueueSummary,
}
