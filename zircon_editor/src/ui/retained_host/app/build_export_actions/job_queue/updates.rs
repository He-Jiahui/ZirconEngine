use super::super::DesktopExportExecutionSummary;
use super::worker::{DesktopExportJobProgress, desktop_export_summary_from_job_result};
use super::{DesktopExportActiveJob, DesktopExportJobQueue, DesktopExportProgressSnapshot};
use crate::core::jobs::JobError;
use std::sync::mpsc::Receiver;

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn poll_updates(
        &mut self,
    ) -> (Vec<DesktopExportExecutionSummary>, bool) {
        let mut summaries = self.completed.drain(..).collect::<Vec<_>>();
        let mut changed = false;
        if let Some(active) = self.active.as_mut() {
            changed |=
                drain_progress_for_active(&self.progress_receiver, active.id, &mut active.progress);
        }
        let terminal = self
            .active
            .as_ref()
            .and_then(|active| active.ticket.try_take());
        if let Some(result) = terminal {
            if let Some(active) = self.active.as_mut() {
                changed |= drain_progress_for_active(
                    &self.progress_receiver,
                    active.id,
                    &mut active.progress,
                );
            }
            if let Some(active) = self.active.take() {
                summaries.push(summary_from_ticket_result(active, result));
                changed = true;
            }
        }
        changed |= !summaries.is_empty();
        (summaries, changed)
    }
}

fn drain_progress_for_active(
    receiver: &Receiver<DesktopExportJobProgress>,
    active_id: u64,
    active_progress: &mut Option<DesktopExportProgressSnapshot>,
) -> bool {
    let mut changed = false;
    while let Ok(progress) = receiver.try_recv() {
        if progress.id == active_id {
            *active_progress = Some(progress.progress);
            changed = true;
        }
    }
    changed
}

fn summary_from_ticket_result(
    active: DesktopExportActiveJob,
    result: Result<super::worker::DesktopExportJobResult, JobError>,
) -> DesktopExportExecutionSummary {
    match result {
        Ok(result) if result.id == active.id => desktop_export_summary_from_job_result(result),
        Ok(result) => DesktopExportExecutionSummary::failed(
            active.profile_name,
            active.output_root,
            format!(
                "desktop export ticket returned job {} for active job {}",
                result.id, active.id
            ),
        ),
        Err(JobError::Cancelled) => cancelled_active_summary(active),
        Err(error) => DesktopExportExecutionSummary::failed(
            active.profile_name,
            active.output_root,
            format!("desktop export job failed: {error}"),
        ),
    }
}

fn cancelled_active_summary(active: DesktopExportActiveJob) -> DesktopExportExecutionSummary {
    DesktopExportExecutionSummary::cancelled(
        active.profile_name,
        active.output_root,
        "Export result ignored because cancellation was requested while it was running".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::drain_progress_for_active;
    use crate::ui::retained_host::app::build_export_actions::{
        DesktopExportProgressSnapshot, job_queue::worker::DesktopExportJobProgress,
    };

    #[test]
    fn terminal_poll_can_drain_progress_sent_after_the_initial_drain() {
        let (sender, receiver) = mpsc::channel();
        let mut progress = None;
        assert!(!drain_progress_for_active(&receiver, 7, &mut progress));

        sender
            .send(DesktopExportJobProgress {
                id: 7,
                progress: DesktopExportProgressSnapshot {
                    stage: "complete".to_string(),
                    percent: 100,
                    message: "Desktop export build finished".to_string(),
                },
            })
            .expect("test progress channel should remain connected");

        assert!(drain_progress_for_active(&receiver, 7, &mut progress));
        assert_eq!(progress.map(|snapshot| snapshot.percent), Some(100));
    }
}
