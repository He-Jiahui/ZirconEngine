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
            Ok(report) if !context.is_cancelled() => Ok(DesktopExportJobResult {
                id: job.id,
                profile_name: job.profile_name,
                output_root: job.output_root,
                report,
            }),
            Ok(_) => Err(JobError::Cancelled),
            Err(EditorExportBuildError::Cancelled { .. }) => Err(JobError::Cancelled),
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
mod tests {
    use std::io;
    use std::sync::mpsc;

    use super::*;
    use crate::core::jobs::{
        test_job_system, CancellationToken, EditorJobSpec, JobCategory, JobError,
    };
    use zircon_runtime::asset::AssetUri;

    struct FailingDesktopExportExecutor;

    impl DesktopExportExecutor for FailingDesktopExportExecutor {
        fn execute(
            &self,
            _project_root: &std::path::Path,
            _output_root: &std::path::Path,
            _manifest: &ProjectManifest,
            _profile_name: &str,
            _cancel: &CancellationToken,
            _progress: &mut dyn FnMut(EditorExportBuildProgress),
        ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
            Err(EditorExportBuildError::Materialize {
                source: io::Error::new(io::ErrorKind::WriteZero, "retained worker source"),
            })
        }
    }

    #[test]
    fn retained_export_worker_ticket_preserves_typed_editor_export_error() {
        let jobs = test_job_system();
        let (progress_sender, _progress_receiver) = mpsc::channel();
        let job = DesktopExportQueuedJob {
            id: 7,
            profile_name: "desktop_windows".to_string(),
            project_root: PathBuf::from("Project"),
            manifest: ProjectManifest::new(
                "Project",
                AssetUri::parse("res://main.scene.toml").expect("test asset URI is valid"),
                1,
            ),
            output_root: PathBuf::from("Builds/windows"),
            cancel: CancellationToken::default(),
        };
        let ticket = jobs
            .submit(
                EditorJobSpec::new("retained export source", JobCategory::Export),
                DesktopExportEditorJob::with_executor(
                    job,
                    Arc::new(FailingDesktopExportExecutor),
                    progress_sender,
                ),
            )
            .expect("retained export job should submit");

        let error = ticket.wait().expect_err("retained export job should fail");
        let export_error = error
            .downcast_ref::<EditorExportBuildError>()
            .expect("job ticket must retain the typed editor export error");
        assert!(matches!(
            export_error,
            EditorExportBuildError::Materialize { source }
                if source.kind() == io::ErrorKind::WriteZero
        ));
        assert!(matches!(error, JobError::Failed(_)));
    }
}
