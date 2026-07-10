use serde::{Deserialize, Serialize};

use super::{DocumentMessage, FocusMessage, ModeMessage, TransactionMessage};

/// Small, cloneable editor facts. Heavy state remains behind query owners.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorMessagePayload {
    Document(DocumentMessage),
    Transaction(TransactionMessage),
    Mode(ModeMessage),
    Focus(FocusMessage),
    Custom {
        schema_id: String,
        payload: serde_json::Value,
    },
}
