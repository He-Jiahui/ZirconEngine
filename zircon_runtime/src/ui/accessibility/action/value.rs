use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    dispatch::UiInputDispatchResult,
    event_ui::UiReflectedPropertySource,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiSurface};

use super::result::unsupported_role_action;
use super::value_target::set_value_property;

use self::payload::set_value_payload;
use self::result::{
    finish_missing_set_value, finish_set_value_mutation, finish_text_input_set_value_rejection,
};
use self::text::prepare_text_input_set_value;

mod payload;
mod result;
mod text;

pub(super) fn dispatch_set_value(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::SetValue)
    {
        return unsupported_role_action(result, target, "target does not expose set value action");
    }
    if !matches!(
        snapshot_node.role,
        UiA11yRole::TextInput | UiA11yRole::Slider
    ) {
        return unsupported_role_action(
            result,
            target,
            "set value requires text input or slider role",
        );
    }
    let Some(property) = set_value_property(surface, target) else {
        return unsupported_role_action(
            result,
            target,
            "target has no mutable value or text property",
        );
    };
    let Some(mut value) = set_value_payload(request, snapshot_node.role) else {
        return finish_missing_set_value(result, target);
    };
    let mut text_constraint_note = None;
    if snapshot_node.role == UiA11yRole::TextInput {
        let prepared = match prepare_text_input_set_value(surface, target, snapshot_node, value) {
            Ok(prepared) => prepared,
            Err(rejection) => {
                return finish_text_input_set_value_rejection(result, target, rejection);
            }
        };
        value = prepared.value;
        text_constraint_note = prepared.constraint_note;
    }

    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(target, property, value.clone())
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    finish_set_value_mutation(
        surface,
        target,
        snapshot_node,
        value,
        text_constraint_note,
        result,
        report,
    )
}
