use super::*;
use winit::dpi::PhysicalPosition;
use winit::event::FingerId;

#[test]
fn button_states_map_to_runtime_constants() {
    assert_eq!(
        button_state(ElementState::Pressed),
        Some(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1)
    );
    assert_eq!(
        button_state(ElementState::Released),
        Some(ZR_RUNTIME_BUTTON_STATE_RELEASED_V1)
    );
}

#[test]
fn pointer_touch_ids_and_touch_button_phases_are_preserved() {
    let finger_id = FingerId::from_raw(42);
    let source = PointerSource::Touch {
        finger_id,
        force: None,
    };
    let button = ButtonSource::Touch {
        finger_id,
        force: None,
    };

    assert_eq!(pointer_source_touch_id(&source), Some(42));
    assert_eq!(
        pointer_kind_touch_id(PointerKind::Touch(finger_id)),
        Some(42)
    );
    assert_eq!(
        touch_button_phase(&button, ElementState::Pressed),
        Some((42, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1))
    );
    assert_eq!(
        touch_button_phase(&button, ElementState::Released),
        Some((42, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1))
    );
    assert_eq!(pointer_source_touch_id(&PointerSource::Mouse), None);
    assert_eq!(
        touch_button_phase(
            &ButtonSource::Mouse(MouseButton::Left),
            ElementState::Pressed
        ),
        None
    );
}

#[test]
fn mouse_buttons_and_wheel_delta_use_runtime_abi_constants() {
    assert_eq!(
        mouse_button(ButtonSource::Mouse(MouseButton::Left)),
        Some(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1)
    );
    assert_eq!(
        mouse_button(ButtonSource::Mouse(MouseButton::Right)),
        Some(ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1)
    );
    assert_eq!(
        mouse_button(ButtonSource::Mouse(MouseButton::Middle)),
        Some(ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1)
    );
    assert_eq!(mouse_button(ButtonSource::Unknown(9)), None);
    assert_eq!(
        mouse_wheel_delta(MouseScrollDelta::LineDelta(1.5, -2.0)),
        (ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, 1.5, -2.0)
    );
    assert_eq!(
        mouse_wheel_delta(MouseScrollDelta::PixelDelta(PhysicalPosition::new(
            8.0, -9.0
        ))),
        (ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1, 8.0, -9.0)
    );
}
