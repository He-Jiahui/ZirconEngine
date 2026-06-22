use super::super::DesktopExportJobQueue;
use super::status::job_status_task_detail;
use super::{DesktopExportJobPhase, DesktopExportJobSnapshot};
use crate::ui::workbench::snapshot::{StatusTaskProgressSnapshot, StatusTaskProgressTone};

pub(in crate::ui::retained_host::app) fn desktop_export_status_task_from_queue(
    queue: &DesktopExportJobQueue,
) -> Option<StatusTaskProgressSnapshot> {
    queue
        .snapshots()
        .first()
        .map(desktop_export_status_task_from_job)
}

pub(in crate::ui::retained_host::app) fn desktop_export_status_task_from_job(
    snapshot: &DesktopExportJobSnapshot,
) -> StatusTaskProgressSnapshot {
    let tone = match snapshot.phase {
        DesktopExportJobPhase::Queued | DesktopExportJobPhase::Running => {
            StatusTaskProgressTone::Info
        }
        DesktopExportJobPhase::CancelRequested => StatusTaskProgressTone::Warning,
    };
    let percent = snapshot.progress.as_ref().map(|progress| progress.percent);
    StatusTaskProgressSnapshot::new(
        format!("desktop_export:{}", snapshot.id),
        format!("Export {}", snapshot.profile_name),
    )
    .with_detail(job_status_task_detail(snapshot))
    .with_percent(percent)
    .with_tone(tone)
}
