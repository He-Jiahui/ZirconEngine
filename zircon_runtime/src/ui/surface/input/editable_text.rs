mod ime_context;
mod mutation;
mod state_transition;

pub(in crate::ui::surface) use mutation::commit_editable_text_composition_for_focus_loss;
pub(super) use mutation::{apply_editable_text_state, TextComponentEventKind};

use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiKeyboardInputEvent, UiTextInputEvent,
    },
    event_ui::UiNodeId,
    surface::UiTextEditAction,
};

use crate::ui::text::apply_text_edit_action;

use super::super::surface::UiSurface;
use super::{
    is_valid_input_owner,
    keyboard_clipboard::dispatch_keyboard_clipboard,
    owner_route::owner_routed_result,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
    text_constraints::text_input_constraints_for_node,
    text_keyboard::{
        keyboard_clipboard_action, keyboard_requests_newline, keyboard_text_edit_actions,
        keyboard_text_payload,
    },
    text_state::editable_text_state_for_node,
};

use state_transition::{committed_text_state, delete_surrounding_text_state, preedit_text_state};

pub(super) fn dispatch_keyboard_text_edit(
    surface: &mut UiSurface,
    keyboard: UiKeyboardInputEvent,
    target: UiNodeId,
) -> Option<UiInputDispatchResult> {
    let editable = editable_text_state_for_node(surface, target)?;
    if let Some(action) = keyboard_clipboard_action(&keyboard) {
        return Some(dispatch_keyboard_clipboard(
            surface, keyboard, target, editable, action,
        ));
    }
    if keyboard_requests_newline(&keyboard) {
        let constraints = text_input_constraints_for_node(surface, target);
        if !constraints.allows_multiline() {
            return None;
        }
        let next = committed_text_state(editable, "\n".to_string(), constraints);
        return Some(apply_editable_text_state(
            surface,
            UiInputEvent::Keyboard(keyboard),
            target,
            next,
            "keyboard.text_edit",
            TextComponentEventKind::Change,
        ));
    }
    if let Some(text) = keyboard_text_payload(&keyboard) {
        let constraints = text_input_constraints_for_node(surface, target);
        let next = committed_text_state(editable, text.to_string(), constraints);
        return Some(apply_editable_text_state(
            surface,
            UiInputEvent::Keyboard(keyboard),
            target,
            next,
            "keyboard.text_payload",
            TextComponentEventKind::Change,
        ));
    }
    let actions = keyboard_text_edit_actions(&keyboard, &editable)?;
    let next = actions.into_iter().fold(editable, apply_text_edit_action);
    Some(apply_editable_text_state(
        surface,
        UiInputEvent::Keyboard(keyboard),
        target,
        next,
        "keyboard.text_edit",
        TextComponentEventKind::Change,
    ))
}

pub(super) fn dispatch_text_input(
    surface: &mut UiSurface,
    text: UiTextInputEvent,
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

    let constraints = text_input_constraints_for_node(surface, target);
    let next = committed_text_state(editable, text.text, constraints);
    let result = apply_editable_text_state(
        surface,
        event,
        target,
        next,
        "text.edit",
        TextComponentEventKind::Change,
    );
    with_editable_text_route_policy(surface, result)
}

pub(super) fn dispatch_ime_input(
    surface: &mut UiSurface,
    ime: UiImeInputEvent,
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
    let next = match ime.kind {
        UiImeInputEventKind::Preedit => preedit_text_state(
            editable,
            &ime.text,
            ime.cursor_range,
            &ime.preedit_clauses,
            text_input_constraints_for_node(surface, target),
        ),
        UiImeInputEventKind::Commit => committed_text_state(
            editable,
            ime.text,
            text_input_constraints_for_node(surface, target),
        ),
        UiImeInputEventKind::Cancel => {
            apply_text_edit_action(editable, UiTextEditAction::CancelComposition)
        }
        UiImeInputEventKind::DeleteSurrounding => {
            let Some(delete) = ime.delete_surrounding else {
                let result = owner_routed_result(surface, event, Some(target), "ime.edit");
                return with_editable_text_route_policy(surface, result);
            };
            let Some(next) = delete_surrounding_text_state(editable, delete) else {
                let result = owner_routed_result(surface, event, Some(target), "ime.edit");
                return with_editable_text_route_policy(surface, result);
            };
            next
        }
    };

    let mut result = apply_editable_text_state(
        surface,
        event,
        target,
        next,
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
