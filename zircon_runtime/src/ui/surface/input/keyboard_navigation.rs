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
    match (keyboard.logical_key.as_str(), keyboard.key_code) {
        ("ArrowLeft", _) | (_, 37) => Some(UiNavigationEventKind::Left),
        ("ArrowUp", _) | (_, 38) => Some(UiNavigationEventKind::Up),
        ("ArrowRight", _) | (_, 39) => Some(UiNavigationEventKind::Right),
        ("ArrowDown", _) | (_, 40) => Some(UiNavigationEventKind::Down),
        _ => None,
    }
}
