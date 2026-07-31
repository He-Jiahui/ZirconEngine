pub(super) const BUILD_EXPORT_ACTION_CONTROL_ID: &str = "BuildExportAction";

mod action_ids;
mod error;
mod execution_summary;
mod host_actions;
mod job_queue;
mod output_folder;
mod profiles;

pub(super) use action_ids::{BuildExportAction, parse_build_export_action};
use error::DesktopExportActionError;
#[cfg(test)]
pub(super) use execution_summary::DesktopExportExecutionState;
pub(super) use execution_summary::{DesktopExportExecutionSummary, apply_summary_to_target};
pub(super) use job_queue::{
    DesktopExportCancellation, DesktopExportJobQueue, DesktopExportJobSnapshot,
    apply_job_snapshot_to_target,
};
#[cfg(test)]
pub(super) use job_queue::{DesktopExportJobPhase, DesktopExportProgressSnapshot};
pub(super) use profiles::{
    default_desktop_export_output_root, desktop_export_profile, desktop_export_profiles,
    export_platform_label,
};

#[cfg(test)]
mod tests;
