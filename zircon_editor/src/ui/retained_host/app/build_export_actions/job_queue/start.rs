use std::sync::Arc;

use super::super::DesktopExportExecutionSummary;
use super::worker::DesktopExportEditorJob;
use super::{
    DesktopExportActiveJob, DesktopExportJobPhase, DesktopExportJobQueue, DesktopExportJobSnapshot,
    DesktopExportProgressSnapshot,
};
use crate::core::jobs::{EditorJobSpec, JobCategory};

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn start_next(
        &mut self,
        editor_manager: Arc<crate::ui::host::EditorManager>,
    ) -> Option<DesktopExportJobSnapshot> {
        if self.active.is_some() {
            return None;
        }
        let job = self.pending.pop_front()?;
        if job.cancel.is_cancelled() {
            return None;
        }
        let snapshot = DesktopExportJobSnapshot {
            id: job.id,
            profile_name: job.profile_name.clone(),
            output_root: job.output_root.clone(),
            phase: DesktopExportJobPhase::Running,
            progress: Some(DesktopExportProgressSnapshot {
                stage: "queued".to_string(),
                percent: 0,
                message: "Waiting for export runner to start".to_string(),
            }),
        };
        let id = job.id;
        let profile_name = job.profile_name.clone();
        let output_root = job.output_root.clone();
        let cancel = job.cancel.clone();
        let label = format!("Export {profile_name}");
        let editor_job =
            DesktopExportEditorJob::new(job, editor_manager, self.progress_sender.clone());
        match self.jobs.submit(
            EditorJobSpec::new(label, JobCategory::Export).with_cancel(cancel.clone()),
            editor_job,
        ) {
            Ok(ticket) => {
                self.active = Some(DesktopExportActiveJob {
                    id,
                    profile_name,
                    output_root,
                    cancel,
                    progress: snapshot.progress.clone(),
                    ticket,
                });
                Some(snapshot)
            }
            Err(error) => {
                self.completed
                    .push_back(DesktopExportExecutionSummary::failed(
                        profile_name,
                        output_root,
                        format!("failed to submit desktop export job: {error}"),
                    ));
                None
            }
        }
    }
}
