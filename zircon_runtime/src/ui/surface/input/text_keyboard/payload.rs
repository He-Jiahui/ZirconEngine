use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

pub(in crate::ui::surface::input) fn keyboard_requests_newline(
    keyboard: &UiKeyboardInputEvent,
) -> bool {
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

pub(in crate::ui::surface::input) fn keyboard_text_payload(
    keyboard: &UiKeyboardInputEvent,
) -> Option<&str> {
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
