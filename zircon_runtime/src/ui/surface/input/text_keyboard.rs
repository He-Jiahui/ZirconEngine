use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::{UiEditableTextState, UiTextEditAction},
};

pub(super) fn keyboard_text_edit_action(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
) -> Option<UiTextEditAction> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }

    match keyboard.logical_key.as_str() {
        "Backspace" => Some(UiTextEditAction::Backspace),
        "Delete" => Some(UiTextEditAction::Delete),
        "Escape" => Some(UiTextEditAction::CancelComposition),
        "ArrowLeft" => Some(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset),
            extend_selection: false,
        }),
        "ArrowRight" => Some(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset),
            extend_selection: false,
        }),
        "Home" => Some(UiTextEditAction::MoveCaret {
            offset: 0,
            extend_selection: false,
        }),
        "End" => Some(UiTextEditAction::MoveCaret {
            offset: state.text.len(),
            extend_selection: false,
        }),
        _ => keyboard_text_edit_action_from_key_code(keyboard, state),
    }
}

fn keyboard_text_edit_action_from_key_code(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
) -> Option<UiTextEditAction> {
    match keyboard.key_code {
        8 => Some(UiTextEditAction::Backspace),
        46 => Some(UiTextEditAction::Delete),
        27 => Some(UiTextEditAction::CancelComposition),
        37 => Some(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset),
            extend_selection: false,
        }),
        39 => Some(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset),
            extend_selection: false,
        }),
        36 => Some(UiTextEditAction::MoveCaret {
            offset: 0,
            extend_selection: false,
        }),
        35 => Some(UiTextEditAction::MoveCaret {
            offset: state.text.len(),
            extend_selection: false,
        }),
        _ => None,
    }
}

fn previous_text_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_text_boundary(text, offset);
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
        .unwrap_or(0)
}

fn next_text_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_text_boundary(text, offset);
    text.char_indices()
        .map(|(index, _)| index)
        .find(|index| *index > offset)
        .unwrap_or(text.len())
}

fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
