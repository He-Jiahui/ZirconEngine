use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::{UiEditableTextState, UiTextEditAction},
};

use crate::ui::text::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KeyboardClipboardAction {
    Copy,
    Cut,
    Paste,
}

pub(super) fn keyboard_clipboard_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<KeyboardClipboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }
    if keyboard.metadata.modifiers.alt {
        return None;
    }

    let logical_key = keyboard.logical_key.as_str();
    if !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.shift
        && !keyboard.metadata.modifiers.super_key
    {
        match logical_key {
            "Copy" | "copy" => return Some(KeyboardClipboardAction::Copy),
            "Cut" | "cut" => return Some(KeyboardClipboardAction::Cut),
            "Paste" | "paste" => return Some(KeyboardClipboardAction::Paste),
            _ => {}
        }
    }
    if keyboard.metadata.modifiers.shift
        && !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.super_key
        && (logical_key == "Delete" || keyboard.key_code == 46)
    {
        return Some(KeyboardClipboardAction::Cut);
    }
    if !(keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key) {
        return None;
    }

    if matches!(logical_key, "c" | "C") || keyboard.key_code == 67 {
        return Some(KeyboardClipboardAction::Copy);
    }
    if matches!(logical_key, "x" | "X") || keyboard.key_code == 88 {
        return Some(KeyboardClipboardAction::Cut);
    }
    if matches!(logical_key, "v" | "V") || keyboard.key_code == 86 {
        return Some(KeyboardClipboardAction::Paste);
    }
    None
}

pub(super) fn keyboard_requests_newline(keyboard: &UiKeyboardInputEvent) -> bool {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return false;
    }
    if keyboard.metadata.modifiers.alt
        || keyboard.metadata.modifiers.control
        || keyboard.metadata.modifiers.shift
        || keyboard.metadata.modifiers.super_key
    {
        return false;
    }

    keyboard.logical_key == "Enter" || keyboard.key_code == 13
}

pub(super) fn keyboard_text_payload(keyboard: &UiKeyboardInputEvent) -> Option<&str> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }
    if keyboard.metadata.modifiers.alt
        || keyboard.metadata.modifiers.control
        || keyboard.metadata.modifiers.super_key
    {
        return None;
    }
    if keyboard.logical_key == "Tab" || keyboard.key_code == 9 {
        return None;
    }

    let text = keyboard.text.as_deref()?;
    if text.is_empty() || text.chars().any(char::is_control) {
        return None;
    }
    Some(text)
}

pub(super) fn keyboard_text_edit_actions(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
) -> Option<Vec<UiTextEditAction>> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }

    let extend_selection = keyboard.metadata.modifiers.shift;
    let word_navigation = keyboard.metadata.modifiers.control;
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = keyboard.metadata.modifiers.super_key
        && !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.alt;
    match keyboard.logical_key.as_str() {
        key if keyboard_requests_select_all(keyboard, key) => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        "Backspace" if word_navigation => Some(delete_previous_word_actions(state)),
        "Delete" if word_navigation => Some(delete_next_word_actions(state)),
        "Backspace" => Some(single_action(UiTextEditAction::Backspace)),
        "Delete" => Some(single_action(UiTextEditAction::Delete)),
        "Escape" => Some(escape_actions(state)),
        "ArrowLeft" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowRight" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowLeft" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowRight" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowUp" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        "ArrowDown" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        "Home" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        "End" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => keyboard_text_edit_actions_from_key_code(
            keyboard,
            state,
            extend_selection,
            word_navigation,
        ),
    }
}

fn keyboard_text_edit_actions_from_key_code(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
    extend_selection: bool,
    word_navigation: bool,
) -> Option<Vec<UiTextEditAction>> {
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = keyboard.metadata.modifiers.super_key
        && !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.alt;
    match keyboard.key_code {
        65 | 97 if keyboard_requests_select_all(keyboard, "") => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        8 if word_navigation => Some(delete_previous_word_actions(state)),
        46 if word_navigation => Some(delete_next_word_actions(state)),
        8 => Some(single_action(UiTextEditAction::Backspace)),
        46 => Some(single_action(UiTextEditAction::Delete)),
        27 => Some(escape_actions(state)),
        37 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        39 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        37 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        39 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        38 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        40 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        36 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        35 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => None,
    }
}

fn delete_previous_word_actions(state: &UiEditableTextState) -> Vec<UiTextEditAction> {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Backspace);
    }
    let caret = state.caret.offset.min(state.text.len());
    let start = previous_text_boundary(&state.text, caret, true);
    if start == caret {
        single_action(UiTextEditAction::Backspace)
    } else {
        vec![
            UiTextEditAction::SetSelection {
                anchor: start,
                focus: caret,
            },
            UiTextEditAction::Backspace,
        ]
    }
}

fn delete_next_word_actions(state: &UiEditableTextState) -> Vec<UiTextEditAction> {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Delete);
    }
    let caret = state.caret.offset.min(state.text.len());
    let end = next_text_boundary(&state.text, caret, true);
    if end == caret {
        single_action(UiTextEditAction::Delete)
    } else {
        vec![
            UiTextEditAction::SetSelection {
                anchor: caret,
                focus: end,
            },
            UiTextEditAction::Delete,
        ]
    }
}

fn escape_actions(state: &UiEditableTextState) -> Vec<UiTextEditAction> {
    if state.composition.is_some() {
        single_action(UiTextEditAction::CancelComposition)
    } else {
        single_action(UiTextEditAction::MoveCaret {
            offset: state.caret.offset,
            extend_selection: false,
        })
    }
}

fn single_action(action: UiTextEditAction) -> Vec<UiTextEditAction> {
    vec![action]
}

fn has_active_selection(state: &UiEditableTextState) -> bool {
    state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
}

fn home_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        line_start_boundary(&state.text, state.caret.offset)
    }
}

fn end_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        line_end_boundary(&state.text, state.caret.offset)
    }
}

fn previous_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        previous_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(0)
    }
}

fn next_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        next_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(state.text.len())
    }
}

fn keyboard_requests_select_all(keyboard: &UiKeyboardInputEvent, logical_key: &str) -> bool {
    (keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key)
        && !keyboard.metadata.modifiers.alt
        && matches!(logical_key, "a" | "A" | "")
}

fn previous_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        previous_word_boundary(text, offset).unwrap_or(0)
    } else {
        previous_grapheme_boundary(text, offset).unwrap_or(0)
    }
}

fn next_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        next_word_boundary(text, offset).unwrap_or(text.len())
    } else {
        next_grapheme_boundary(text, offset).unwrap_or(text.len())
    }
}
