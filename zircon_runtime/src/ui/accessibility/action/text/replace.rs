use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    dispatch::UiInputDispatchResult,
};

use crate::ui::{
    dispatch::UiTextDocumentSession,
    surface::{
        UiSurface,
        input::{
            editable_text_state_for_node, retained_grapheme_count_for_constraints,
            synchronize_text_document,
        },
        text_input_constraints_for_node,
    },
};

use super::super::{
    result::unsupported_role_action,
    text_state::{text_input_is_read_only, text_selection_range},
    value_target::set_value_property,
};

use self::replacement::selected_text_replacement;
use self::result::{
    finish_missing_replace_selected_text, finish_read_only_replace_selected_text,
    finish_replace_selected_text_mutation,
};

mod replacement;
mod result;

pub(in crate::ui::accessibility::action) fn dispatch_replace_selected_text(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    mut text_documents: Option<&mut UiTextDocumentSession>,
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
        return finish_missing_replace_selected_text(result, target);
    };
    if text_input_is_read_only(surface, target) {
        return finish_read_only_replace_selected_text(result, target);
    }
    let editable = editable_text_state_for_node(surface, target);
    if let Some(editable) = editable.as_ref() {
        synchronize_text_document(text_documents.as_deref_mut(), surface, target, editable);
    }

    let current_text = snapshot_node.state.value.as_deref().unwrap_or_default();
    let selected_range =
        text_selection_range(current_text, snapshot_node.state.text_selection.as_ref());
    let constraints = text_input_constraints_for_node(surface, target);
    let retained_graphemes = editable.as_ref().map_or(
        crate::ui::surface::input::TextInputRetainedGraphemeCount::SourceScan,
        |editable| {
            if editable.composition.is_some() {
                return crate::ui::surface::input::TextInputRetainedGraphemeCount::SourceScan;
            }
            retained_grapheme_count_for_constraints(
                text_documents.as_deref_mut(),
                surface,
                target,
                selected_range,
                constraints,
            )
        },
    );
    let replacement = selected_text_replacement(
        snapshot_node,
        selected_range,
        &replacement,
        constraints,
        retained_graphemes,
    );
    let text_constraint_note = replacement.constraint_note;
    let text_constraint_receipt = replacement.constraint_receipt;
    let caret_offset = replacement.caret_offset;
    let committed_edit = replacement.committed_edit;
    finish_replace_selected_text_mutation(
        surface,
        text_documents,
        target,
        property,
        replacement.text,
        caret_offset,
        committed_edit,
        text_constraint_note,
        text_constraint_receipt,
        result,
    )
}
