use thiserror::Error;

use crate::core::editor_message::DocumentId;

use super::ToolkitInstanceId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ToolkitRegistryError {
    #[error("document {document:?} already has an open toolkit")]
    DocumentAlreadyRegistered { document: DocumentId },
    #[error("toolkit instance {instance:?} is already registered")]
    InstanceAlreadyRegistered { instance: ToolkitInstanceId },
    #[error("toolkit instance {instance:?} is not registered for editing")]
    EditInstanceNotRegistered { instance: ToolkitInstanceId },
    #[error("toolkit instance {instance:?} has invalid menu path `{path}`")]
    InvalidMenuPath {
        instance: ToolkitInstanceId,
        path: String,
    },
    #[error("toolkit instance {instance:?} declares duplicate menu path `{path}`")]
    DuplicateMenuPath {
        instance: ToolkitInstanceId,
        path: String,
    },
    #[error("document {document:?} has {active_saves} active save operation(s)")]
    DocumentBusy {
        document: DocumentId,
        active_saves: usize,
    },
    #[error("document {document:?} has {active_edits} active edit operation(s)")]
    DocumentEditing {
        document: DocumentId,
        active_edits: usize,
    },
    #[error("document {document:?} already has a close operation in progress")]
    CloseAlreadyInProgress { document: DocumentId },
    #[error("cannot clear document toolkits while saves are active for {documents:?}")]
    DocumentsBusy { documents: Vec<DocumentId> },
    #[error("cannot clear document toolkits while edits are active for {documents:?}")]
    DocumentsEditing { documents: Vec<DocumentId> },
    #[error("cannot clear document toolkits while close operations are active for {documents:?}")]
    DocumentsClosing { documents: Vec<DocumentId> },
    #[error("close lease for document {document:?} is no longer valid")]
    CloseLeaseInvalid { document: DocumentId },
    #[error("document clear lease is no longer valid")]
    ClearLeaseInvalid,
    #[error("document id allocation space is exhausted")]
    DocumentIdExhausted,
    #[error("edit lease count for document {document:?} is exhausted")]
    EditCountExhausted { document: DocumentId },
    #[error("edit generation for document {document:?} is exhausted")]
    EditGenerationExhausted { document: DocumentId },
    #[error("document toolkit generation space is exhausted")]
    GenerationExhausted,
}
