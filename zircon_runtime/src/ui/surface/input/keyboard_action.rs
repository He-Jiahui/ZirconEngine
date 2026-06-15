use zircon_runtime_interface::ui::{
    component::UiComponentKeyboardAction,
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
};

pub(super) fn keyboard_component_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiComponentKeyboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    if is_activation_key(keyboard, normalized.as_str()) {
        return Some(UiComponentKeyboardAction::Activate);
    }
    if is_cancel_key(keyboard, normalized.as_str()) {
        return Some(UiComponentKeyboardAction::Cancel);
    }

    match normalized.as_str() {
        "arrowright" | "right" | "arrowdown" | "down" | "gamepaddpadright" | "gamepaddpaddown" => {
            Some(UiComponentKeyboardAction::Next)
        }
        "arrowleft" | "left" | "arrowup" | "up" | "gamepaddpadleft" | "gamepaddpadup" => {
            Some(UiComponentKeyboardAction::Previous)
        }
        "home" => Some(UiComponentKeyboardAction::First),
        "end" => Some(UiComponentKeyboardAction::Last),
        "pageup" => Some(UiComponentKeyboardAction::LargeIncrement),
        "pagedown" => Some(UiComponentKeyboardAction::LargeDecrement),
        _ => match keyboard.key_code {
            39 | 40 => Some(UiComponentKeyboardAction::Next),
            37 | 38 => Some(UiComponentKeyboardAction::Previous),
            36 => Some(UiComponentKeyboardAction::First),
            35 => Some(UiComponentKeyboardAction::Last),
            33 => Some(UiComponentKeyboardAction::LargeIncrement),
            34 => Some(UiComponentKeyboardAction::LargeDecrement),
            _ => None,
        },
    }
}

pub(super) fn tree_view_keyboard_component_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiComponentKeyboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    if is_begin_edit_key(keyboard, normalized.as_str()) {
        return Some(UiComponentKeyboardAction::BeginEdit);
    }

    match normalized.as_str() {
        "arrowright" | "right" | "gamepaddpadright" => Some(UiComponentKeyboardAction::Increment),
        "arrowleft" | "left" | "gamepaddpadleft" => Some(UiComponentKeyboardAction::Decrement),
        "arrowdown" | "down" | "gamepaddpaddown" => Some(UiComponentKeyboardAction::Next),
        "arrowup" | "up" | "gamepaddpadup" => Some(UiComponentKeyboardAction::Previous),
        _ => match keyboard.key_code {
            39 => Some(UiComponentKeyboardAction::Increment),
            37 => Some(UiComponentKeyboardAction::Decrement),
            40 => Some(UiComponentKeyboardAction::Next),
            38 => Some(UiComponentKeyboardAction::Previous),
            _ => keyboard_component_action(keyboard),
        },
    }
}

pub(super) fn keyboard_component_text(keyboard: &UiKeyboardInputEvent) -> Option<&str> {
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

    let text = keyboard.text.as_deref()?;
    if text.is_empty()
        || text.chars().any(char::is_control)
        || text.chars().all(char::is_whitespace)
    {
        return None;
    }
    Some(text)
}

pub(super) fn keyboard_requests_default_activation(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    is_activation_key(keyboard, normalized.as_str())
}

pub(super) fn keyboard_requests_popup_dismissal(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    is_cancel_key(keyboard, normalized.as_str())
}

fn is_activation_key(keyboard: &UiKeyboardInputEvent, normalized: &str) -> bool {
    matches!(normalized, "enter" | "space" | "spacebar" | "virtualaccept")
        || keyboard.logical_key == " "
        || matches!(keyboard.key_code, 13 | 32)
}

fn is_cancel_key(keyboard: &UiKeyboardInputEvent, normalized: &str) -> bool {
    matches!(normalized, "escape" | "esc" | "virtualback") || keyboard.key_code == 27
}

fn is_begin_edit_key(keyboard: &UiKeyboardInputEvent, normalized: &str) -> bool {
    normalized == "f2" || keyboard.key_code == 113
}

fn normalized_key_name(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
