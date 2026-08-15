use winit::dpi::PhysicalPosition;
use winit::event::{
    ButtonSource, ElementState, FingerId, Ime, KeyEvent, MouseScrollDelta, PointerKind,
    PointerSource, TouchPhase, WindowEvent,
};
use winit::keyboard::{
    Key, KeyCode, KeyLocation, ModifiersState, NamedKey, NativeKeyCode, PhysicalKey,
};
use zircon_runtime::ui::platform_input::{translate_winit_modifiers, translate_winit_window_event};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeDeleteSurrounding, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
        UiInputSequence, UiInputTimestamp, UiKeyboardInputState, UiPointerId, UiPointerSource,
        UiPreciseScrollDelta, UiWindowId,
    },
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
    window::{UiWindowInputContext, UiWindowInputPumpEvent},
};

#[test]
fn runtime_keyboard_translation_preserves_repeat_text_modifiers_keys_and_synthetic_flag() {
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

    let shared = translate_input(
        WindowEvent::KeyboardInput {
            device_id: None,
            event,
            is_synthetic: true,
        },
        ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::META,
    );

    let UiInputEvent::Keyboard(keyboard) = shared else {
        panic!("keyboard translation should produce keyboard input");
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
fn runtime_keyboard_translation_preserves_native_scan_codes_and_named_release() {
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

    let shared = translate_input(
        WindowEvent::KeyboardInput {
            device_id: None,
            event,
            is_synthetic: false,
        },
        ModifiersState::ALT,
    );

    let UiInputEvent::Keyboard(keyboard) = shared else {
        panic!("keyboard translation should produce keyboard input");
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
fn runtime_ime_translation_maps_preedit_commit_and_disable() {
    let preedit = translate_window_event(WindowEvent::Ime(Ime::Preedit(
        "a b".to_string(),
        Some((1, 3)),
    )))
    .expect("preedit should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(preedit)) = preedit else {
        panic!("preedit should produce IME input");
    };
    assert_eq!(preedit.kind, UiImeInputEventKind::Preedit);
    assert_eq!(preedit.text, "a b");
    assert_eq!(preedit.cursor_range.unwrap().start_byte, 1);
    assert_eq!(preedit.cursor_range.unwrap().end_byte, 3);

    let commit = translate_window_event(WindowEvent::Ime(Ime::Commit("text".to_string())))
        .expect("commit should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Text(commit)) = commit else {
        panic!("IME commit should produce text input");
    };
    assert_eq!(commit.text, "text");

    let disabled =
        translate_window_event(WindowEvent::Ime(Ime::Disabled)).expect("disable should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(disabled)) = disabled else {
        panic!("disable should produce IME input");
    };
    assert_eq!(disabled.kind, UiImeInputEventKind::Cancel);
    assert!(disabled.text.is_empty());
    assert_eq!(disabled.cursor_range, None);

    assert_eq!(translate_window_event(WindowEvent::Ime(Ime::Enabled)), None);
    let delete_surrounding = translate_window_event(WindowEvent::Ime(Ime::DeleteSurrounding {
        before_bytes: 2,
        after_bytes: 1,
    }))
    .expect("delete surrounding should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(delete_surrounding)) = delete_surrounding
    else {
        panic!("delete surrounding should produce IME input");
    };
    assert_eq!(
        delete_surrounding.kind,
        UiImeInputEventKind::DeleteSurrounding
    );
    assert_eq!(
        delete_surrounding.delete_surrounding,
        Some(UiImeDeleteSurrounding::new(2, 1))
    );
}

#[test]
fn runtime_wheel_translation_preserves_precise_pixel_xy_and_legacy_scalar() {
    let shared = translate_input(
        WindowEvent::MouseWheel {
            device_id: None,
            delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(1.25, -8.0)),
            phase: TouchPhase::Moved,
        },
        ModifiersState::empty(),
    );

    let UiInputEvent::Pointer(pointer) = shared else {
        panic!("wheel translation should produce pointer input");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
    assert_eq!(pointer.event.point, UiPoint::default());
    assert_eq!(pointer.event.scroll_delta, -0.8);
    assert_eq!(
        pointer.precise_scroll,
        Some(UiPreciseScrollDelta::pixels(1.25, -8.0))
    );
}

#[test]
fn runtime_wheel_translation_preserves_line_xy() {
    let shared = translate_input(
        WindowEvent::MouseWheel {
            device_id: None,
            delta: MouseScrollDelta::LineDelta(2.0, -3.5),
            phase: TouchPhase::Moved,
        },
        ModifiersState::empty(),
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

#[test]
fn runtime_touch_pointer_events_map_pointer_id_source_kind_and_button() {
    let finger_id = FingerId::from_raw(77);
    let down = translate_window_event(WindowEvent::PointerButton {
        device_id: None,
        state: ElementState::Pressed,
        position: PhysicalPosition::new(10.0, 20.0),
        primary: true,
        button: ButtonSource::Touch {
            finger_id,
            force: None,
        },
    })
    .expect("touch press should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = down else {
        panic!("touch press should produce pointer input");
    };
    assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
    assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
    assert_eq!(pointer.event.kind, UiPointerEventKind::Down);
    assert_eq!(pointer.event.button, Some(UiPointerButton::Primary));

    let moved = translate_window_event(WindowEvent::PointerMoved {
        device_id: None,
        position: PhysicalPosition::new(12.0, 22.0),
        primary: true,
        source: PointerSource::Touch {
            finger_id,
            force: None,
        },
    })
    .expect("touch move should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = moved else {
        panic!("touch move should produce pointer input");
    };
    assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
    assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
    assert_eq!(pointer.event.point, UiPoint::new(12.0, 22.0));

    let canceled = translate_window_event(WindowEvent::PointerLeft {
        device_id: None,
        position: Some(PhysicalPosition::new(14.0, 24.0)),
        primary: true,
        kind: PointerKind::Touch(finger_id),
    })
    .expect("touch cancel should translate");
    let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = canceled else {
        panic!("touch cancel should produce pointer input");
    };
    assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
    assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
    assert_eq!(pointer.event.point, UiPoint::new(14.0, 24.0));
}

#[test]
fn m1_s3_editor_event_loop_routes_winit_input_through_runtime_platform_input() {
    let window_events =
        include_str!("../../../ui/retained_host/host_contract/window/event_loop/events.rs");
    let platform_input =
        include_str!("../../../ui/retained_host/host_contract/window/event_loop/platform_input.rs");
    let host_contract_mod = include_str!("../../../ui/retained_host/host_contract/mod.rs");

    assert!(platform_input.contains("translate_winit_window_event"));
    assert!(platform_input.contains("translate_winit_modifiers"));
    assert!(window_events.contains("translate_platform_input_event(&event)"));
    assert!(window_events.contains("WindowEvent::Ime(_)"));
    assert!(!host_contract_mod.contains("mod native_input_translation;"));
    assert!(!platform_input.contains("native_input_translation"));
    assert!(!window_events.contains("native_input_translation"));
}

fn translate_input(event: WindowEvent, modifiers: ModifiersState) -> UiInputEvent {
    let Some(UiWindowInputPumpEvent::Input(input)) =
        translate_winit_window_event(input_context(modifiers), &event)
    else {
        panic!("window event should translate to input event");
    };
    input
}

fn translate_window_event(event: WindowEvent) -> Option<UiWindowInputPumpEvent> {
    translate_winit_window_event(input_context(ModifiersState::empty()), &event)
}

fn input_context(modifiers: ModifiersState) -> UiWindowInputContext {
    UiWindowInputContext {
        metadata: input_metadata(),
        ..UiWindowInputContext::default()
    }
    .with_modifiers(translate_winit_modifiers(modifiers))
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(123), UiInputSequence::new(7));
    metadata.window_id = Some(UiWindowId::new("editor.main"));
    metadata
}
