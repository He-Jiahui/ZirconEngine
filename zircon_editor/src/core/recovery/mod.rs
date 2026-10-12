//! 恢复域汇集自动保存、文档日志、项目会话副作用台账和残留锁；宿主必须先取得项目会话权限再执行恢复计划。

#[path = "autosave/mod.rs"]
mod autosave;
#[path = "autosave_adapter/mod.rs"]
mod autosave_adapter;
#[path = "autosave_catalog/mod.rs"]
mod autosave_catalog;
#[path = "autosave_diagnostics.rs"]
mod autosave_diagnostics;
#[path = "autosave_service.rs"]
mod autosave_service;
#[path = "autosave_shutdown.rs"]
mod autosave_shutdown;
#[path = "document_journal/mod.rs"]
mod document_journal;
#[path = "project_recovery_assessment.rs"]
mod project_recovery_assessment;
#[path = "project_session_effect_ledger/mod.rs"]
mod project_session_effect_ledger;
#[path = "restore_executor.rs"]
mod restore_executor;
#[path = "restore_flow.rs"]
mod restore_flow;
#[path = "session_guard.rs"]
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
#[path = "tests/cases.rs"]
mod tests;
