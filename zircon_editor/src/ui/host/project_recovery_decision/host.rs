use std::path::Path;
use std::time::Duration;

use crate::core::logging::{LogEntry, LogSeverity, LogSource};
use crate::core::notifications::{
    NotificationId, NotificationSource, ToastNotification, ToastSeverity,
};
use crate::core::recovery::{RestoreExecutionReport, RestoreStartup};

use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use super::RecoveryExecutionCompletion;
use super::service::{
    RecoveryCompletionDisposition, RecoveryOwnerAdmission, RecoveryUnresolvedSummary,
};

const RECOVERY_TOAST_SOURCE: &str = "editor.recovery";
const RECOVERY_SUCCESS_TOAST_LIFETIME: Duration = Duration::from_secs(5);
const RECOVERY_FAILURE_TOAST_LIFETIME: Duration = Duration::from_secs(8);
const RECOVERY_SUCCESS_TITLE_KEY: &str = "editor.recovery.completed.title";
const RECOVERY_SUCCESS_MESSAGE_KEY: &str = "editor.recovery.completed.message";
const RECOVERY_FAILURE_TITLE_KEY: &str = "editor.recovery.failed.title";
const RECOVERY_FAILURE_MESSAGE_KEY: &str = "editor.recovery.unresolved.message";
const RECOVERY_FAILURE_LOG_DETAIL_LIMIT: usize = 4;
const UNKNOWN_RECOVERY_EXECUTION_FRAME: u64 = 0;

