use serde::{Deserialize, Serialize};

use crate::core::jobs::{EditorJobEventJournalGap, JobEvent};

use super::{
    DocumentMessage, EditorMessageSchemaId, FocusMessage, ModeMessage, SceneInspectionMessage,
    ToolMessage, TransactionMessage,
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
    JobJournalGap(EditorJobEventJournalGap),
    Custom {
        schema_id: EditorMessageSchemaId,
        payload: serde_json::Value,
    },
}
