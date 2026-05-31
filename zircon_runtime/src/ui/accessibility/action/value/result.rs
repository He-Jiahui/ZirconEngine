use zircon_runtime_interface::ui::{
    accessibility::{UiA11yRole, UiAccessibilityActionStatus, UiAccessibilityNode},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationReport, UiPropertyMutationStatus, UiSurface};

use super::super::{
    result::{append_binding_report_diagnostic, finish_handled, finish_unhandled},
    text_state::sync_text_input_set_value_edit_metadata,
};
use super::text::TextInputSetValueRejection;

pub(super) fn finish_missing_set_value(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "missing_value",
        "set value action requires value or numeric_value",
    )
}

pub(super) fn finish_text_input_set_value_rejection(
    result: UiInputDispatchResult,
    target: UiNodeId,
    rejection: TextInputSetValueRejection,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        rejection.code,
        rejection.reason,
    )
}

pub(super) fn finish_set_value_mutation(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    value: UiValue,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: Result<UiPropertyMutationReport, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            finish_accepted_set_value(
                surface,
                target,
                snapshot_node,
                value,
                text_constraint_note,
                result,
                report,
            )
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            finish_unchanged_set_value(
                surface,
                target,
                snapshot_node,
                &value,
                text_constraint_note,
                result,
                report,
            )
        }
        Ok(report) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("set value mutation was rejected"),
        ),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("set value mutation failed: {error}"),
        ),
    }
}

fn finish_accepted_set_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    value: UiValue,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.set_value");
    append_text_constraint_note(&mut result, text_constraint_note);
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    append_binding_report_diagnostic(&mut result, &report);
    if snapshot_node.role == UiA11yRole::TextInput {
        sync_text_input_set_value_edit_metadata(surface, target, &value, &mut result);
    }
    result.component_events.push(UiComponentEventReport {
        target,
        event: UiComponentEvent::ValueChanged {
            property: report.property,
            value,
        },
        delivered: true,
        drag: None,
    });
    result
}

fn finish_unchanged_set_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    value: &UiValue,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.set_value");
    append_text_constraint_note(&mut result, text_constraint_note);
    result.diagnostics.notes.push(format!(
        "accessibility_property_unchanged:{}",
        report.property
    ));
    append_binding_report_diagnostic(&mut result, &report);
    if snapshot_node.role == UiA11yRole::TextInput {
        sync_text_input_set_value_edit_metadata(surface, target, value, &mut result);
    }
    result
}

fn append_text_constraint_note(result: &mut UiInputDispatchResult, note: Option<&'static str>) {
    if let Some(note) = note {
        result.diagnostics.notes.push(note.to_string());
    }
}
