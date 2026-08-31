use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use zircon_runtime::asset::{AssetUri, ProjectImportReceipt};

use crate::core::jobs::JobError;
use crate::core::logging::{EditorLogService, LogEntry, LogJump, LogSeverity, LogSource};

use super::EditorAssetImportResult;

#[derive(Clone)]
pub(super) struct EditorAssetImportDiagnostics {
    logs: Arc<EditorLogService>,
}

pub(super) struct EditorAssetImportFlightDiagnostics {
    projection: EditorAssetImportDiagnostics,
    uri: Arc<AssetUri>,
    state: DeferredSubmissionDiagnostic<Result<EditorAssetImportResult, JobError>>,
}

pub(super) struct EditorModelImportDiagnostics {
    projection: EditorAssetImportDiagnostics,
    source_path: PathBuf,
    state: DeferredSubmissionDiagnostic<Result<ProjectImportReceipt, JobError>>,
}

struct DeferredSubmissionDiagnostic<T> {
    state: Mutex<DeferredSubmissionDiagnosticState<T>>,
}

enum DeferredSubmissionDiagnosticState<T> {
    PendingSubmission,
    Armed,
    Deferred(T),
    Emitted,
}

impl<T> DeferredSubmissionDiagnostic<T> {
    fn new() -> Self {
        Self {
            state: Mutex::new(DeferredSubmissionDiagnosticState::PendingSubmission),
        }
    }

    fn arm(&self) -> Option<T> {
        let mut state = self.lock_state();
        match std::mem::replace(&mut *state, DeferredSubmissionDiagnosticState::Emitted) {
            DeferredSubmissionDiagnosticState::PendingSubmission => {
                *state = DeferredSubmissionDiagnosticState::Armed;
                None
            }
            DeferredSubmissionDiagnosticState::Deferred(result) => Some(result),
            DeferredSubmissionDiagnosticState::Armed => {
                *state = DeferredSubmissionDiagnosticState::Armed;
                None
            }
            DeferredSubmissionDiagnosticState::Emitted => None,
        }
    }

    fn complete(&self, result: T) -> Option<T> {
        let mut state = self.lock_state();
        match std::mem::replace(&mut *state, DeferredSubmissionDiagnosticState::Emitted) {
            DeferredSubmissionDiagnosticState::PendingSubmission => {
                *state = DeferredSubmissionDiagnosticState::Deferred(result);
                None
            }
            DeferredSubmissionDiagnosticState::Armed => Some(result),
            DeferredSubmissionDiagnosticState::Deferred(previous) => {
                *state = DeferredSubmissionDiagnosticState::Deferred(previous);
                None
            }
            DeferredSubmissionDiagnosticState::Emitted => None,
        }
    }

