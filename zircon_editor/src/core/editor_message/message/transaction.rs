use serde::{Deserialize, Serialize};

use crate::core::editing::engine::{HistoryContextId, TransactionId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionMessage {
    Started {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Canceled {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Committed {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Undone {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Redone {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    HistoryTrimmed {
        history: HistoryContextId,
    },
}
