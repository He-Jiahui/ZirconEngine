use crate::text::document::{TextDocumentAdmissionFailure, TextDocumentStoreError};

/// 区分会话同步、编辑意图、公开偏移编码与底层文档存储的拒绝原因。
///
/// `Store` 保留原存储错误，供调用方判定；`diagnostic_code` 则投影为不含正文的静态诊断码。
/// 同步方法没有直接返回错误时，会先按 owner 保存此值，再由后续身份查询或编辑返回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiTextDocumentSessionError {
    SourceNotSynchronized,
    InvalidEditIntent,
    ByteOffsetOverflow,
    SourceEpochExhausted,
    Store(TextDocumentStoreError),
}

impl UiTextDocumentSessionError {
    /// 跨输入报告传递稳定且不含文本内容的失败码；不会暴露文档载荷或借错误码放宽准入限制。
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
