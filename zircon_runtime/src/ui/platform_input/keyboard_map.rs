use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey, NativeKeyCode, PhysicalKey};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputState;

pub(super) fn keyboard_state(state: ElementState, repeat: bool) -> UiKeyboardInputState {
    match (state, repeat) {
        (ElementState::Pressed, true) => UiKeyboardInputState::Repeated,
        (ElementState::Pressed, false) => UiKeyboardInputState::Pressed,
        (ElementState::Released, _) => UiKeyboardInputState::Released,
    }
}

pub(super) fn legacy_key_code(key: &Key) -> u32 {
    match key {
        Key::Named(NamedKey::Backspace) => 8,
        Key::Named(NamedKey::Tab) => 9,
        Key::Named(NamedKey::Enter) => 13,
        Key::Named(NamedKey::Shift) => 16,
        Key::Named(NamedKey::Control) => 17,
        Key::Named(NamedKey::Alt) => 18,
        Key::Named(NamedKey::CapsLock) => 20,
        Key::Named(NamedKey::Escape) => 27,
        Key::Named(NamedKey::PageUp) => 33,
        Key::Named(NamedKey::PageDown) => 34,
        Key::Named(NamedKey::End) => 35,
        Key::Named(NamedKey::Home) => 36,
        Key::Named(NamedKey::ArrowLeft) => 37,
        Key::Named(NamedKey::ArrowUp) => 38,
        Key::Named(NamedKey::ArrowRight) => 39,
        Key::Named(NamedKey::ArrowDown) => 40,
        Key::Named(NamedKey::Delete) => 46,
        Key::Character(text) => legacy_character_key_code(text),
        _ => 0,
    }
}

pub(super) fn native_scan_code(physical_key: PhysicalKey) -> Option<u32> {
    match physical_key {
        PhysicalKey::Unidentified(NativeKeyCode::Android(code)) => Some(code),
        PhysicalKey::Unidentified(NativeKeyCode::MacOS(code)) => Some(u32::from(code)),
        PhysicalKey::Unidentified(NativeKeyCode::Windows(code)) => Some(u32::from(code)),
        PhysicalKey::Unidentified(NativeKeyCode::Xkb(code)) => Some(code),
        PhysicalKey::Code(_) | PhysicalKey::Unidentified(NativeKeyCode::Unidentified) => None,
    }
}

pub(super) fn physical_key_name(physical_key: PhysicalKey) -> String {
    match physical_key {
        PhysicalKey::Code(code) => code.to_string(),
        PhysicalKey::Unidentified(native) => format!("{native:?}"),
    }
}

pub(super) fn logical_key_name(key: &Key) -> String {
    match key {
        Key::Named(named) => named.to_string(),
        Key::Character(text) => text.to_string(),
        Key::Dead(Some(ch)) => format!("Dead({ch})"),
        Key::Dead(None) => "Dead".to_string(),
        Key::Unidentified(native) => format!("{native:?}"),
    }
}

fn legacy_character_key_code(text: &str) -> u32 {
    let mut chars = text.chars();
    let Some(ch) = chars.next() else {
        return 0;
    };
    if chars.next().is_some() {
        return 0;
    }
    match ch {
        ' ' => 32,
        '0'..='9' => ch as u32,
        'a'..='z' => ch.to_ascii_uppercase() as u32,
        'A'..='Z' => ch as u32,
        ';' | ':' => 186,
        '=' | '+' => 187,
        ',' | '<' => 188,
        '-' | '_' => 189,
        '.' | '>' => 190,
        '/' | '?' => 191,
        '`' | '~' => 192,
        '[' | '{' => 219,
        '\\' | '|' => 220,
        ']' | '}' => 221,
        '\'' | '"' => 222,
        _ => 0,
    }
}
