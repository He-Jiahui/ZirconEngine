mod cancellation;
mod enqueue;
mod queries;
#[path = "job_queue/snapshot.rs"]
mod snapshot;
mod start;
mod state;
mod updates;
mod worker;

pub(in crate::ui::retained_host::app) use cancellation::DesktopExportCancellation;
#[cfg(test)]
pub(in crate::ui::retained_host::app) use snapshot::desktop_export_status_task_from_job;
pub(in crate::ui::retained_host::app) use snapshot::{
    apply_job_snapshot_to_target, desktop_export_status_task_from_queue, DesktopExportJobPhase,
    DesktopExportJobSnapshot, DesktopExportProgressSnapshot,
};
pub(in crate::ui::retained_host::app) use state::DesktopExportJobQueue;
use state::{DesktopExportActiveJob, DesktopExportQueuedJob};
