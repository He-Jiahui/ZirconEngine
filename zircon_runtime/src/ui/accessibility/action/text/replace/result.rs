use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    dispatch::{UiComponentEventReport, UiInputDispatchResult, UiTextInputConstraintReceipt},
    event_ui::UiNodeId,
};

use crate::ui::surface::{UiSurface, UiTextComponentEventKind};
use crate::ui::{dispatch::UiTextDocumentSession, text::CommittedTextEditIntent};

use super::super::super::{
    result::{finish_handled, finish_unhandled},
    text_state::{collapsed_text_state, commit_accessibility_text_edit},
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
    text_documents: Option<&mut UiTextDocumentSession>,
    target: UiNodeId,
    value_property: String,
    text: String,
    caret_offset: usize,
    committed_edit: Option<CommittedTextEditIntent>,
    text_constraint_note: Option<&'static str>,
    text_constraint_receipt: Option<UiTextInputConstraintReceipt>,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    append_text_constraint_receipt(&mut result, text_constraint_note, text_constraint_receipt);
    let state = match collapsed_text_state(surface, target, text, caret_offset) {
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

    let mut result = finish_handled(result, target, "accessibility.replace_selected_text");
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
