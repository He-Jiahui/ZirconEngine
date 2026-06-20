use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;

use super::super::super::super::{build_export_actions, RetainedEditorHost};

pub(super) fn apply_target_overlays(
    host: &RetainedEditorHost,
    profile_name: &str,
    job_snapshots: &[build_export_actions::DesktopExportJobSnapshot],
    target: &mut BuildExportTargetViewData,
) {
    if let Some(summary) = host.desktop_export_reports.get(profile_name) {
        build_export_actions::apply_summary_to_target(target, summary);
    }
    if let Some(job) = job_snapshots
        .iter()
        .find(|job| job.profile_name == profile_name)
    {
        build_export_actions::apply_job_snapshot_to_target(target, job);
    }
}
