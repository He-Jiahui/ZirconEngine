use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationReport, UiPropertyMutationStatus, UiSurface};

use super::super::super::{
    result::{append_binding_report_diagnostic, finish_handled, finish_unhandled},
    text_state::sync_text_input_edit_metadata,
};

pub(super) fn finish_missing_replace_selected_text(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "missing_value",
        "replace selected text action requires value or numeric_value",
    )
}

pub(super) fn finish_read_only_replace_selected_text(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "read_only",
        "text input is read-only",
    )
}

pub(super) fn finish_replace_selected_text_mutation(
    surface: &mut UiSurface,
    target: UiNodeId,
    value: UiValue,
    caret_offset: usize,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: Result<UiPropertyMutationReport, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            finish_accepted_replace_selected_text(
                surface,
                target,
                value,
                caret_offset,
                text_constraint_note,
                result,
                report,
            )
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            finish_unchanged_replace_selected_text(
                surface,
                target,
                caret_offset,
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
                .unwrap_or("replace selected text mutation was rejected"),
        ),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("replace selected text mutation failed: {error}"),
        ),
    }
}

fn finish_accepted_replace_selected_text(
    surface: &mut UiSurface,
    target: UiNodeId,
    value: UiValue,
    caret_offset: usize,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.replace_selected_text");
    append_text_constraint_note(&mut result, text_constraint_note);
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    append_binding_report_diagnostic(&mut result, &report);
    sync_text_input_edit_metadata(surface, target, caret_offset, &mut result);
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

fn finish_unchanged_replace_selected_text(
    surface: &mut UiSurface,
    target: UiNodeId,
    caret_offset: usize,
    text_constraint_note: Option<&'static str>,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.replace_selected_text");
    append_text_constraint_note(&mut result, text_constraint_note);
    result.diagnostics.notes.push(format!(
        "accessibility_property_unchanged:{}",
        report.property
    ));
    append_binding_report_diagnostic(&mut result, &report);
    sync_text_input_edit_metadata(surface, target, caret_offset, &mut result);
    result
}

fn append_text_constraint_note(result: &mut UiInputDispatchResult, note: Option<&'static str>) {
    if let Some(note) = note {
        result.diagnostics.notes.push(note.to_string());
    }
}
