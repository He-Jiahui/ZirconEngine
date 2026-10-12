use super::status::{summary_pane_diagnostics, summary_status_label};
use super::DesktopExportExecutionSummary;

pub(in crate::ui::retained_host::app) fn apply_summary_to_target(
    target: &mut crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
    summary: &DesktopExportExecutionSummary,
) {
    target.status = summary_status_label(summary).into();
    target.generated_files = summary.generated_files.to_string().into();
    target.native_dynamic_packages = summary.copied_packages.to_string().into();
    target.diagnostics = summary_pane_diagnostics(summary).into();
    target.fatal = target.fatal || summary.fatal();
}
