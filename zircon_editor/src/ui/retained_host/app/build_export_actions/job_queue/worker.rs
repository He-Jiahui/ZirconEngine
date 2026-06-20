use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};
use std::thread;

use super::super::DesktopExportExecutionSummary;
use super::{DesktopExportProgressSnapshot, DesktopExportQueuedJob};

#[derive(Debug)]
pub(super) struct DesktopExportJobResult {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) cancel_requested: Arc<AtomicBool>,
    pub(super) result: Result<crate::ui::host::EditorExportBuildReport, String>,
}

#[derive(Debug)]
pub(super) struct DesktopExportJobProgress {
    pub(super) id: u64,
    pub(super) progress: DesktopExportProgressSnapshot,
}

#[derive(Debug)]
pub(super) enum DesktopExportJobMessage {
    Progress(DesktopExportJobProgress),
    Finished(DesktopExportJobResult),
}

pub(super) fn spawn_desktop_export_job(
    job: DesktopExportQueuedJob,
    editor_manager: Arc<crate::ui::host::EditorManager>,
    sender: Sender<DesktopExportJobMessage>,
) {
    thread::spawn(move || {
        let progress_sender = sender.clone();
        let job_id = job.id;
        let result = editor_manager
            .execute_native_aware_export_build_with_cancellation_and_progress(
                &job.project_root,
                &job.output_root,
                &job.manifest,
                &job.profile_name,
                Some(job.cancel_requested.as_ref()),
                move |progress| {
                    let _ = progress_sender.send(DesktopExportJobMessage::Progress(
                        DesktopExportJobProgress {
                            id: job_id,
                            progress: DesktopExportProgressSnapshot::from_report(progress),
                        },
                    ));
                },
            );
        let _ = sender.send(DesktopExportJobMessage::Finished(DesktopExportJobResult {
            id: job.id,
            profile_name: job.profile_name,
            output_root: job.output_root,
            cancel_requested: job.cancel_requested,
            result,
        }));
    });
}

pub(super) fn desktop_export_summary_from_job_result(
    result: DesktopExportJobResult,
) -> DesktopExportExecutionSummary {
    if result.cancel_requested.load(Ordering::SeqCst) {
        return DesktopExportExecutionSummary::cancelled(
            result.profile_name,
            result.output_root,
            "Export result ignored because cancellation was requested while it was running"
                .to_string(),
        );
    }
    match result.result {
        Ok(report) => DesktopExportExecutionSummary::from_report(result.output_root, report),
        Err(error) => {
            DesktopExportExecutionSummary::failed(result.profile_name, result.output_root, error)
        }
    }
}
