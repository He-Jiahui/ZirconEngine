use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::UiNavigationEventKind,
};

pub(super) fn keyboard_navigation_kind(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiNavigationEventKind> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }
    if let Some(kind) = directional_navigation_kind(keyboard) {
        return Some(kind);
    }
    if keyboard.logical_key == "Tab" || keyboard.key_code == 9 {
        if keyboard.metadata.modifiers.control
            || keyboard.metadata.modifiers.alt
            || keyboard.metadata.modifiers.super_key
        {
            return None;
        }
        return Some(if keyboard.metadata.modifiers.shift {
            UiNavigationEventKind::Previous
        } else {
            UiNavigationEventKind::Next
        });
    }
    None
}

fn directional_navigation_kind(keyboard: &UiKeyboardInputEvent) -> Option<UiNavigationEventKind> {
    if let Some(kind) = logical_directional_navigation_kind(keyboard.logical_key.as_str()) {
        return Some(kind);
    }

    match keyboard.key_code {
        37 => Some(UiNavigationEventKind::Left),
        38 => Some(UiNavigationEventKind::Up),
        39 => Some(UiNavigationEventKind::Right),
        40 => Some(UiNavigationEventKind::Down),
        _ => None,
    }
}

fn logical_directional_navigation_kind(logical_key: &str) -> Option<UiNavigationEventKind> {
    let normalized = normalized_key_name(logical_key);
    match normalized.as_str() {
        "arrowleft" | "left" | "gamepaddpadleft" => Some(UiNavigationEventKind::Left),
        "arrowup" | "up" | "gamepaddpadup" => Some(UiNavigationEventKind::Up),
        "arrowright" | "right" | "gamepaddpadright" => Some(UiNavigationEventKind::Right),
        "arrowdown" | "down" | "gamepaddpaddown" => Some(UiNavigationEventKind::Down),
        _ => None,
    }
}

fn normalized_key_name(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
