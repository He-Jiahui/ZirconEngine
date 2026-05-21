use winit::event::{
    ButtonSource, ElementState, MouseButton, MouseScrollDelta, PointerKind, PointerSource,
};
use zircon_runtime_interface::{
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
};

pub(in crate::entry::runtime_entry_app) fn pointer_source_touch_id(
    source: &PointerSource,
) -> Option<u64> {
    match source {
        PointerSource::Touch { finger_id, .. } => Some(finger_id.into_raw() as u64),
        _ => None,
    }
}

pub(in crate::entry::runtime_entry_app) fn pointer_kind_touch_id(kind: PointerKind) -> Option<u64> {
    match kind {
        PointerKind::Touch(finger_id) => Some(finger_id.into_raw() as u64),
        _ => None,
    }
}

pub(in crate::entry::runtime_entry_app) fn touch_button_phase(
    button: &ButtonSource,
    state: ElementState,
) -> Option<(u64, u32)> {
    let ButtonSource::Touch { finger_id, .. } = button else {
        return None;
    };
    let phase = match state {
        ElementState::Pressed => ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
        ElementState::Released => ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    };
    Some((finger_id.into_raw() as u64, phase))
}

pub(in crate::entry::runtime_entry_app) fn mouse_button(button: ButtonSource) -> Option<u32> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1),
        Some(MouseButton::Right) => Some(ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1),
        Some(MouseButton::Middle) => Some(ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1),
        _ => None,
    }
}

pub(in crate::entry::runtime_entry_app) fn button_state(state: ElementState) -> Option<u32> {
    match state {
        ElementState::Pressed => Some(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1),
        ElementState::Released => Some(ZR_RUNTIME_BUTTON_STATE_RELEASED_V1),
    }
}

pub(in crate::entry::runtime_entry_app) fn mouse_wheel_delta(
    delta: MouseScrollDelta,
) -> (u32, f32, f32) {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => (ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, x, y),
        MouseScrollDelta::PixelDelta(position) => (
            ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
            position.x as f32,
            position.y as f32,
        ),
    }
}

#[cfg(test)]
mod tests {
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
}
