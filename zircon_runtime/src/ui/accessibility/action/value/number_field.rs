use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    binding::UiBindingSourceKind,
    component::UiValue,
    dispatch::{
        UiComponentEventReport, UiInputDispatchResult, UiNumberInputCommitMethod,
        UiNumberInputCommitStatus,
    },
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity},
};

use crate::ui::surface::{
    UiSurface, UiTextComponentEventKind,
    input::{
        editable_text_state_for_node, number_field_commit_decision, prepare_number_field_properties,
    },
};

use super::super::result::{finish_handled, finish_unhandled};

pub(super) fn dispatch_number_field_set_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: String,
    value: UiValue,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let Some(current) = editable_text_state_for_node(surface, target) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "number_state_unavailable",
            "number field editable state is unavailable",
        );
    };
    if current.read_only {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "read_only",
            "number field is read-only",
        );
    }
    let text = value.display_text();
    let Some(decision) = number_field_commit_decision(
        surface,
        target,
        &text,
        UiNumberInputCommitMethod::Accessibility,
    ) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "number_metadata_invalid",
            "number field metadata is incomplete or invalid",
        );
    };
    result.diagnostics.number_input = Some(decision.receipt);
    if decision.receipt.commit_status == UiNumberInputCommitStatus::Rejected {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "number_value_rejected",
            "number field value failed invariant parsing or policy validation",
        );
    }

    let state = UiEditableTextState {
        caret: UiTextCaret {
            offset: decision.text.len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: None,
        composition: None,
        read_only: false,
        text: decision.text,
    };
    let transaction = match prepare_number_field_properties(
        surface,
        target,
        value_property.as_str(),
        UiValue::Float(decision.value),
        &state,
        false,
        UiBindingSourceKind::AccessibilityAction,
    )
    .and_then(|prepared| prepared.commit())
    {
        Ok(transaction) => transaction,
        Err(error) => {
            if let Some(receipt) = result.diagnostics.number_input.as_mut() {
                receipt.commit_status = UiNumberInputCommitStatus::Rejected;
            }
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                error.diagnostic_code(),
                "number field property transaction was rejected",
            );
        }
    };

    let value_changed = transaction.value_changed;
    if let Some(binding_report) = transaction.binding_report {
        result.record_binding_report(binding_report);
    }
    for (property, dirty) in transaction.changed_properties {
        result.diagnostics.notes.push(format!(
            "accessibility_number_property_changed:{property}:{dirty:?}"
        ));
    }
    let mut result = finish_handled(result, target, "accessibility.set_value");
    if value_changed {
        let event = surface.component_value_event(
            target,
            value_property,
            UiValue::Float(decision.value),
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
