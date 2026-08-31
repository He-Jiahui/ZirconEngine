mod document_transaction;
mod ime_context;
mod mutation;
pub(super) mod profile;
mod property_transaction;
mod state_transition;

pub(in crate::ui) use document_transaction::{
    PreparedUiEditableTextDocumentTransaction, UiEditableTextDocumentTransactionReceipt,
};
pub(super) use mutation::{
    TextComponentEventKind, apply_editable_text_state, cancel_number_field_edit_state,
    step_number_field_keyboard_state, submit_editable_text_state,
};
pub(in crate::ui) use mutation::{
    UiEditableTextTransactionError, commit_editable_text_transaction,
};
pub(in crate::ui::surface) use mutation::{
    cancel_editable_text_composition_for_input_method_loss, finish_editable_text_for_focus_loss,
};
pub(in crate::ui) use property_transaction::{
    PreparedUiEditableTextPropertyTransaction, UiEditableTextPropertyTransactionError,
    UiEditableTextPropertyTransactionReceipt, commit_editable_text_properties,
    commit_editable_text_properties_with_value, prepare_editable_text_properties_with_edit,
    prepare_editable_text_properties_with_value, prepare_number_field_model_update_properties,
    prepare_number_field_properties,
};

use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiKeyboardInputEvent, UiKeyboardInputState, UiTextInputEvent,
    },
    event_ui::UiNodeId,
    surface::{UiTextEditAction, UiTextRange},
};

use crate::ui::{
    dispatch::UiTextDocumentSession,
    text::{apply_text_edit_action_with_intent, apply_text_edit_actions_with_intent},
};

use super::super::surface::UiSurface;
use super::{
    is_valid_input_owner,
    keyboard_clipboard::dispatch_keyboard_clipboard,
    owner_route::owner_routed_result,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
    text_constraints::{
        TextInputConstraints, TextInputRetainedGraphemeCount, text_input_constraints_for_node,
    },
    text_keyboard::{
        keyboard_clipboard_action, keyboard_requests_newline, keyboard_text_edit_actions,
        keyboard_text_history_direction, keyboard_text_payload,
    },
    text_state::{editable_text_input_is_secure, editable_text_state_for_node},
};

use state_transition::{
    TextInputStateTransition, committed_text_state, delete_surrounding_text_state,
    preedit_text_state, retained_document_replaced_range,
};

