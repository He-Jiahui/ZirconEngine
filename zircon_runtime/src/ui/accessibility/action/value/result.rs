use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiComponentEventReport, UiInputDispatchResult, UiTextInputConstraintReceipt},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use crate::ui::surface::{
    UiPropertyMutationReport, UiPropertyMutationStatus, UiSurface, UiTextComponentEventKind,
};
use crate::ui::{dispatch::UiTextDocumentSession, text::CommittedTextEditIntent};

use super::super::{
    result::{append_binding_report_diagnostic, finish_handled, finish_unhandled},
    text_state::{collapsed_text_state, commit_accessibility_text_edit},
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

pub(super) fn finish_text_input_set_value(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    target: UiNodeId,
    value_property: String,
    text: String,
    committed_edit: Option<CommittedTextEditIntent>,
    text_constraint_note: Option<&'static str>,
    text_constraint_receipt: Option<UiTextInputConstraintReceipt>,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    append_text_constraint_receipt(&mut result, text_constraint_note, text_constraint_receipt);
    let collapse_offset = text.len();
    let state = match collapsed_text_state(surface, target, text, collapse_offset) {
        Ok(state) => state,
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                error.diagnostic_code(),
                error.reason(),
            );
        }
    };
    let value_changed = match commit_accessibility_text_edit(
        surface,
        text_documents,
        target,
        value_property.as_str(),
        &state,
        committed_edit,
        &mut result,
    ) {
        Ok(value_changed) => value_changed,
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                error.diagnostic_code(),
                error.reason(),
            );
        }
    };

    let mut result = finish_handled(result, target, "accessibility.set_value");
    result.diagnostics.notes.push(format!(
        "accessibility_property_{}:{value_property}",
        if value_changed {
            "changed"
        } else {
            "unchanged"
        }
    ));
    if value_changed {
        let event = surface.text_component_event(
            target,
            value_property,
            state.text,
            UiTextComponentEventKind::Change,
        );
        result.component_events.push(UiComponentEventReport {
            target,
            event,
            delivered: true,
            drag: None,
            template_action: None,
        });
    }
    surface.redact_secure_text_result(target, &mut result);
    result
}

pub(super) fn finish_set_value_mutation(
    surface: &mut UiSurface,
    target: UiNodeId,
    value: UiValue,
    text_constraint_note: Option<&'static str>,
    text_constraint_receipt: Option<UiTextInputConstraintReceipt>,
    result: UiInputDispatchResult,
    report: Result<UiPropertyMutationReport, UiTreeError>,
) -> UiInputDispatchResult {
    let mut result = result;
    append_text_constraint_receipt(&mut result, text_constraint_note, text_constraint_receipt);
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            finish_accepted_set_value(surface, target, value, result, report)
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            finish_unchanged_set_value(surface, target, result, report)
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
    value: UiValue,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.set_value");
    result.diagnostics.notes.push(format!(
        "accessibility_property_changed:{}:{:?}",
        report.property, report.invalidation.dirty
    ));
    append_binding_report_diagnostic(&mut result, &report);
    let event = UiComponentEvent::ValueChanged {
        property: report.property.clone(),
        value,
    };
    result.component_events.push(UiComponentEventReport {
        target,
        event,
        delivered: true,
        drag: None,
        template_action: None,
    });
    surface.redact_secure_text_result(target, &mut result);
    result
}

fn finish_unchanged_set_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    result: UiInputDispatchResult,
    report: UiPropertyMutationReport,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.set_value");
    result.diagnostics.notes.push(format!(
        "accessibility_property_unchanged:{}",
        report.property
    ));
    append_binding_report_diagnostic(&mut result, &report);
    surface.redact_secure_text_result(target, &mut result);
    result
}

fn append_text_constraint_receipt(
    result: &mut UiInputDispatchResult,
    note: Option<&'static str>,
    receipt: Option<UiTextInputConstraintReceipt>,
) {
    result.diagnostics.text_constraint = receipt;
    if let Some(note) = note {
        result.diagnostics.notes.push(note.to_string());
    }
}
