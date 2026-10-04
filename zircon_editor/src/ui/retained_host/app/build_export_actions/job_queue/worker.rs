use std::path::PathBuf;
use std::sync::{mpsc::Sender, Arc};

use super::super::DesktopExportExecutionSummary;
use super::{DesktopExportProgressSnapshot, DesktopExportQueuedJob};
use crate::core::jobs::{EditorJob, JobContext, JobError};
use crate::ui::host::{
    EditorExportBuildError, EditorExportBuildProgress, EditorExportBuildReport, EditorManager,
};
use zircon_runtime::asset::project::ProjectManifest;

trait DesktopExportExecutor: Send + Sync {
    fn execute(
        &self,
        project_root: &std::path::Path,
        output_root: &std::path::Path,
        manifest: &ProjectManifest,
        profile_name: &str,
        cancel: &crate::core::jobs::CancellationToken,
        progress: &mut dyn FnMut(EditorExportBuildProgress),
    ) -> Result<EditorExportBuildReport, EditorExportBuildError>;
}

impl DesktopExportExecutor for EditorManager {
    fn execute(
        &self,
        project_root: &std::path::Path,
        output_root: &std::path::Path,
        manifest: &ProjectManifest,
        profile_name: &str,
        cancel: &crate::core::jobs::CancellationToken,
        progress: &mut dyn FnMut(EditorExportBuildProgress),
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        self.execute_native_aware_export_build_with_cancellation_and_progress(
            project_root,
            output_root,
            manifest,
            profile_name,
            cancel,
            progress,
        )
    }
}

#[derive(Debug)]
pub(super) struct DesktopExportJobResult {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) report: crate::ui::host::EditorExportBuildReport,
}

#[derive(Debug)]
pub(super) struct DesktopExportJobProgress {
    pub(super) id: u64,
    pub(super) progress: DesktopExportProgressSnapshot,
}

pub(super) struct DesktopExportEditorJob {
    job: DesktopExportQueuedJob,
    executor: Arc<dyn DesktopExportExecutor>,
    progress_sender: Sender<DesktopExportJobProgress>,
}

impl DesktopExportEditorJob {
    pub(super) fn new(
        job: DesktopExportQueuedJob,
        editor_manager: Arc<crate::ui::host::EditorManager>,
        progress_sender: Sender<DesktopExportJobProgress>,
    ) -> Self {
        Self {
            job,
            executor: editor_manager,
            progress_sender,
        }
    }

    #[cfg(test)]
    fn with_executor(
        job: DesktopExportQueuedJob,
        executor: Arc<dyn DesktopExportExecutor>,
        progress_sender: Sender<DesktopExportJobProgress>,
    ) -> Self {
        Self {
            job,
            executor,
            progress_sender,
        }
    }
}

impl EditorJob for DesktopExportEditorJob {
    type Output = DesktopExportJobResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let Self {
            job,
            executor,
            progress_sender,
        } = self;
        let job_id = job.id;
        let progress_context = context.clone();
        let mut report_progress = move |progress| {
            let progress = DesktopExportProgressSnapshot::from_report(progress);
            let detail = if progress.message.trim().is_empty() {
                progress.stage.clone()
            } else {
                format!("{} - {}", progress.stage, progress.message)
            };
            progress_context.report_progress(u32::from(progress.percent), 100, detail);
            let _ = progress_sender.send(DesktopExportJobProgress {
                id: job_id,
                progress,
            });
        };
        let result = executor.execute(
            &job.project_root,
            &job.output_root,
            &job.manifest,
            &job.profile_name,
            context.cancellation_token(),
            &mut report_progress,
        );
        match result {
            Ok(report) if !context.is_cancelled() => report
                .into_result()
                .map(|report| DesktopExportJobResult {
                    id: job.id,
                    profile_name: job.profile_name,
                    output_root: job.output_root,
                    report,
                })
                .map_err(JobError::failed),
            Ok(_) => Err(JobError::Cancelled),
            Err(EditorExportBuildError::Cancelled { .. }) => Err(JobError::Cancelled),
            Err(EditorExportBuildError::ReportFailed { .. }) if context.is_cancelled() => {
                Err(JobError::Cancelled)
            }
            Err(error) => Err(JobError::failed(error)),
        }
    }
}

pub(super) fn desktop_export_summary_from_job_result(
    result: DesktopExportJobResult,
) -> DesktopExportExecutionSummary {
    DesktopExportExecutionSummary::from_report(result.output_root, result.report)
}

#[cfg(test)]
#[path = "worker/tests/astra_outcome_tests.rs"]
mod astra_outcome_tests;

#[cfg(test)]
#[path = "tests/worker.rs"]
mod tests;
