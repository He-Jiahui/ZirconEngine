use std::sync::{atomic::Ordering, Arc};

use super::worker::spawn_desktop_export_job;
use super::{
    DesktopExportActiveJob, DesktopExportJobPhase, DesktopExportJobQueue, DesktopExportJobSnapshot,
    DesktopExportProgressSnapshot,
};

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn start_next(
        &mut self,
        editor_manager: Arc<crate::ui::host::EditorManager>,
    ) -> Option<DesktopExportJobSnapshot> {
        if self.active.is_some() {
            return None;
        }
        let job = self.pending.pop_front()?;
        if job.cancel_requested.load(Ordering::SeqCst) {
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
        self.active = Some(DesktopExportActiveJob {
            id: job.id,
            profile_name: job.profile_name.clone(),
            output_root: job.output_root.clone(),
            cancel_requested: job.cancel_requested.clone(),
            progress: snapshot.progress.clone(),
        });

        spawn_desktop_export_job(job, editor_manager, self.sender.clone());
        Some(snapshot)
    }
}
