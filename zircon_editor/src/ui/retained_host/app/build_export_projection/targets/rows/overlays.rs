use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;

use super::super::super::super::{build_export_actions, RetainedEditorHost};
use super::super::diagnostics::prepend_desktop_export_output_diagnostic;

pub(in super::super) fn apply_target_overlays(
    host: &RetainedEditorHost,
    project_root: &Path,
    job_snapshots: &[build_export_actions::DesktopExportJobSnapshot],
    target: &mut BuildExportTargetViewData,
) {
    let profile_name = target.profile_name.as_str();
    let output_root = host.effective_desktop_export_output_root(project_root, profile_name);
    let summary = host.desktop_export_reports.get(profile_name);
    let job = job_snapshots
        .iter()
        .find(|job| job.profile_name == profile_name);
    let diagnostics = prepend_desktop_export_output_diagnostic(
        output_root.as_path(),
        target.diagnostics.to_string(),
    );

    target.diagnostics = diagnostics.into();
    if let Some(summary) = summary {
        build_export_actions::apply_summary_to_target(target, summary);
    }
    if let Some(job) = job {
        build_export_actions::apply_job_snapshot_to_target(target, job);
    }
}

#[cfg(test)]
#[path = "tests/overlays_performance_tests.rs"]
mod performance_tests;
