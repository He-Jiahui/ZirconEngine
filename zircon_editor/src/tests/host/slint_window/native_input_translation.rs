use crate::ui::slint_host::{
    native_ime_event_to_shared_input, native_keyboard_event_to_shared_input,
    native_mouse_wheel_event_to_shared_input,
};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Ime, KeyEvent, MouseScrollDelta};
use winit::keyboard::{
    Key, KeyCode, KeyLocation, ModifiersState, NamedKey, NativeKeyCode, PhysicalKey,
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputState, UiPreciseScrollDelta, UiWindowId,
    },
    layout::UiPoint,
    surface::UiPointerEventKind,
};

#[test]
fn native_keyboard_translation_preserves_repeat_text_modifiers_and_keys() {
    let event = KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::KeyA),
        logical_key: Key::Character("A".into()),
        text: Some("A".into()),
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: true,
        text_with_all_modifiers: Some("A".into()),
        key_without_modifiers: Key::Character("a".into()),
    };
    let modifiers = ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::META;

    let shared = native_keyboard_event_to_shared_input(input_metadata(), &event, modifiers, true);

    let UiInputEvent::Keyboard(keyboard) = shared else {
        panic!("native keyboard translation should produce keyboard input");
    };
    assert_eq!(keyboard.state, UiKeyboardInputState::Repeated);
    assert_eq!(keyboard.key_code, 65);
    assert_eq!(keyboard.scan_code, None);
    assert_eq!(keyboard.physical_key, "KeyA");
    assert_eq!(keyboard.logical_key, "A");
    assert_eq!(keyboard.text.as_deref(), Some("A"));
    assert!(keyboard.metadata.modifiers.shift);
    assert!(keyboard.metadata.modifiers.control);
    assert!(keyboard.metadata.modifiers.super_key);
    assert!(!keyboard.metadata.modifiers.alt);
    assert!(keyboard.metadata.synthetic);
}

#[test]
fn native_keyboard_translation_preserves_native_scan_codes_and_named_release() {
    let event = KeyEvent {
        physical_key: PhysicalKey::Unidentified(NativeKeyCode::Windows(0x1c)),
        logical_key: Key::Named(NamedKey::Enter),
        text: Some("\r".into()),
        location: KeyLocation::Standard,
        state: ElementState::Released,
        repeat: false,
        text_with_all_modifiers: Some("\r".into()),
        key_without_modifiers: Key::Named(NamedKey::Enter),
    };

    let shared =
        native_keyboard_event_to_shared_input(input_metadata(), &event, ModifiersState::ALT, false);

    let UiInputEvent::Keyboard(keyboard) = shared else {
        panic!("native keyboard translation should produce keyboard input");
    };
    assert_eq!(keyboard.state, UiKeyboardInputState::Released);
    assert_eq!(keyboard.key_code, 13);
    assert_eq!(keyboard.scan_code, Some(0x1c));
    assert_eq!(keyboard.physical_key, "Windows(0x001C)");
    assert_eq!(keyboard.logical_key, "Enter");
    assert_eq!(keyboard.text.as_deref(), Some("\r"));
    assert!(keyboard.metadata.modifiers.alt);
    assert!(!keyboard.metadata.synthetic);
}

#[test]
fn native_ime_translation_maps_preedit_commit_and_disable() {
    let preedit = native_ime_event_to_shared_input(
        input_metadata(),
        &Ime::Preedit("a b".to_string(), Some((1, 3))),
    )
    .expect("preedit should translate");
    let UiInputEvent::Ime(preedit) = preedit else {
        panic!("preedit should produce IME input");
    };
    assert_eq!(preedit.kind, UiImeInputEventKind::Preedit);
    assert_eq!(preedit.text, "a b");
    assert_eq!(preedit.cursor_range.unwrap().start_byte, 1);
    assert_eq!(preedit.cursor_range.unwrap().end_byte, 3);

    let commit = native_ime_event_to_shared_input(input_metadata(), &Ime::Commit("啊".to_string()))
        .expect("commit should translate");
    let UiInputEvent::Text(commit) = commit else {
        panic!("IME commit should produce text input");
    };
    assert_eq!(commit.text, "啊");

    let disabled = native_ime_event_to_shared_input(input_metadata(), &Ime::Disabled)
        .expect("disable should translate");
    let UiInputEvent::Ime(disabled) = disabled else {
        panic!("IME disable should produce cancel input");
    };
    assert_eq!(disabled.kind, UiImeInputEventKind::Cancel);
    assert!(disabled.text.is_empty());
    assert_eq!(disabled.cursor_range, None);

    assert_eq!(
        native_ime_event_to_shared_input(input_metadata(), &Ime::Enabled),
        None
    );
    assert_eq!(
        native_ime_event_to_shared_input(
            input_metadata(),
            &Ime::DeleteSurrounding {
                before_bytes: 2,
                after_bytes: 1,
            },
        ),
        None,
    );
}

#[test]
fn native_wheel_translation_preserves_precise_pixel_xy_and_legacy_scalar() {
    let shared = native_mouse_wheel_event_to_shared_input(
        input_metadata(),
        UiPoint::new(24.0, 36.0),
        MouseScrollDelta::PixelDelta(PhysicalPosition::new(1.25, -8.0)),
    );

    let UiInputEvent::Pointer(pointer) = shared else {
        panic!("wheel translation should produce pointer input");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
    assert_eq!(pointer.event.point, UiPoint::new(24.0, 36.0));
    assert_eq!(pointer.event.scroll_delta, -0.8);
    assert_eq!(
        pointer.precise_scroll,
        Some(UiPreciseScrollDelta::pixels(1.25, -8.0))
    );
}

#[test]
fn native_wheel_translation_preserves_line_xy() {
    let shared = native_mouse_wheel_event_to_shared_input(
        input_metadata(),
        UiPoint::new(4.0, 6.0),
        MouseScrollDelta::LineDelta(2.0, -3.5),
    );

    let UiInputEvent::Pointer(pointer) = shared else {
        panic!("wheel translation should produce pointer input");
    };
    assert_eq!(pointer.event.scroll_delta, -3.5);
    assert_eq!(
        pointer.precise_scroll,
        Some(UiPreciseScrollDelta::lines(2.0, -3.5))
    );
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(123), UiInputSequence::new(7));
    metadata.window_id = Some(UiWindowId::new("editor.main"));
    metadata
}
