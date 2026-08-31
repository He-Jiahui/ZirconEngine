mod autosave;
mod autosave_adapter;
mod autosave_catalog;
mod autosave_diagnostics;
mod autosave_service;
mod autosave_shutdown;
mod document_journal;
mod project_recovery_assessment;
mod project_session_effect_ledger;
mod restore_executor;
mod restore_flow;
mod session_guard;

pub use autosave::{
    AutosaveContentDigest, AutosaveDocumentId, AutosaveDocumentState, AutosaveEngineSchema,
    AutosaveError, AutosaveExtension, AutosaveJobPolicy, AutosaveJournalRange, AutosavePlan,
    AutosavePolicy, AutosaveScheduler, AutosaveSnapshotProvenance, AutosaveSourceDigest,
    AutosaveStore, AUTOSAVE_RETAINED_SNAPSHOT_COUNT,
};
pub use autosave_adapter::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentOutcome,
    AutosaveDocumentOutcomeKind, AutosaveDocumentRequest, AutosaveFailureStage,
    AutosaveHealthTelemetry, AutosaveJobAdapter, AutosaveRetryability, AutosaveSnapshot,
    AutosaveSnapshotSource, AutosaveWriteResult, DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};
pub use autosave_catalog::{
    AutosaveRecoveryCatalogDiagnostic, AutosaveRecoveryCatalogDiagnosticKind,
    AutosaveRecoveryCatalogReport, AutosaveSourcePath,
};
pub use autosave_diagnostics::{
    AutosaveDiagnosticError, AutosaveDiagnosticReadIssue, AutosaveDiagnosticRecord,
    AutosaveDiagnosticReport, AutosaveDiagnosticStore,
};
pub(crate) use autosave_service::{EditorAutosavePoll, EditorAutosaveService};
pub(crate) use autosave_shutdown::AutosaveShutdownReport;
pub use document_journal::{
    DocumentJournalAppend, DocumentJournalCoordinator, DocumentJournalCoordinatorError,
};
pub(crate) use project_recovery_assessment::{
    ProjectRecoveryAdmission, ProjectRecoveryAssessment, ProjectRecoveryAssessmentError,
    ProjectRecoveryReconciliationReason, ProjectRecoveryTakeoverDisposition,
};
pub(crate) use project_session_effect_ledger::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedger,
    ProjectSessionEffectLedgerError, ProjectSessionEffectLedgerPhase,
    ProjectSessionEffectLedgerStore, ProjectSessionEffectRecoveryEntry,
    ProjectSessionRecoveryStatus,
};
pub use restore_executor::{
    RecoveredDocumentCopy, RestoreDocumentExecutionError, RestoreExecutionError,
    RestoreExecutionOutcome, RestoreExecutionRecord, RestoreExecutionReport,
    RestoreExecutionRetryability, RestoreExecutor,
};
pub use restore_flow::{
    RestoreAction, RestoreCandidate, RestoreFlow, RestoreFlowError, RestoreFreshness, RestorePlan,
    RestoreResolution, RestoreStartup,
};
pub use session_guard::{
    ProjectSessionAdmissionRecordV1, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
    SessionGuardError, SessionGuardResidual, SessionLockDurability, SessionLockInspection,
    SESSION_LOCK_FILE_NAME,
};

#[cfg(test)]
mod tests;
