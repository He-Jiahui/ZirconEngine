use serde::{Deserialize, Serialize};

use crate::core::editor_message::HistoryContextId;

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
