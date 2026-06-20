pub(super) const BUILD_EXPORT_ACTION_CONTROL_ID: &str = "BuildExportAction";

mod action_ids;
mod execution_summary;
mod host_actions;
mod job_queue;
mod output_folder;
mod profiles;

pub(super) use action_ids::{parse_build_export_action, BuildExportAction};
#[cfg(test)]
pub(super) use execution_summary::DesktopExportExecutionState;
pub(super) use execution_summary::{apply_summary_to_target, DesktopExportExecutionSummary};
pub(super) use job_queue::{
    apply_job_snapshot_to_target, desktop_export_status_task_from_queue, DesktopExportCancellation,
    DesktopExportJobQueue, DesktopExportJobSnapshot,
};
pub(super) use profiles::{
    default_desktop_export_output_root, desktop_export_profile, desktop_export_profiles,
    export_platform_label,
};

#[cfg(test)]
mod tests;
