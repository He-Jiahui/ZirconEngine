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
pub(in crate::ui::retained_host::app) use snapshot::{
    DesktopExportJobPhase, DesktopExportJobSnapshot, DesktopExportProgressSnapshot,
    apply_job_snapshot_to_target,
};
pub(in crate::ui::retained_host::app) use state::DesktopExportJobQueue;
use state::{DesktopExportActiveJob, DesktopExportQueuedJob};
