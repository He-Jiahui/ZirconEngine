use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::UiReflectedPropertySource,
};

use crate::ui::{
    dispatch::UiTextDocumentSession,
    surface::{
        UiPropertyMutationRequest, UiSurface,
        input::{editable_text_state_for_node, synchronize_text_document},
    },
};

use super::result::unsupported_role_action;
use super::value_target::set_value_property;

use self::number_field::dispatch_number_field_set_value;
use self::payload::set_value_payload;
use self::result::{
    finish_missing_set_value, finish_set_value_mutation, finish_text_input_set_value,
    finish_text_input_set_value_rejection,
};
use self::text::prepare_text_input_set_value;

mod number_field;
mod payload;
mod result;
mod text;

pub(super) fn dispatch_set_value(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    mut text_documents: Option<&mut UiTextDocumentSession>,
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
    let Some(value) = set_value_payload(request, snapshot_node.role) else {
        return finish_missing_set_value(result, target);
    };
    if snapshot_node.role == UiA11yRole::TextInput {
        if surface
            .tree
            .node(target)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(crate::ui::surface::input::is_number_field_metadata)
        {
            return dispatch_number_field_set_value(surface, target, property, value, result);
        }
        let prepared = match prepare_text_input_set_value(surface, target, snapshot_node, value) {
            Ok(prepared) => prepared,
            Err(rejection) => {
                return finish_text_input_set_value_rejection(result, target, rejection);
            }
        };
        if let Some(editable) = editable_text_state_for_node(surface, target) {
            synchronize_text_document(text_documents.as_deref_mut(), surface, target, &editable);
        }
        return finish_text_input_set_value(
            surface,
            text_documents,
            target,
            property,
            prepared.text,
            prepared.committed_edit,
            prepared.constraint_note,
            prepared.constraint_receipt,
            result,
        );
    }

    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(target, property, value.clone())
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    finish_set_value_mutation(surface, target, value, None, None, result, report)
}
