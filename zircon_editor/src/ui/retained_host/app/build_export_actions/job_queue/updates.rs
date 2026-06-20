use super::super::DesktopExportExecutionSummary;
use super::worker::{desktop_export_summary_from_job_result, DesktopExportJobMessage};
use super::DesktopExportJobQueue;

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn poll_updates(
        &mut self,
    ) -> (Vec<DesktopExportExecutionSummary>, bool) {
        let mut summaries = Vec::new();
        let mut changed = false;
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                DesktopExportJobMessage::Progress(progress) => {
                    if let Some(active) = self
                        .active
                        .as_mut()
                        .filter(|active| active.id == progress.id)
                    {
                        active.progress = Some(progress.progress);
                        changed = true;
                    }
                }
                DesktopExportJobMessage::Finished(result) => {
                    if self
                        .active
                        .as_ref()
                        .is_some_and(|active| active.id == result.id)
                    {
                        self.active = None;
                    }
                    summaries.push(desktop_export_summary_from_job_result(result));
                    changed = true;
                }
            }
        }
        (summaries, changed)
    }
}
