#[path = "snapshot/progress.rs"]
mod progress;
#[path = "snapshot/status.rs"]
mod status;
#[path = "snapshot/target.rs"]
mod target;

use std::path::PathBuf;

pub(in crate::ui::retained_host::app) use target::apply_job_snapshot_to_target;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum DesktopExportJobPhase {
    Queued,
    Running,
    CancelRequested,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct DesktopExportJobSnapshot {
    pub(in crate::ui::retained_host::app) id: u64,
    pub(in crate::ui::retained_host::app) profile_name: String,
    pub(in crate::ui::retained_host::app) output_root: PathBuf,
    pub(in crate::ui::retained_host::app) phase: DesktopExportJobPhase,
    pub(in crate::ui::retained_host::app) progress: Option<DesktopExportProgressSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct DesktopExportProgressSnapshot {
    pub(in crate::ui::retained_host::app) stage: String,
    pub(in crate::ui::retained_host::app) percent: u8,
    pub(in crate::ui::retained_host::app) message: String,
}
