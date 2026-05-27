use zircon_runtime_interface::ui::dispatch::UiInputDispatchResult;

use crate::ui::surface::UiPropertyMutationReport;

pub(in crate::ui::accessibility::action) fn append_binding_report_diagnostic(
    result: &mut UiInputDispatchResult,
    report: &UiPropertyMutationReport,
) {
    result.record_binding_report(report.binding.clone());
    result.diagnostics.notes.push(format!(
        "accessibility_binding_updates:applied={},unchanged={},rejected={}",
        report.binding.applied_count, report.binding.unchanged_count, report.binding.rejected_count
    ));
    if let Some(update) = report.binding.updates.first() {
        result.diagnostics.notes.push(format!(
            "accessibility_binding_source:{:?}",
            update.source.kind
        ));
    }
}
