use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    component::UiComponentEvent,
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationReport, UiPropertyMutationStatus};

use super::super::result::{
    append_binding_report_diagnostic, finish_handled, finish_unhandled, finish_unhandled_with_note,
};

pub(super) fn finish_tooltip_dismiss(
    result: UiInputDispatchResult,
    target: UiNodeId,
    tooltip_id: String,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.dismiss_tooltip");
    result
        .diagnostics
        .notes
        .push(format!("accessibility_tooltip_hidden:{tooltip_id}"));
    result
}

pub(super) fn finish_popup_dismiss_lookup_error(
    result: UiInputDispatchResult,
    target: UiNodeId,
    error: UiTreeError,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "mutation_error",
        &format!("dismiss popup lookup failed: {error}"),
    )
}

pub(super) fn finish_unsupported_dismiss(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled_with_note(
        result,
        Some(target),
        UiAccessibilityActionStatus::Unsupported,
        "unsupported_role_action",
        "accessibility dismiss requires popup id",
        "accessibility dismiss requires popup id",
    )
}

pub(super) fn finish_popup_dismiss_mutation(
    popup_id: UiNodeId,
    result: UiInputDispatchResult,
    report: Result<UiPropertyMutationReport, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            finish_accepted_popup_dismiss(popup_id, result, report)
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            finish_unchanged_popup_dismiss(popup_id, result, report)
        }
        Ok(report) => finish_unhandled(
            result,
            Some(popup_id),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("dismiss mutation was rejected"),
        ),
        Err(error) => finish_unhandled(
            result,
            Some(popup_id),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("dismiss mutation failed: {error}"),
        ),
    }
}

fn finish_accepted_popup_dismiss(
    popup_id: UiNodeId,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, popup_id, "accessibility.dismiss");
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    result.component_events.push(UiComponentEventReport {
        target: popup_id,
        event: UiComponentEvent::ClosePopup,
        delivered: true,
        drag: None,
    });
    append_binding_report_diagnostic(&mut result, &report);
    result
}

fn finish_unchanged_popup_dismiss(
    popup_id: UiNodeId,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, popup_id, "accessibility.dismiss");
    result.diagnostics.notes.push(format!(
        "accessibility_property_unchanged:{}",
        report.property
    ));
    append_binding_report_diagnostic(&mut result, &report);
    result
}
