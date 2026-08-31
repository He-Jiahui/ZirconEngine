use crate::text::document::{TextDocumentAdmissionFailure, TextDocumentStoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiTextDocumentSessionError {
    SourceNotSynchronized,
    InvalidEditIntent,
    ByteOffsetOverflow,
    SourceEpochExhausted,
    Store(TextDocumentStoreError),
}

impl UiTextDocumentSessionError {
    pub(in crate::ui) const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::SourceNotSynchronized => "source_not_synchronized",
            Self::InvalidEditIntent => "invalid_edit_intent",
            Self::ByteOffsetOverflow => "byte_offset_overflow",
            Self::SourceEpochExhausted => "source_epoch_exhausted",
            Self::Store(TextDocumentStoreError::UnknownDocument) => "unknown_document",
            Self::Store(TextDocumentStoreError::StaleRevision { .. }) => "stale_revision",
            Self::Store(TextDocumentStoreError::DocumentOwnerExhausted) => {
                "document_owner_exhausted"
            }
            Self::Store(TextDocumentStoreError::SnapshotLeaseBudgetUnavailable) => {
                "snapshot_lease_budget_unavailable"
            }
            Self::Store(TextDocumentStoreError::AdmissionDenied(failure)) => {
                admission_diagnostic_code(failure)
            }
            Self::Store(TextDocumentStoreError::ReceiptProjection(_)) => {
                "receipt_projection_rejected"
            }
            Self::Store(TextDocumentStoreError::Edit(_)) => "document_edit_rejected",
        }
    }
}

impl From<TextDocumentStoreError> for UiTextDocumentSessionError {
    fn from(error: TextDocumentStoreError) -> Self {
        Self::Store(error)
    }
}

const fn admission_diagnostic_code(failure: TextDocumentAdmissionFailure) -> &'static str {
    match failure {
        TextDocumentAdmissionFailure::DocumentCount => "admission_document_count",
        TextDocumentAdmissionFailure::DocumentBytes => "admission_document_bytes",
        TextDocumentAdmissionFailure::TotalDocumentBytes => "admission_total_document_bytes",
        TextDocumentAdmissionFailure::ReplacementBytes => "admission_replacement_bytes",
        TextDocumentAdmissionFailure::DocumentRetainedSourceBytes => {
            "admission_document_retained_source_bytes"
        }
        TextDocumentAdmissionFailure::TotalRetainedSourceBytes => {
            "admission_total_retained_source_bytes"
        }
        TextDocumentAdmissionFailure::AdditionSources => "admission_addition_sources",
        TextDocumentAdmissionFailure::Pieces => "admission_pieces",
        TextDocumentAdmissionFailure::CurrentSnapshotBytes => "admission_current_snapshot_bytes",
        TextDocumentAdmissionFailure::ActiveSnapshotLeaseCount => {
            "admission_active_snapshot_lease_count"
        }
        TextDocumentAdmissionFailure::ActiveSnapshotLeaseBytes => {
            "admission_active_snapshot_lease_bytes"
        }
    }
}
