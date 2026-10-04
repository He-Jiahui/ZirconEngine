use super::super::DesktopExportExecutionSummary;
use super::worker::{desktop_export_summary_from_job_result, DesktopExportJobProgress};
use super::{DesktopExportActiveJob, DesktopExportJobQueue, DesktopExportProgressSnapshot};
use crate::core::jobs::JobError;
use std::sync::mpsc::Receiver;

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn poll_updates(
        &mut self,
    ) -> (Vec<DesktopExportExecutionSummary>, bool) {
        let mut summaries = take_completed(&mut self.completed);
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

fn take_completed<T>(completed: &mut std::collections::VecDeque<T>) -> Vec<T> {
    std::mem::take(completed).into()
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
        Err(error) => {
            if let Some(crate::ui::host::EditorExportBuildError::ReportFailed { report }) =
                error.downcast_ref::<crate::ui::host::EditorExportBuildError>()
            {
                return DesktopExportExecutionSummary::from_report(
                    active.output_root,
                    (**report).clone(),
                );
            }
            DesktopExportExecutionSummary::failed(
                active.profile_name,
                active.output_root,
                format!("desktop export job failed: {error}"),
            )
        }
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
#[path = "tests/updates.rs"]
mod tests;

#[cfg(test)]
#[path = "updates/tests/completed_take_tests.rs"]
mod completed_take_tests;
