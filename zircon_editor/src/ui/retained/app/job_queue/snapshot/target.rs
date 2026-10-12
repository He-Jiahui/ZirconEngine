use super::status::{job_pane_diagnostics, job_status_label};
use super::DesktopExportJobSnapshot;

pub(in crate::ui::retained_host::app) fn apply_job_snapshot_to_target(
    target: &mut crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
    snapshot: &DesktopExportJobSnapshot,
) {
    target.status = job_status_label(snapshot).into();
    target.diagnostics = job_pane_diagnostics(snapshot).into();
    target.fatal = false;
}
