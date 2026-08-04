use thiserror::Error;

use crate::core::editing::engine::EditCommandError;
use crate::core::editor_message::DocumentId;

#[derive(Debug, Error)]
pub enum DirtyRegistryError {
    #[error("document {document:?} is not registered with the dirty-state projection")]
    DocumentNotRegistered { document: DocumentId },
    #[error("dirty external-effect revision space is exhausted")]
    ExternalEffectRevisionExhausted,
    #[error("dirty document generation space is exhausted")]
    DocumentGenerationExhausted,
    #[error("dirty snapshot for document {document:?} did not stabilize after {attempts} attempts")]
    SnapshotUnstable {
        document: DocumentId,
        attempts: usize,
    },
    #[error("dirty delta did not stabilize after {attempts} changed-document attempts")]
    DeltaUnstable { attempts: usize },
    #[error("dirty registry cursor belongs to another registry instance")]
    CursorRegistryMismatch,
    #[error("document {document:?} changed after save snapshot generation {expected_generation}")]
    DocumentChangedDuringSave {
        document: DocumentId,
        expected_generation: u64,
    },
    #[error(transparent)]
    Transaction(#[from] EditCommandError),
}
