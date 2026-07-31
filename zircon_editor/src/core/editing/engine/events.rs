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

pub trait TransactionEventSink: Send + Sync {
    fn publish(&self, event: TransactionEvent) -> TransactionEventDelivery;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionEventDelivery {
    Delivered,
    Backpressured,
    Rejected,
}

#[derive(Default)]
pub(crate) struct DetachedTransactionEventSink;

impl TransactionEventSink for DetachedTransactionEventSink {
    fn publish(&self, _event: TransactionEvent) -> TransactionEventDelivery {
        TransactionEventDelivery::Delivered
    }
}
