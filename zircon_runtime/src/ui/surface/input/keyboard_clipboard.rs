use zircon_runtime_interface::ui::{
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiDispatchAppliedEffect, UiDispatchEffect,
        UiDispatchHostRequest, UiDispatchHostRequestKind, UiDispatchPhase, UiInputDispatchResult,
        UiInputEvent, UiKeyboardInputEvent,
    },
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextEditAction},
};

use crate::ui::text::apply_text_edit_action;

use super::super::surface::UiSurface;
use super::{
    editable_text::{apply_editable_text_state, TextComponentEventKind},
    is_valid_input_owner,
    owner_route::owner_routed_result,
    text_keyboard::KeyboardClipboardAction,
};

pub(super) fn dispatch_keyboard_clipboard(
    surface: &mut UiSurface,
    keyboard: UiKeyboardInputEvent,
    target: UiNodeId,
    editable: UiEditableTextState,
    action: KeyboardClipboardAction,
) -> UiInputDispatchResult {
    let phase = match action {
        KeyboardClipboardAction::Copy => "keyboard.clipboard_copy",
        KeyboardClipboardAction::Cut => "keyboard.clipboard_cut",
        KeyboardClipboardAction::Paste => "keyboard.clipboard_paste",
    };
    let event = UiInputEvent::Keyboard(keyboard);
    if !is_valid_input_owner(surface, target) {
        return owner_routed_result(surface, event, Some(target), phase);
    }

    match action {
        KeyboardClipboardAction::Copy => {
            let mut result = owner_routed_result(surface, event, Some(target), phase);
            if let Some(text) = selected_clipboard_text(&editable) {
                push_clipboard_request(
                    &mut result,
                    target,
                    UiClipboardRequestKind::WriteText,
                    Some(text),
                );
            } else {
                result
                    .diagnostics
                    .notes
                    .push("clipboard selection empty".to_string());
            }
            result
        }
        KeyboardClipboardAction::Cut => {
            let Some(text) = selected_clipboard_text(&editable) else {
                let mut result = owner_routed_result(surface, event, Some(target), phase);
                result
                    .diagnostics
                    .notes
                    .push("clipboard selection empty".to_string());
                return result;
            };
            if editable.read_only {
                let mut result = owner_routed_result(surface, event, Some(target), phase);
                result
                    .diagnostics
                    .notes
                    .push("clipboard cut blocked by read-only text".to_string());
                return result;
            }

            let next = apply_text_edit_action(editable, UiTextEditAction::Delete);
            let mut result = apply_editable_text_state(
                surface,
                event,
                target,
                next,
                phase,
                TextComponentEventKind::Change,
            );
            push_clipboard_request(
                &mut result,
                target,
                UiClipboardRequestKind::WriteText,
                Some(text),
            );
            result
        }
        KeyboardClipboardAction::Paste => {
            let mut result = owner_routed_result(surface, event, Some(target), phase);
            if editable.read_only {
                result
                    .diagnostics
                    .notes
                    .push("clipboard paste blocked by read-only text".to_string());
            } else {
                push_clipboard_request(&mut result, target, UiClipboardRequestKind::ReadText, None);
            }
            result
        }
    }
}

fn selected_clipboard_text(state: &UiEditableTextState) -> Option<String> {
    let range = state.selection.as_ref()?.range();
    if range.start == range.end || range.end > state.text.len() {
        return None;
    }
    state
        .text
        .get(range.start..range.end)
        .map(ToString::to_string)
}

fn push_clipboard_request(
    result: &mut UiInputDispatchResult,
    target: UiNodeId,
    kind: UiClipboardRequestKind,
    text: Option<String>,
) {
    let request = UiClipboardRequest {
        kind,
        owner: target,
        text,
    };
    let effect = UiDispatchEffect::RequestClipboard {
        request: request.clone(),
    };
    let effect_index = result.reply.effects.len();
    result.reply.effects.push(effect.clone());
    result.reply.handler = Some(target);
    result.reply.phase = Some(UiDispatchPhase::DefaultAction);
    result.applied_effects.push(UiDispatchAppliedEffect {
        effect_index,
        effect,
    });
    result.host_requests.push(UiDispatchHostRequest {
        effect_index,
        request: UiDispatchHostRequestKind::Clipboard(request),
        reason: "text clipboard shortcut".to_string(),
    });
}
