mod ime;
mod keyboard;
mod pointer;
mod window;

use winit::event::WindowEvent;
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    window::{
        UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent, UiWindowPixelPosition,
        UiWindowRedrawReason,
    },
};

use ime::translate_ime_event;
use keyboard::translate_keyboard_event;
pub use keyboard::translate_winit_modifiers;
use pointer::{
    translate_mouse_wheel_event, translate_pointer_button, translate_pointer_entered,
    translate_pointer_left, translate_pointer_moved,
};
use window::{input_event, window_event, window_metrics_from_physical_size};

pub fn translate_winit_window_event(
    context: UiWindowInputContext,
    event: &WindowEvent,
) -> Option<UiWindowInputPumpEvent> {
    match event {
        WindowEvent::CloseRequested => {
            Some(window_event(&context, UiWindowEventKind::CloseRequested))
        }
        WindowEvent::SurfaceResized(size) => Some(window_event(
            &context,
            UiWindowEventKind::Resized {
                metrics: window_metrics_from_physical_size(*size, context.window_metrics),
            },
        )),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => Some(window_event(
            &context,
            UiWindowEventKind::ScaleFactorChanged {
                scale_factor: *scale_factor,
            },
        )),
        WindowEvent::Moved(position) => Some(window_event(
            &context,
            UiWindowEventKind::Moved {
                position: UiWindowPixelPosition::new(position.x, position.y),
            },
        )),
        WindowEvent::PointerMoved {
            position, source, ..
        } => translate_pointer_moved(context, *position, source),
        WindowEvent::PointerEntered { position, kind, .. } => {
            translate_pointer_entered(&context, *position, kind)
        }
        WindowEvent::PointerLeft { position, kind, .. } => {
            translate_pointer_left(context, *position, kind)
        }
        WindowEvent::PointerButton {
            state,
            button,
            position,
            ..
        } => translate_pointer_button(context, *state, button.clone(), *position),
        WindowEvent::KeyboardInput {
            event,
            is_synthetic,
            ..
        } => Some(input_event(translate_keyboard_event(
            context,
            event,
            *is_synthetic,
        ))),
        WindowEvent::Ime(event) => translate_ime_event(context, event),
        WindowEvent::MouseWheel { delta, .. } => Some(input_event(translate_mouse_wheel_event(
            context,
            UiPoint::default(),
            *delta,
        ))),
        WindowEvent::RedrawRequested => Some(window_event(
            &context,
            UiWindowEventKind::RequestRedraw {
                reason: UiWindowRedrawReason::Host,
            },
        )),
        WindowEvent::Focused(focused) => Some(window_event(
            &context,
            UiWindowEventKind::Focused { focused: *focused },
        )),
        WindowEvent::Occluded(occluded) => Some(window_event(
            &context,
            UiWindowEventKind::Occluded {
                occluded: *occluded,
            },
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use winit::event::{
        ButtonSource, ElementState, FingerId, Ime, KeyEvent, MouseScrollDelta, PointerKind,
        PointerSource, SurfaceSizeWriter,
    };
    use winit::keyboard::{
        Key, KeyCode, KeyLocation, ModifiersState, NamedKey, NativeKeyCode, PhysicalKey,
    };
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence,
            UiInputTimestamp, UiKeyboardInputState, UiPointerId, UiPointerSource,
            UiPreciseScrollDelta, UiWindowId,
        },
        layout::{UiPoint, UiSize},
        surface::{UiPointerButton, UiPointerEventKind},
        window::{
            UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent, UiWindowMetrics,
        },
    };

    use super::{
        translate_ime_event, translate_keyboard_event, translate_mouse_wheel_event,
        translate_winit_modifiers,
    };

    #[test]
    fn translate_winit_keyboard_matrix_matches_runtime_input_baseline() {
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
        let input = translate_keyboard_event(
            input_context().with_modifiers(translate_winit_modifiers(
                ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::META,
            )),
            &event,
            true,
        )
        .normalize();

        let UiInputEvent::Keyboard(keyboard) = input else {
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
    fn translate_winit_keyboard_preserves_native_scan_codes_and_named_release() {
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
        let input = translate_keyboard_event(
            input_context().with_modifiers(translate_winit_modifiers(ModifiersState::ALT)),
            &event,
            false,
        )
        .normalize();

        let UiInputEvent::Keyboard(keyboard) = input else {
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
    fn translate_winit_window_keyboard_preserves_synthetic_flag() {
        let event = KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::KeyP),
            logical_key: Key::Character("P".into()),
            text: Some("P".into()),
            location: KeyLocation::Standard,
            state: ElementState::Pressed,
            repeat: false,
            text_with_all_modifiers: Some("P".into()),
            key_without_modifiers: Key::Character("p".into()),
        };
        let input = super::translate_winit_window_event(
            input_context(),
            &winit::event::WindowEvent::KeyboardInput {
                device_id: None,
                event,
                is_synthetic: true,
            },
        )
        .expect("keyboard input should translate");

        let UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(keyboard)) = input else {
            panic!("keyboard window event should produce keyboard input");
        };
        assert!(keyboard.metadata.synthetic);
    }

    #[test]
    fn translate_winit_ime_preedit_commit_and_disable_match_runtime_input_baseline() {
        let preedit = translate_ime_event(
            input_context(),
            &Ime::Preedit("a b".to_string(), Some((1, 3))),
        )
        .expect("preedit should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(preedit)) = preedit else {
            panic!("preedit should produce IME input");
        };
        assert_eq!(preedit.kind, UiImeInputEventKind::Preedit);
        assert_eq!(preedit.text, "a b");
        assert_eq!(preedit.cursor_range.unwrap().start_byte, 1);
        assert_eq!(preedit.cursor_range.unwrap().end_byte, 3);
        assert!(preedit.preedit_clauses.is_empty());

        let commit = translate_ime_event(input_context(), &Ime::Commit("text".to_string()))
            .expect("commit should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(commit)) = commit else {
            panic!("IME commit should preserve its input-method semantics");
        };
        assert_eq!(commit.kind, UiImeInputEventKind::Commit);
        assert_eq!(commit.text, "text");
        assert_eq!(commit.cursor_range, None);

        let disabled = translate_ime_event(input_context(), &Ime::Disabled)
            .expect("disabled should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(disabled)) = disabled else {
            panic!("IME disabled should produce cancel input");
        };
        assert_eq!(disabled.kind, UiImeInputEventKind::Cancel);
        assert!(disabled.text.is_empty());
        assert_eq!(disabled.cursor_range, None);

        assert_eq!(translate_ime_event(input_context(), &Ime::Enabled), None);
        let delete_surrounding = translate_ime_event(
            input_context(),
            &Ime::DeleteSurrounding {
                before_bytes: 2,
                after_bytes: 1,
            },
        )
        .expect("delete surrounding should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Ime(delete_surrounding)) =
            delete_surrounding
        else {
            panic!("delete surrounding should produce IME input");
        };
        assert_eq!(
            delete_surrounding.kind,
            UiImeInputEventKind::DeleteSurrounding
        );
        assert_eq!(
            delete_surrounding.delete_surrounding,
            Some(zircon_runtime_interface::ui::dispatch::UiImeDeleteSurrounding::new(2, 1))
        );
    }

    #[test]
    fn translate_winit_wheel_preserves_precise_delta_and_line_delta_scale() {
        let input = translate_mouse_wheel_event(
            input_context(),
            UiPoint::new(24.0, 36.0),
            MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(1.25, -8.0)),
        )
        .normalize();
        let UiInputEvent::Pointer(pointer) = input else {
            panic!("wheel translation should produce pointer input");
        };

        assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
        assert_eq!(pointer.event.point, UiPoint::new(24.0, 36.0));
        assert_eq!(pointer.event.scroll_delta, -0.8);
        assert_eq!(
            pointer.precise_scroll,
            Some(UiPreciseScrollDelta::pixels(1.25, -8.0))
        );

        let input = translate_mouse_wheel_event(
            input_context(),
            UiPoint::new(4.0, 6.0),
            MouseScrollDelta::LineDelta(2.0, -3.5),
        )
        .normalize();
        let UiInputEvent::Pointer(pointer) = input else {
            panic!("wheel translation should produce pointer input");
        };
        assert_eq!(pointer.event.scroll_delta, -3.5);
        assert_eq!(
            pointer.precise_scroll,
            Some(UiPreciseScrollDelta::lines(2.0, -3.5))
        );
    }

    #[test]
    fn translate_winit_touch_phase_maps_pointer_id_through_runtime_platform_input() {
        let cases = [
            (
                zircon_runtime_interface::ui::window::UiWindowTouchPhase::Started,
                UiPointerEventKind::Down,
                Some(UiPointerButton::Primary),
            ),
            (
                zircon_runtime_interface::ui::window::UiWindowTouchPhase::Moved,
                UiPointerEventKind::Move,
                None,
            ),
            (
                zircon_runtime_interface::ui::window::UiWindowTouchPhase::Ended,
                UiPointerEventKind::Up,
                Some(UiPointerButton::Primary),
            ),
            (
                zircon_runtime_interface::ui::window::UiWindowTouchPhase::Canceled,
                UiPointerEventKind::Cancel,
                None,
            ),
        ];

        for (index, (phase, kind, button)) in cases.into_iter().enumerate() {
            let pointer_id = UiPointerId::new(40 + index as u64);
            let point = UiPoint::new(8.0 + index as f32, 16.0 + index as f32);
            let shared = zircon_runtime_interface::ui::window::UiWindowPlatformInputEvent::touch(
                input_context(),
                phase,
                pointer_id,
                point,
            )
            .normalize();

            let UiInputEvent::Pointer(pointer) = shared else {
                panic!("touch platform input should normalize to pointer input");
            };
            assert_eq!(pointer.metadata.pointer_id, Some(pointer_id));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
            assert_eq!(pointer.event.kind, kind);
            assert_eq!(pointer.event.button, button);
            assert_eq!(pointer.event.point, point);
            assert_eq!(pointer.precise_scroll, None);
        }
    }

    #[test]
    fn translate_winit_pointer_touch_events_map_to_touch_platform_input() {
        let finger_id = FingerId::from_raw(77);
        let down = super::translate_winit_window_event(
            input_context(),
            &winit::event::WindowEvent::PointerButton {
                device_id: None,
                state: ElementState::Pressed,
                position: winit::dpi::PhysicalPosition::new(10.0, 20.0),
                primary: true,
                button: ButtonSource::Touch {
                    finger_id,
                    force: None,
                },
            },
        )
        .expect("touch press should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = down else {
            panic!("touch press should produce pointer input");
        };
        assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
        assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
        assert_eq!(pointer.event.kind, UiPointerEventKind::Down);
        assert_eq!(pointer.event.button, Some(UiPointerButton::Primary));

        let moved = super::translate_winit_window_event(
            input_context(),
            &winit::event::WindowEvent::PointerMoved {
                device_id: None,
                position: winit::dpi::PhysicalPosition::new(12.0, 22.0),
                primary: true,
                source: PointerSource::Touch {
                    finger_id,
                    force: None,
                },
            },
        )
        .expect("touch move should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = moved else {
            panic!("touch move should produce pointer input");
        };
        assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
        assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
        assert_eq!(pointer.event.point, UiPoint::new(12.0, 22.0));

        let canceled = super::translate_winit_window_event(
            input_context(),
            &winit::event::WindowEvent::PointerLeft {
                device_id: None,
                position: Some(winit::dpi::PhysicalPosition::new(14.0, 24.0)),
                primary: true,
                kind: PointerKind::Touch(finger_id),
            },
        )
        .expect("touch cancel should translate");
        let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = canceled else {
            panic!("touch cancel should produce pointer input");
        };
        assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(77)));
        assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
        assert_eq!(pointer.event.point, UiPoint::new(14.0, 24.0));
    }

    #[test]
    fn translate_winit_modifiers_matches_shared_contract() {
        let modifiers = translate_winit_modifiers(
            ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::META,
        );

        assert!(modifiers.shift);
        assert!(modifiers.control);
        assert!(modifiers.super_key);
        assert!(!modifiers.alt);
        assert!(!modifiers.caps_lock);
        assert!(!modifiers.num_lock);
    }

    #[test]
    fn translate_winit_window_surface_events_use_window_pump_contract() {
        let context = input_context();
        let resized = super::translate_winit_window_event(
            context.clone(),
            &winit::event::WindowEvent::SurfaceResized(winit::dpi::PhysicalSize::new(1280, 720)),
        )
        .expect("resized should translate");
        let UiWindowInputPumpEvent::Window(resized) = resized else {
            panic!("resized should produce window pump event");
        };
        let UiWindowEventKind::Resized { metrics } = resized.kind else {
            panic!("resized should preserve metrics");
        };
        assert_eq!(metrics.physical_size.width, 1280);
        assert_eq!(metrics.physical_size.height, 720);
        assert_eq!(metrics.logical_size.width, 1280.0);
        assert_eq!(metrics.logical_size.height, 720.0);

        let suggested_size = Arc::new(Mutex::new(winit::dpi::PhysicalSize::new(2560, 1440)));
        let scale_changed = super::translate_winit_window_event(
            context.clone(),
            &winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor: 2.0,
                surface_size_writer: SurfaceSizeWriter::new(Arc::downgrade(&suggested_size)),
            },
        )
        .expect("scale-factor change should translate");
        let UiWindowInputPumpEvent::Window(scale_changed) = scale_changed else {
            panic!("scale-factor change should produce window pump event");
        };
        let UiWindowEventKind::ScaleFactorChanged { scale_factor } = scale_changed.kind else {
            panic!("scale-factor change should preserve the DPI payload");
        };
        assert_eq!(scale_factor, 2.0);

        let resized_after_scale = super::translate_winit_window_event(
            context.clone().with_window_metrics(UiWindowMetrics::new(
                UiSize::new(1280.0, 720.0),
                zircon_runtime_interface::ui::window::UiWindowPixelSize::new(1280, 720),
                2.0,
            )),
            &winit::event::WindowEvent::SurfaceResized(winit::dpi::PhysicalSize::new(2560, 1440)),
        )
        .expect("resized surface should preserve the prior DPI scale");
        let UiWindowInputPumpEvent::Window(resized_after_scale) = resized_after_scale else {
            panic!("resized surface should produce window pump event");
        };
        let UiWindowEventKind::Resized { metrics } = resized_after_scale.kind else {
            panic!("resized surface should preserve metrics");
        };
        assert_eq!(metrics.scale_factor, 2.0);
        assert_eq!(metrics.logical_size, UiSize::new(1280.0, 720.0));
        assert_eq!(metrics.physical_size.width, 2560);
        assert_eq!(metrics.physical_size.height, 1440);

        let redraw = super::translate_winit_window_event(
            context,
            &winit::event::WindowEvent::RedrawRequested,
        )
        .expect("redraw should translate");
        let UiWindowInputPumpEvent::Window(redraw) = redraw else {
            panic!("redraw should produce window pump event");
        };
        assert!(matches!(
            redraw.kind,
            UiWindowEventKind::RequestRedraw {
                reason: zircon_runtime_interface::ui::window::UiWindowRedrawReason::Host
            }
        ));
    }

    fn input_context() -> UiWindowInputContext {
        UiWindowInputContext {
            metadata: input_metadata(),
            ..UiWindowInputContext::default()
        }
    }

    fn input_metadata() -> UiInputEventMetadata {
        let mut metadata =
            UiInputEventMetadata::new(UiInputTimestamp::from_micros(123), UiInputSequence::new(7));
        metadata.window_id = Some(UiWindowId::new("runtime.window"));
        metadata
    }
}
