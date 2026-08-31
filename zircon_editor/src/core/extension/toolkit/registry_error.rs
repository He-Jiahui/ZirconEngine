use thiserror::Error;

use crate::core::editor_message::DocumentId;

use super::ToolkitInstanceId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ToolkitRegistryError {
    #[error("document {document:?} already has an open toolkit")]
    DocumentAlreadyRegistered { document: DocumentId },
    #[error("toolkit instance {instance:?} is already registered")]
    InstanceAlreadyRegistered { instance: ToolkitInstanceId },
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
    #[error("document {document:?} already has a close operation in progress")]
    CloseAlreadyInProgress { document: DocumentId },
    #[error("cannot clear document toolkits while saves are active for {documents:?}")]
    DocumentsBusy { documents: Vec<DocumentId> },
    #[error("cannot clear document toolkits while close operations are active for {documents:?}")]
    DocumentsClosing { documents: Vec<DocumentId> },
    #[error("close lease for document {document:?} is no longer valid")]
    CloseLeaseInvalid { document: DocumentId },
    #[error("document id allocation space is exhausted")]
    DocumentIdExhausted,
    #[error("document toolkit generation space is exhausted")]
    GenerationExhausted,
}