pub(super) fn dispatch_keyboard_text_edit(
    surface: &mut UiSurface,
    keyboard: UiKeyboardInputEvent,
    target: UiNodeId,
    mut text_documents: Option<&mut UiTextDocumentSession>,
) -> Option<UiInputDispatchResult> {
    let editable = editable_text_state_for_node(surface, target)?;
    let secure = editable_text_input_is_secure(surface, target);
    synchronize_text_document(text_documents.as_deref_mut(), surface, target, &editable);
    if editable.composition.is_none()
        && !editable.read_only
        && matches!(
            keyboard.state,
            UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
        )
        && !keyboard.metadata.modifiers.alt
        && !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.shift
        && !keyboard.metadata.modifiers.super_key
    {
        let direction = match (keyboard.logical_key.as_str(), keyboard.key_code) {
            ("ArrowUp" | "Up", _) | (_, 38) => Some(1.0),
            ("ArrowDown" | "Down", _) | (_, 40) => Some(-1.0),
            _ => None,
        };
        if let Some(result) = direction.and_then(|direction| {
            step_number_field_keyboard_state(
                surface,
                UiInputEvent::Keyboard(keyboard.clone()),
                target,
                &editable,
                direction,
            )
        }) {
            return Some(result);
        }
    }
    let escape = keyboard.logical_key == "Escape" || keyboard.key_code == 27;
    if escape
        && matches!(
            keyboard.state,
            UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
        )
        && editable.composition.is_none()
        && super::number_field::number_field_edit_is_active(surface, target)
    {
        return Some(cancel_number_field_edit_state(
            surface,
            UiInputEvent::Keyboard(keyboard),
            target,
            editable,
        ));
    }
    if let Some(direction) = keyboard_text_history_direction(&keyboard) {
        return Some(dispatch_keyboard_text_history(
            surface,
            keyboard,
            target,
            editable,
            text_documents.as_deref_mut(),
            direction,
        ));
    }
    if let Some(action) = keyboard_clipboard_action(&keyboard) {
        return Some(dispatch_keyboard_clipboard(
            surface, keyboard, target, editable, action,
        ));
    }
    if keyboard_requests_newline(&keyboard) {
        let constraints = text_input_constraints_for_node(surface, target);
        if !constraints.allows_multiline() {
            if editable.read_only {
                return None;
            }
            let repeated = keyboard.state == UiKeyboardInputState::Repeated;
            return Some(submit_editable_text_state(
                surface,
                UiInputEvent::Keyboard(keyboard),
                target,
                editable,
                repeated,
            ));
        }
        let retained_graphemes = retained_grapheme_count_for_constraints(
            text_documents.as_deref_mut(),
            surface,
            target,
            retained_document_replaced_range(&editable),
            constraints,
        );
        let transition =
            committed_text_state(editable, "\n".to_string(), constraints, retained_graphemes);
        return Some(apply_text_input_state_transition(
            surface,
            text_documents.as_deref_mut(),
            UiInputEvent::Keyboard(keyboard),
            target,
            transition,
            "keyboard.text_edit",
            TextComponentEventKind::Change,
        ));
    }
    if let Some(text) = keyboard_text_payload(&keyboard) {
        let constraints = text_input_constraints_for_node(surface, target);
        let retained_graphemes = retained_grapheme_count_for_constraints(
            text_documents.as_deref_mut(),
            surface,
            target,
            retained_document_replaced_range(&editable),
            constraints,
        );
        let transition =
            committed_text_state(editable, text.to_string(), constraints, retained_graphemes);
        return Some(apply_text_input_state_transition(
            surface,
            text_documents.as_deref_mut(),
            UiInputEvent::Keyboard(keyboard),
            target,
            transition,
            "keyboard.text_payload",
            TextComponentEventKind::Change,
        ));
    }
    let actions = keyboard_text_edit_actions(&keyboard, &editable, secure)?;
    let transition = match apply_text_edit_actions_with_intent(editable, actions) {
        Ok(transition) => TextInputStateTransition::from_edit(transition),
        Err(_) => {
            let mut result = owner_routed_result(
                surface,
                UiInputEvent::Keyboard(keyboard),
                Some(target),
                "keyboard.text_edit",
            );
            result
                .diagnostics
                .notes
                .push("text_edit_sequence_rejected:multiple_committed_edits".to_string());
            return Some(result);
        }
    };
    Some(apply_text_input_state_transition(
        surface,
        text_documents.as_deref_mut(),
        UiInputEvent::Keyboard(keyboard),
        target,
        transition,
        "keyboard.text_edit",
        TextComponentEventKind::Change,
    ))
}

fn dispatch_keyboard_text_history(
    surface: &mut UiSurface,
    keyboard: UiKeyboardInputEvent,
    target: UiNodeId,
    editable: zircon_runtime_interface::ui::surface::UiEditableTextState,
    text_documents: Option<&mut UiTextDocumentSession>,
    direction: crate::ui::dispatch::UiTextHistoryDirection,
) -> UiInputDispatchResult {
    let event = UiInputEvent::Keyboard(keyboard);
    if editable.read_only || editable.composition.is_some() {
        return text_history_unavailable_result(surface, event, target, "text_history_blocked");
    }
    let Some(text_documents) = text_documents else {
        return text_history_unavailable_result(
            surface,
            event,
            target,
            "text_history_session_unavailable",
        );
    };
    let Some(source_epoch) = surface.input.text_document_epoch(target) else {
        return text_history_unavailable_result(
            surface,
            event,
            target,
            "text_history_source_epoch_unavailable",
        );
    };
    let transition = match text_documents.prepare_history_transition(
        &surface.tree.tree_id,
        target,
        source_epoch,
        editable,
        direction,
    ) {
        Ok(Some(transition)) => transition,
        Ok(None) => {
            return text_history_unavailable_result(
                surface,
                event,
                target,
                "text_history_unavailable",
            );
        }
        Err(error) => {
            let note = format!("text_history_rejected:{}", error.diagnostic_code());
            return text_history_unavailable_result(surface, event, target, &note);
        }
    };
    apply_text_input_state_transition(
        surface,
        Some(text_documents),
        event,
        target,
        TextInputStateTransition::from_edit(transition),
        "keyboard.text_history",
        TextComponentEventKind::Change,
    )
}

fn text_history_unavailable_result(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    note: &str,
) -> UiInputDispatchResult {
    let mut result = owner_routed_result(surface, event, Some(target), "keyboard.text_history");
    result.diagnostics.notes.push(note.to_string());
    surface.redact_secure_text_result(target, &mut result);
    result
}

