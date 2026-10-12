use thiserror::Error;

use crate::core::editor_message::DocumentId;

use crate::core::extension::toolkit::{DocumentSaveReport, ToolkitSaveFailure};

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
    #[error(
        "document {document:?} source write committed ({written_bytes} successfully recorded bytes), but the save hook failed afterwards: {source}",
        document = report.document_id(),
        written_bytes = report.written_bytes()
    )]
    /// A real source write was committed before the hook returned an error.
    /// The report retains the successfully recorded byte prefix; an overflow source
    /// means it does not contain the complete total.
    HookFailedAfterWrite {
        report: DocumentSaveReport,
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
            Self::HookFailedAfterWrite { report, .. } => report.document_id(),
        }
    }
}

/// The original unwind payload from a save hook after a recorded source commit.
///
/// The mutex only adapts the panic payload's Send contract to the existing
/// Send + Sync error transport. The display message never replaces the payload.
#[derive(Error)]
#[error("save hook panicked after a source commit: {message}")]
pub struct SaveHookPanic {
    message: String,
    payload: std::sync::Mutex<Box<dyn std::any::Any + Send + 'static>>,
}

impl SaveHookPanic {
    pub(crate) fn new(payload: Box<dyn std::any::Any + Send + 'static>) -> Self {
        let message = if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&'static str>() {
            (*message).to_string()
        } else {
            "non-string panic payload".to_string()
        };
        Self {
            message,
            payload: std::sync::Mutex::new(payload),
        }
    }

    /// Returns the original payload even if its transport mutex was poisoned.
    pub fn into_payload(self) -> Box<dyn std::any::Any + Send + 'static> {
        self.payload
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl std::fmt::Debug for SaveHookPanic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SaveHookPanic")
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}
