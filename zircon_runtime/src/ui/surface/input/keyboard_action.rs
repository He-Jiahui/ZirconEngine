use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

pub(super) fn keyboard_requests_default_activation(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    matches!(
        normalized.as_str(),
        "enter" | "space" | "spacebar" | "virtualaccept"
    ) || keyboard.logical_key == " "
        || matches!(keyboard.key_code, 13 | 32)
}

pub(super) fn keyboard_requests_popup_dismissal(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    let normalized = normalized_key_name(keyboard.logical_key.as_str());
    matches!(normalized.as_str(), "escape" | "esc" | "virtualback") || keyboard.key_code == 27
}

fn normalized_key_name(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
