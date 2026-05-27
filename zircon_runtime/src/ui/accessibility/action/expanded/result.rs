use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    component::UiComponentEvent,
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationReport, UiPropertyMutationStatus};

use super::super::result::{append_binding_report_diagnostic, finish_handled, finish_unhandled};

pub(super) fn finish_expanded_mutation(
    target: UiNodeId,
    phase: &str,
    event: UiComponentEvent,
    result: UiInputDispatchResult,
    report: Result<UiPropertyMutationReport, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            finish_accepted_expanded_mutation(target, phase, event, result, report)
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            finish_unchanged_expanded_mutation(target, phase, result, report)
        }
        Ok(report) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("expand/collapse mutation was rejected"),
        ),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("expand/collapse mutation failed: {error}"),
        ),
    }
}

fn finish_accepted_expanded_mutation(
    target: UiNodeId,
    phase: &str,
    event: UiComponentEvent,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, phase);
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    result.component_events.push(UiComponentEventReport {
        target,
        event,
        delivered: true,
        drag: None,
    });
    append_binding_report_diagnostic(&mut result, &report);
    result
}

fn finish_unchanged_expanded_mutation(
    target: UiNodeId,
    phase: &str,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, phase);
    result.diagnostics.notes.push(format!(
        "accessibility_property_unchanged:{}",
        report.property
    ));
    append_binding_report_diagnostic(&mut result, &report);
    result
}
