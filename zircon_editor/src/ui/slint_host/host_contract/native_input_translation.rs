use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Ime, KeyEvent, MouseScrollDelta};
use winit::keyboard::{Key, ModifiersState, NamedKey, NativeKeyCode, PhysicalKey};
use zircon_runtime_interface::ui::dispatch::UiPointerEvent;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputModifiers,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerInputEvent, UiPreciseScrollDelta,
        UiTextByteRange, UiTextInputEvent,
    },
    layout::UiPoint,
    surface::UiPointerEventKind,
};

const PIXEL_SCROLL_LEGACY_LINE_SCALE: f32 = 0.1;

pub(crate) fn native_keyboard_event_to_shared_input(
    metadata: UiInputEventMetadata,
    event: &KeyEvent,
    modifiers: ModifiersState,
    synthetic: bool,
) -> UiInputEvent {
    let mut metadata = metadata;
    metadata.modifiers = native_modifiers_to_shared(modifiers);
    metadata.synthetic = synthetic;

    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: keyboard_state(event.state, event.repeat),
        key_code: legacy_key_code(&event.logical_key),
        scan_code: native_scan_code(event.physical_key),
        physical_key: physical_key_name(event.physical_key),
        logical_key: logical_key_name(&event.logical_key),
        text: event.text.as_ref().map(ToString::to_string),
    })
}

pub(crate) fn native_ime_event_to_shared_input(
    metadata: UiInputEventMetadata,
    event: &Ime,
) -> Option<UiInputEvent> {
    match event {
        Ime::Preedit(text, cursor_range) => Some(UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Preedit,
            text: text.clone(),
            cursor_range: cursor_range.map(|(start, end)| {
                UiTextByteRange::new(clamp_byte_index(start), clamp_byte_index(end))
            }),
        })),
        Ime::Commit(text) => Some(UiInputEvent::Text(UiTextInputEvent {
            metadata,
            text: text.clone(),
        })),
        Ime::Disabled => Some(UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Cancel,
            text: String::new(),
            cursor_range: None,
        })),
        Ime::Enabled | Ime::DeleteSurrounding { .. } => None,
    }
}

pub(crate) fn native_mouse_wheel_event_to_shared_input(
    metadata: UiInputEventMetadata,
    point: UiPoint,
    delta: MouseScrollDelta,
) -> UiInputEvent {
    let (scroll_delta, precise_scroll) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (y, UiPreciseScrollDelta::lines(x, y)),
        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
            let x = x as f32;
            let y = y as f32;
            (
                y * PIXEL_SCROLL_LEGACY_LINE_SCALE,
                UiPreciseScrollDelta::pixels(x, y),
            )
        }
    };

    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: Some(precise_scroll),
    })
}

fn native_modifiers_to_shared(modifiers: ModifiersState) -> UiInputModifiers {
    UiInputModifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        super_key: modifiers.meta_key(),
        caps_lock: false,
        num_lock: false,
    }
}

fn keyboard_state(state: ElementState, repeat: bool) -> UiKeyboardInputState {
    match (state, repeat) {
        (ElementState::Pressed, true) => UiKeyboardInputState::Repeated,
        (ElementState::Pressed, false) => UiKeyboardInputState::Pressed,
        (ElementState::Released, _) => UiKeyboardInputState::Released,
    }
}

fn physical_key_name(physical_key: PhysicalKey) -> String {
    match physical_key {
        PhysicalKey::Code(code) => code.to_string(),
        PhysicalKey::Unidentified(native) => format!("{native:?}"),
    }
}

fn logical_key_name(key: &Key) -> String {
    match key {
        Key::Named(named) => named.to_string(),
        Key::Character(text) => text.to_string(),
        Key::Dead(Some(ch)) => format!("Dead({ch})"),
        Key::Dead(None) => "Dead".to_string(),
        Key::Unidentified(native) => format!("{native:?}"),
    }
}

fn legacy_key_code(key: &Key) -> u32 {
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

fn native_scan_code(physical_key: PhysicalKey) -> Option<u32> {
    match physical_key {
        PhysicalKey::Unidentified(NativeKeyCode::Android(code)) => Some(code),
        PhysicalKey::Unidentified(NativeKeyCode::MacOS(code)) => Some(u32::from(code)),
        PhysicalKey::Unidentified(NativeKeyCode::Windows(code)) => Some(u32::from(code)),
        PhysicalKey::Unidentified(NativeKeyCode::Xkb(code)) => Some(code),
        PhysicalKey::Code(_) | PhysicalKey::Unidentified(NativeKeyCode::Unidentified) => None,
    }
}

fn clamp_byte_index(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}
