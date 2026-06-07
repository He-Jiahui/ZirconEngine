use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::surface::input) enum KeyboardClipboardAction {
    Copy,
    Cut,
    Paste,
}

pub(in crate::ui::surface::input) fn keyboard_clipboard_action(
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
