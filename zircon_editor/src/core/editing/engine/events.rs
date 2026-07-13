use super::{HistoryContextId, TransactionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionEventKind {
    Started,
    Canceled,
    Committed,
    UndoApplied,
    RedoApplied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionEvent {
    pub transaction: TransactionId,
    pub history: HistoryContextId,
    pub label: String,
    pub timestamp_frame: u64,
    pub kind: TransactionEventKind,
}