pub(super) fn dispatch_text_input(
    surface: &mut UiSurface,
    text: UiTextInputEvent,
    mut text_documents: Option<&mut UiTextDocumentSession>,
) -> UiInputDispatchResult {
    let target = text_input_target(surface);
    let event = UiInputEvent::Text(text.clone());
    let Some(target) = target else {
        let result = owner_routed_result(surface, event, None, "text.owner");
        return with_editable_text_route_policy(surface, result);
    };

    let Some(editable) = editable_text_state_for_node(surface, target) else {
        let mut result = owner_routed_result(surface, event, Some(target), "text.owner");
        result
            .diagnostics
            .notes
            .push("text target is not editable".to_string());
        return with_editable_text_route_policy(surface, result);
    };
    synchronize_text_document(text_documents.as_deref_mut(), surface, target, &editable);

    let constraints = text_input_constraints_for_node(surface, target);
    let retained_graphemes = retained_grapheme_count_for_constraints(
        text_documents.as_deref_mut(),
        surface,
        target,
        retained_document_replaced_range(&editable),
        constraints,
    );
    let transition = committed_text_state(editable, text.text, constraints, retained_graphemes);
    let result = apply_text_input_state_transition(
        surface,
        text_documents.as_deref_mut(),
        event,
        target,
        transition,
        "text.edit",
        TextComponentEventKind::Change,
    );
    with_editable_text_route_policy(surface, result)
}

pub(super) fn apply_committed_text_payload(
    surface: &mut UiSurface,
    mut text_documents: Option<&mut UiTextDocumentSession>,
    event: UiInputEvent,
    target: UiNodeId,
    editable: zircon_runtime_interface::ui::surface::UiEditableTextState,
    text: String,
    phase: &str,
) -> UiInputDispatchResult {
    synchronize_text_document(text_documents.as_deref_mut(), surface, target, &editable);
    let constraints = text_input_constraints_for_node(surface, target);
    let retained_graphemes = retained_grapheme_count_for_constraints(
        text_documents.as_deref_mut(),
        surface,
        target,
        retained_document_replaced_range(&editable),
        constraints,
    );
    let transition = committed_text_state(editable, text, constraints, retained_graphemes);
    apply_text_input_state_transition(
        surface,
        text_documents.as_deref_mut(),
        event,
        target,
        transition,
        phase,
        TextComponentEventKind::Change,
    )
}

pub(super) fn dispatch_ime_input(
    surface: &mut UiSurface,
    ime: UiImeInputEvent,
    mut text_documents: Option<&mut UiTextDocumentSession>,
) -> UiInputDispatchResult {
    let target = surface.input.input_method_owner;
    let clear_owner = matches!(ime.kind, UiImeInputEventKind::Cancel);
    let event = UiInputEvent::Ime(ime.clone());
    let Some(target) = target.filter(|owner| is_valid_input_owner(surface, *owner)) else {
        surface.disable_input_method_for_focus_loss();
        let mut result = owner_routed_result(surface, event, None, "ime.owner");
        result
            .diagnostics
            .notes
            .push("owner route rejected".to_string());
        result
            .diagnostics
            .notes
            .push("ime owner missing".to_string());
        return with_editable_text_route_policy(surface, result);
    };

    let Some(editable) = editable_text_state_for_node(surface, target) else {
        let mut result = owner_routed_result(surface, event, Some(target), "ime.owner");
        result
            .diagnostics
            .notes
            .push("ime target is not editable".to_string());
        if clear_owner {
            surface.input.clear_input_method();
            result
                .diagnostics
                .notes
                .push("ime owner cleared".to_string());
        }
        return with_editable_text_route_policy(surface, result);
    };
    synchronize_text_document(text_documents.as_deref_mut(), surface, target, &editable);

    if let Err(error) = ime.validate() {
        let mut result = owner_routed_result(surface, event, Some(target), "ime.payload");
        result
            .diagnostics
            .notes
            .push(format!("invalid IME payload: {error}"));
        return with_editable_text_route_policy(surface, result);
    }

    let component_event_kind = match ime.kind {
        UiImeInputEventKind::Commit => TextComponentEventKind::Submit,
        _ => TextComponentEventKind::Change,
    };
    let constraints = text_input_constraints_for_node(surface, target);
    let retained_graphemes = match ime.kind {
        UiImeInputEventKind::Preedit => retained_grapheme_count_for_constraints(
            text_documents.as_deref_mut(),
            surface,
            target,
            retained_document_replaced_range(&editable),
            constraints,
        ),
        UiImeInputEventKind::Commit => retained_grapheme_count_for_constraints(
            text_documents.as_deref_mut(),
            surface,
            target,
            retained_document_replaced_range(&editable),
            constraints,
        ),
        UiImeInputEventKind::Cancel | UiImeInputEventKind::DeleteSurrounding => {
            TextInputRetainedGraphemeCount::SourceScan
        }
    };
    let transition = match ime.kind {
        UiImeInputEventKind::Preedit => preedit_text_state(
            editable,
            &ime.text,
            ime.cursor_range,
            &ime.preedit_clauses,
            constraints,
            retained_graphemes,
        ),
        UiImeInputEventKind::Commit => {
            committed_text_state(editable, ime.text, constraints, retained_graphemes)
        }
        UiImeInputEventKind::Cancel => TextInputStateTransition::from_edit(
            apply_text_edit_action_with_intent(editable, UiTextEditAction::CancelComposition),
        ),
        UiImeInputEventKind::DeleteSurrounding => {
            let Some(delete) = ime.delete_surrounding else {
                let result = owner_routed_result(surface, event, Some(target), "ime.edit");
                return with_editable_text_route_policy(surface, result);
            };
            let Some(transition) = delete_surrounding_text_state(editable, delete) else {
                let result = owner_routed_result(surface, event, Some(target), "ime.edit");
                return with_editable_text_route_policy(surface, result);
            };
            transition
        }
    };

    let mut result = apply_text_input_state_transition(
        surface,
        text_documents.as_deref_mut(),
        event,
        target,
        transition,
        "ime.edit",
        component_event_kind,
    );
    if clear_owner {
        surface.input.clear_input_method();
        result
            .diagnostics
            .notes
            .push("ime owner cleared".to_string());
    }
    with_editable_text_route_policy(surface, result)
}

