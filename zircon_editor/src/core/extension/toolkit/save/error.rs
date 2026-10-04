use thiserror::Error;

use crate::core::editor_message::DocumentId;

use crate::core::extension::toolkit::ToolkitSaveFailure;

#[derive(Debug, Error)]
/// 文档级保存调度失败；宿主按类别决定重试或报告，失败不会自动移除文档注册。
pub enum SaveError {
    #[error("document {document:?} has no open toolkit")]
    DocumentNotRegistered { document: DocumentId },
    #[error("document {document:?} already has a save operation in progress")]
    SaveAlreadyInProgress { document: DocumentId },
    #[error("document {document:?} has a close operation in progress")]
    DocumentClosing { document: DocumentId },
    #[error("document {document:?} reference validation failed: {source}")]
    ReferenceValidationFailed {
        document: DocumentId,
        #[source]
        source: ToolkitSaveFailure,
    },
    #[error("document {document:?} save hook failed: {source}")]
    HookFailed {
        document: DocumentId,
        #[source]
        source: ToolkitSaveFailure,
    },
    #[error("document {document:?} autosave snapshot hook failed: {source}")]
    AutosaveHookFailed {
        document: DocumentId,
        #[source]
        source: ToolkitSaveFailure,
    },
}

impl SaveError {
    pub const fn document_id(&self) -> DocumentId {
        match self {
            Self::DocumentNotRegistered { document }
            | Self::SaveAlreadyInProgress { document }
            | Self::DocumentClosing { document }
            | Self::ReferenceValidationFailed { document, .. }
            | Self::HookFailed { document, .. }
            | Self::AutosaveHookFailed { document, .. } => *document,
        }
    }
}
