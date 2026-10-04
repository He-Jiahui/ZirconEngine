//! Winit 鼠标和触摸身份、阶段及单位到 Runtime ABI 的适配。
//! 不支持的按钮返回 None，单位转换和输入策略仍由 Runtime 消费者解释。

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
#[path = "tests/pointer.rs"]
mod tests;
