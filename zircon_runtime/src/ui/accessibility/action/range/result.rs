use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, dispatch::UiInputDispatchResult,
    event_ui::UiNodeId, tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationReport, UiPropertyMutationStatus};

use super::super::result::{
    append_binding_report_diagnostic, finish_handled, finish_unhandled, unsupported_role_action,
};
use super::adjustment::value_changed_event;

// 以事务报告决定是否发布 ValueChanged；到边界的未变化操作仍可被消费，不重复发值变更事件。
pub(super) fn finish_adjust_value_mutation(
    target: UiNodeId,
    result: UiInputDispatchResult,
    report: Result<Option<(UiPropertyMutationReport, f64)>, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(Some((report, value)))
            if matches!(report.status, UiPropertyMutationStatus::Accepted) =>
        {
            finish_accepted_adjust_value(target, result, report, value)
        }
        Ok(Some((report, _value)))
            if matches!(report.status, UiPropertyMutationStatus::Unchanged) =>
        {
            finish_unchanged_adjust_value(target, result, report)
        }
        Ok(Some((report, _value))) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("adjust value mutation was rejected"),
        ),
        Ok(None) => unsupported_role_action(result, target, "target has no range value contract"),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("adjust value mutation failed: {error}"),
        ),
    }
}

fn finish_accepted_adjust_value(
    target: UiNodeId,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
    value: f64,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.adjust_value");
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    append_binding_report_diagnostic(&mut result, &report);
    result
        .component_events
        .push(value_changed_event(target, report.property, value));
    result
}

// BUG: [CR-UI-COMP-0003] 滑块步进未变化时仍输出 property_changed，导致诊断与 Unchanged 绑定报告矛盾。
// 证据：上层仅在 UiPropertyMutationStatus::Unchanged 时调用此函数；应记录未变化而非改值。
fn finish_unchanged_adjust_value(
    target: UiNodeId,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.adjust_value");
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    append_binding_report_diagnostic(&mut result, &report);
    result
}
