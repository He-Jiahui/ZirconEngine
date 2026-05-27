use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest,
        UiAccessibilityActionStatus, UiAccessibilityNode,
    },
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::UiReflectedPropertySource,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiSurface};

use super::super::{
    result::{finish_unhandled, unsupported_role_action},
    text_state::text_input_is_read_only,
    value_target::set_value_property,
};

use self::replacement::selected_text_replacement;
use self::result::finish_replace_selected_text_mutation;

mod replacement;
mod result;

pub(in crate::ui::accessibility::action) fn dispatch_replace_selected_text(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::ReplaceSelectedText)
    {
        return unsupported_role_action(
            result,
            target,
            "target does not expose replace selected text action",
        );
    }
    if snapshot_node.role != UiA11yRole::TextInput {
        return unsupported_role_action(
            result,
            target,
            "replace selected text requires text input role",
        );
    }
    let Some(property) = set_value_property(surface, target) else {
        return unsupported_role_action(
            result,
            target,
            "target has no mutable value or text property",
        );
    };
    let Some(replacement) = request
        .value
        .clone()
        .or_else(|| request.numeric_value.map(|value| value.to_string()))
    else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "missing_value",
            "replace selected text action requires value or numeric_value",
        );
    };
    if text_input_is_read_only(surface, target) {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "read_only",
            "text input is read-only",
        );
    }

    let replacement = selected_text_replacement(surface, target, snapshot_node, &replacement);
    let text_constraint_note = replacement.constraint_note;
    let caret_offset = replacement.caret_offset;
    let value = UiValue::String(replacement.text);

    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(target, property, value.clone())
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    finish_replace_selected_text_mutation(
        surface,
        target,
        value,
        caret_offset,
        text_constraint_note,
        result,
        report,
    )
}
