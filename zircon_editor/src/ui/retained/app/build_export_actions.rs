pub(super) const BUILD_EXPORT_ACTION_CONTROL_ID: &str = "BuildExportAction";

#[path = "build_export_actions/action_ids.rs"]
mod action_ids;
#[path = "build_export_actions/error.rs"]
mod error;
#[path = "build_export_actions/execution_summary.rs"]
mod execution_summary;
#[path = "build_export_actions/host_actions.rs"]
mod host_actions;
#[path = "build_export_actions/job_queue.rs"]
mod job_queue;
#[path = "build_export_actions/output_folder.rs"]
mod output_folder;
#[path = "build_export_actions/profiles.rs"]
mod profiles;

pub(super) use action_ids::{parse_build_export_action, BuildExportAction};
use error::DesktopExportActionError;
#[cfg(test)]
pub(super) use execution_summary::DesktopExportExecutionState;
pub(super) use execution_summary::{apply_summary_to_target, DesktopExportExecutionSummary};
pub(super) use job_queue::{
    apply_job_snapshot_to_target, DesktopExportCancellation, DesktopExportJobQueue,
    DesktopExportJobSnapshot,
};
#[cfg(test)]
pub(super) use job_queue::{DesktopExportJobPhase, DesktopExportProgressSnapshot};
pub(super) use profiles::{
    default_desktop_export_output_root, desktop_export_profile, desktop_export_profiles,
    export_platform_label,
};

#[cfg(test)]
#[path = "build_export_actions/tests/cases.rs"]
mod tests;
