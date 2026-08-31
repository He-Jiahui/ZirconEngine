use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::UiNavigationEventKind,
};

const MAX_NORMALIZED_DIRECTION_KEY_BYTES: usize = "gamepaddpadright".len();

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
    let (normalized, len) = normalized_direction_key(logical_key)?;
    match &normalized[..len] {
        b"arrowleft" | b"left" | b"gamepaddpadleft" => Some(UiNavigationEventKind::Left),
        b"arrowup" | b"up" | b"gamepaddpadup" => Some(UiNavigationEventKind::Up),
        b"arrowright" | b"right" | b"gamepaddpadright" => Some(UiNavigationEventKind::Right),
        b"arrowdown" | b"down" | b"gamepaddpaddown" => Some(UiNavigationEventKind::Down),
        _ => None,
    }
}

fn normalized_direction_key(
    key: &str,
) -> Option<([u8; MAX_NORMALIZED_DIRECTION_KEY_BYTES], usize)> {
    let mut normalized = [0; MAX_NORMALIZED_DIRECTION_KEY_BYTES];
    let mut len = 0;
    for byte in key.bytes().filter(u8::is_ascii_alphanumeric) {
        if len == normalized.len() {
            return None;
        }
        normalized[len] = byte.to_ascii_lowercase();
        len += 1;
    }
    Some((normalized, len))
}

#[cfg(test)]
#[path = "keyboard_navigation/single_normalize_tests.rs"]
mod single_normalize_tests;