fn apply_text_input_state_transition(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    event: UiInputEvent,
    target: UiNodeId,
    transition: TextInputStateTransition,
    phase: &str,
    component_event_kind: TextComponentEventKind,
) -> UiInputDispatchResult {
    let TextInputStateTransition {
        state,
        constraint_receipt,
        committed_edit,
    } = transition;
    let mut result = apply_editable_text_state(
        surface,
        text_documents,
        event,
        target,
        state,
        committed_edit,
        phase,
        component_event_kind,
    );
    if !constraint_receipt.is_empty() {
        result.diagnostics.text_constraint = Some(constraint_receipt);
    }
    result
}

pub(in crate::ui) fn synchronize_text_document(
    text_documents: Option<&mut UiTextDocumentSession>,
    surface: &UiSurface,
    target: UiNodeId,
    state: &zircon_runtime_interface::ui::surface::UiEditableTextState,
) {
    let Some(text_documents) = text_documents else {
        return;
    };
    let Some(source_epoch) = surface.input.text_document_epoch(target) else {
        return;
    };
    text_documents.synchronize_source(&surface.tree.tree_id, target, source_epoch, &state.text);
    if editable_text_input_is_secure(surface, target) {
        text_documents.discard_history(&surface.tree.tree_id, target);
    }
}

pub(in crate::ui) fn retained_grapheme_count_for_constraints(
    text_documents: Option<&mut UiTextDocumentSession>,
    surface: &UiSurface,
    target: UiNodeId,
    replaced_range: UiTextRange,
    constraints: TextInputConstraints,
) -> TextInputRetainedGraphemeCount {
    if !constraints.requires_retained_grapheme_count() {
        return TextInputRetainedGraphemeCount::SourceScan;
    }
    let Some(source_epoch) = surface.input.text_document_epoch(target) else {
        return TextInputRetainedGraphemeCount::SourceScan;
    };
    let Some(text_documents) = text_documents else {
        return TextInputRetainedGraphemeCount::SourceScan;
    };
    text_documents
        .retained_grapheme_count(
            &surface.tree.tree_id,
            target,
            source_epoch,
            replaced_range.start..replaced_range.end,
        )
        .map(TextInputRetainedGraphemeCount::DocumentIndex)
        .unwrap_or(TextInputRetainedGraphemeCount::SourceScan)
}

fn with_editable_text_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

fn text_input_target(surface: &mut UiSurface) -> Option<UiNodeId> {
    let ime_owner = surface.input.input_method_owner;
    if ime_owner.is_some_and(|owner| is_valid_input_owner(surface, owner)) {
        return ime_owner;
    }
    if ime_owner.is_some() {
        surface.disable_input_method_for_focus_loss();
    }
    surface
        .focus
        .focused
        .filter(|owner| is_valid_input_owner(surface, *owner))
}
