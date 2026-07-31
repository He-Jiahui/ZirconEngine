use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;

use super::super::super::super::{RetainedEditorHost, build_export_actions};
use super::super::diagnostics::prepend_desktop_export_output_diagnostic;

pub(in super::super) fn apply_target_overlays(
    host: &RetainedEditorHost,
    project_root: &Path,
    job_snapshots: &[build_export_actions::DesktopExportJobSnapshot],
    target: &mut BuildExportTargetViewData,
) {
    let profile_name = target.profile_name.to_string();
    let output_root = host.effective_desktop_export_output_root(project_root, &profile_name);
    target.diagnostics = prepend_desktop_export_output_diagnostic(
        output_root.as_path(),
        target.diagnostics.to_string(),
    )
    .into();
    if let Some(summary) = host.desktop_export_reports.get(profile_name.as_str()) {
        build_export_actions::apply_summary_to_target(target, summary);
    }
    if let Some(job) = job_snapshots
        .iter()
        .find(|job| job.profile_name == profile_name)
    {
        build_export_actions::apply_job_snapshot_to_target(target, job);
    }
}
