use serde::{Deserialize, Serialize};

use crate::core::editing::engine::HistoryContextId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionMessage {
    Committed {
        history: HistoryContextId,
        label: String,
    },
    Undone {
        history: HistoryContextId,
    },
    Redone {
        history: HistoryContextId,
    },
    HistoryTrimmed {
        history: HistoryContextId,
    },
}
