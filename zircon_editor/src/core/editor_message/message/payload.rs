use serde::{Deserialize, Serialize};

use crate::core::jobs::JobEvent;

use super::{
    DocumentMessage, FocusMessage, ModeMessage, SceneInspectionMessage, ToolMessage,
    TransactionMessage,
};

/// Small, cloneable editor facts. Heavy state remains behind query owners.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorMessagePayload {
    Document(DocumentMessage),
    Transaction(TransactionMessage),
    Mode(ModeMessage),
    Focus(FocusMessage),
    SceneInspection(SceneInspectionMessage),
    Tool(ToolMessage),
    Job(JobEvent),
    Custom {
        schema_id: String,
        payload: serde_json::Value,
    },
}