    fn reject(&self) -> bool {
        let mut state = self.lock_state();
        if matches!(&*state, DeferredSubmissionDiagnosticState::Emitted) {
            false
        } else {
            *state = DeferredSubmissionDiagnosticState::Emitted;
            true
        }
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, DeferredSubmissionDiagnosticState<T>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl EditorAssetImportFlightDiagnostics {
    pub(super) fn new(uri: Arc<AssetUri>, projection: EditorAssetImportDiagnostics) -> Self {
        Self {
            projection,
            uri,
            state: DeferredSubmissionDiagnostic::new(),
        }
    }

    pub(super) fn arm(&self) {
        if let Some(result) = self.state.arm() {
            self.projection.project_asset_result(&self.uri, &result);
        }
    }

    pub(super) fn project_result(&self, result: Result<EditorAssetImportResult, JobError>) {
        if let Some(result) = self.state.complete(result) {
            self.projection.project_asset_result(&self.uri, &result);
        }
    }

    pub(super) fn reject_submission(&self, detail: &str) {
        if self.state.reject() {
            self.projection
                .project_asset_submission_error(&self.uri, detail);
        }
    }
}

impl EditorModelImportDiagnostics {
    pub(super) fn new(source_path: PathBuf, projection: EditorAssetImportDiagnostics) -> Self {
        Self {
            projection,
            source_path,
            state: DeferredSubmissionDiagnostic::new(),
        }
    }

    pub(super) fn arm(&self) {
        if let Some(result) = self.state.arm() {
            self.projection
                .project_model_result(&self.source_path, &result);
        }
    }

    pub(super) fn project_result(&self, result: Result<ProjectImportReceipt, JobError>) {
        if let Some(result) = self.state.complete(result) {
            self.projection
                .project_model_result(&self.source_path, &result);
        }
    }

    pub(super) fn reject_submission(&self, detail: &str) {
        if self.state.reject() {
            self.projection
                .project_model_submission_error(&self.source_path, detail);
        }
    }
}

impl EditorAssetImportDiagnostics {
    pub(super) fn new(logs: Arc<EditorLogService>) -> Self {
        Self { logs }
    }

    pub(super) fn project_asset_result(
        &self,
        uri: &AssetUri,
        result: &Result<EditorAssetImportResult, JobError>,
    ) {
        let jump = LogJump::asset(uri.to_string()).ok();
        match result {
            Ok(result) if result.status().imported => self.emit(
                LogSeverity::Info,
                format!(
                    "editor_asset_import result=committed uri={} importer={} digest={} reasons={:?}",
                    uri,
                    result.status().importer_id,
                    result.status().source_hash,
                    result.reasons(),
                ),
                "editor_asset_import result=committed message=truncated",
                jump,
            ),
            Ok(result) => self.emit(
                LogSeverity::Warning,
                format!(
                    "editor_asset_import result=not_imported uri={} importer={} digest={} reasons={:?}",
                    uri,
                    result.status().importer_id,
                    result.status().source_hash,
                    result.reasons(),
                ),
                "editor_asset_import result=not_imported message=truncated",
                jump,
            ),
            Err(error) => self.emit(
                job_error_severity(error),
                format!("editor_asset_import result=failed uri={uri} error={error}"),
                "editor_asset_import result=failed message=truncated",
                jump,
            ),
        }
    }

    pub(super) fn project_asset_submission_error(&self, uri: &AssetUri, detail: &str) {
        self.emit(
            LogSeverity::Error,
            format!("editor_asset_import result=rejected uri={uri} error={detail}"),
            "editor_asset_import result=rejected message=truncated",
            LogJump::asset(uri.to_string()).ok(),
        );
    }

    pub(super) fn project_model_result(
        &self,
        source_path: &Path,
        result: &Result<ProjectImportReceipt, JobError>,
    ) {
        match result {
            Ok(receipt) => {
                let source_uri = receipt.source_uri().to_string();
                self.emit(
                    LogSeverity::Info,
                    format!(
                        "editor_model_import result=committed uri={source_uri} generation={} records={}",
                        receipt.generation_sequence(),
                        receipt.committed_records().len(),
                    ),
                    "editor_model_import result=committed message=truncated",
                    LogJump::asset(source_uri).ok(),
                );
            }
            Err(error) => self.emit(
                job_error_severity(error),
                format!(
                    "editor_model_import result=failed source={} error={error}",
                    source_path.display(),
                ),
                "editor_model_import result=failed message=truncated",
                None,
            ),
        }
    }

    pub(super) fn project_model_submission_error(&self, source_path: &Path, detail: &str) {
        self.emit(
            LogSeverity::Error,
            format!(
                "editor_model_import result=rejected source={} error={detail}",
                source_path.display(),
            ),
            "editor_model_import result=rejected message=truncated",
            None,
        );
    }

    fn emit(
        &self,
        severity: LogSeverity,
        message: String,
        fallback: &'static str,
        jump: Option<LogJump>,
    ) {
        let entry =
            LogEntry::new_with_fallback(LogSource::import(), severity, message, fallback, 0, jump);
        if let Ok(entry) = entry {
            let _ = self.logs.emit(entry);
        }
    }
}

const fn job_error_severity(error: &JobError) -> LogSeverity {
    match error {
        JobError::Cancelled => LogSeverity::Warning,
        JobError::Failed(_) | JobError::Panicked(_) | JobError::ResultChannelClosed => {
            LogSeverity::Error
        }
    }
}