impl EditorManager {
    /// Installs a captured residual recovery startup after the new project session reached Ready.
    pub(in crate::ui::host) fn begin_project_recovery_decisions(
        &self,
        project_root: &Path,
        startup: RestoreStartup,
    ) -> Result<(), EditorError> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(project_recovery_error)?;
        // The caller already owns the non-reentrant project transition guard.
        let session = self.project_session_guard.lock()
            .map_err(|_| project_recovery_error("project session owner lock is poisoned"))?;
        let guard = session.as_ref()
            .ok_or_else(|| project_recovery_error("project recovery requires an owned Ready session"))?;
        let admission = RecoveryOwnerAdmission::from_guard(guard)
            .map_err(project_recovery_error)?;
        self.project_recovery
            .begin(center, project_root, startup, &admission)
            .map(|_| ())
            .map_err(project_recovery_error)
    }

    /// Advances receipt collection and worker completion outside rendering and native callbacks.
    pub(crate) fn pump_project_recovery_decisions(&self) -> Result<(), EditorError> {
        // This is a workload hint only; it confers no session authority and consumes no receipt.
        // The guard is dropped before acquiring the project transition lane.
        if !self.project_recovery.is_active() {
            return Ok(());
        }
        let completion = {
            // Tick does not hold this guard. Begin and Close use their already-held entry instead.
            let _transition = self.begin_project_session_transition()?;
            let center = self.context().notifications().decisions()
                .map_err(project_recovery_error)?;
            let session = self.project_session_guard.lock()
                .map_err(|_| project_recovery_error("project session owner lock is poisoned"))?;
            let guard = session.as_ref()
                .ok_or_else(|| project_recovery_error("project recovery requires an owned Ready session"))?;
            let admission = RecoveryOwnerAdmission::from_guard(guard)
                .map_err(project_recovery_error)?;
            self.project_recovery
                .pump(center, self.context().jobs(), &admission)
                .map_err(project_recovery_error)?
        };
        // User-facing publication and logging never run under the transition or owner locks.
        if let Some(completion) = completion {
            self.publish_recovery_execution_completion(completion)?;
        }
        Ok(())
    }

    /// A session with unresolved choices or a running restore job must retain its project
    /// ownership. Otherwise close could clear the only residual marker while a discard action is
    /// still pending.
    pub(in crate::ui::host) fn ensure_project_recovery_is_settled(
        &self,
    ) -> Result<(), EditorError> {
        if self.project_recovery.request_review_if_active() {
            return Err(EditorError::Project(
                "project recovery choices or background restore work are still active; complete the recovery flow before closing the project"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn project_recovery_is_active(&self) -> bool {
        self.project_recovery.is_active()
    }

    fn publish_recovery_execution_completion(
        &self,
        completion: RecoveryExecutionCompletion,
    ) -> Result<(), EditorError> {
        let source = NotificationSource::builtin(RECOVERY_TOAST_SOURCE)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        let job = completion.job();
        if let Some(result) = completion.result() {
            match result {
                Ok(report) if report.has_failures() => {
                    self.emit_recovery_document_failures(job.value(), report);
                }
                Err(error) => self.emit_recovery_execution_failure(job.value(), error),
                Ok(_) => {}
            }
        }
        let (id_suffix, severity, title_key, message_key, lifetime) = match completion.disposition() {
            RecoveryCompletionDisposition::Completed => (
                "completed",
                ToastSeverity::Success,
                RECOVERY_SUCCESS_TITLE_KEY,
                RECOVERY_SUCCESS_MESSAGE_KEY,
                RECOVERY_SUCCESS_TOAST_LIFETIME,
            ),
            RecoveryCompletionDisposition::Incomplete => {
                self.emit_recovery_unresolved(job.value(), completion.unresolved());
                (
                    "failed",
                    ToastSeverity::Error,
                    RECOVERY_FAILURE_TITLE_KEY,
                    RECOVERY_FAILURE_MESSAGE_KEY,
                    RECOVERY_FAILURE_TOAST_LIFETIME,
                )
            }
            // The resolved decision is the acknowledgement. Do not label it a successful restore.
            RecoveryCompletionDisposition::EvidenceKept => return Ok(()),
        };
        let id = NotificationId::parse(format!("editor.recovery.{id_suffix}.{}", job.value()))
            .map_err(|error| EditorError::Project(error.to_string()))?;
        let toast = ToastNotification::new(id, source, severity, title_key, message_key, lifetime)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.context()
            .notifications()
            .publish_toast(toast)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    fn emit_recovery_execution_failure(&self, job: u64, error: &crate::core::jobs::JobError) {
        let entry = LogEntry::new(
            LogSource::editor(),
            LogSeverity::Error,
            format!("autosave recovery job {job} failed: {error}"),
            UNKNOWN_RECOVERY_EXECUTION_FRAME,
            None,
        )
        .or_else(|_| {
            LogEntry::new(
                LogSource::editor(),
                LogSeverity::Error,
                "autosave recovery job failed; diagnostic exceeds the log-entry limit.",
                UNKNOWN_RECOVERY_EXECUTION_FRAME,
                None,
            )
        });
        if let Ok(entry) = entry {
            let _ = self.context().logs().emit(entry);
        }
    }

    fn emit_recovery_unresolved(&self, job: u64, summary: RecoveryUnresolvedSummary) {
        let entry = LogEntry::new(
            LogSource::editor(),
            LogSeverity::Error,
            format!(
                "autosave recovery job {job} leaves {} unresolved document(s): {} safe to retry, {} requiring operator action",
                summary.total, summary.retryable, summary.operator_required
            ),
            UNKNOWN_RECOVERY_EXECUTION_FRAME,
            None,
        );
        if let Ok(entry) = entry {
            let _ = self.context().logs().emit(entry);
        }
    }

    fn emit_recovery_document_failures(&self, job: u64, report: &RestoreExecutionReport) {
        let details = report
            .records()
            .iter()
            .filter_map(|record| {
                record.failure().map(|failure| {
                    format!(
                        "{} ({:?}): {failure}",
                        record.document().as_str(),
                        record.action()
                    )
                })
            })
            .take(RECOVERY_FAILURE_LOG_DETAIL_LIMIT)
            .collect::<Vec<_>>()
            .join("; ");
        let omitted = report
            .failure_count()
            .saturating_sub(RECOVERY_FAILURE_LOG_DETAIL_LIMIT);
        let omitted = (omitted > 0).then(|| format!("; {omitted} additional failure(s) omitted"));
        let message = format!(
            "autosave recovery job {job} completed {}/{} document(s); {} failed: {details}{}",
            report.success_count(),
            report.records().len(),
            report.failure_count(),
            omitted.as_deref().unwrap_or_default()
        );
        let entry = LogEntry::new(
            LogSource::editor(),
            LogSeverity::Error,
            message,
            UNKNOWN_RECOVERY_EXECUTION_FRAME,
            None,
        )
        .or_else(|_| {
            LogEntry::new(
                LogSource::editor(),
                LogSeverity::Error,
                "autosave recovery completed with document failures; inspect recovery diagnostics.",
                UNKNOWN_RECOVERY_EXECUTION_FRAME,
                None,
            )
        });
        if let Ok(entry) = entry {
            let _ = self.context().logs().emit(entry);
        }
    }
}

fn project_recovery_error(error: impl std::fmt::Display) -> EditorError {
    EditorError::Project(format!("project recovery lifecycle failed: {error}"))
}
