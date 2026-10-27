//! 脚本按键查询直达本帧 InputManager 状态，按名称或代码折叠到同一类型化按钮；调用时须存在当前运行时上下文。
use crate::core::framework::input::{
    InputButton, InputFrameSnapshot, MouseScrollUnit, PIXEL_SCROLL_LINE_DELTA_SCALE,
};
use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostValue, ScriptHostValueRef,
};
use crate::core::manager::{input_manager_handle, resolve_manager_service};
use crate::script::runtime_context_for_frame;

use super::values::{parse_key_code, script_core_error, with_string};

pub(super) fn key_pressed(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let core = runtime.core_handle()?;
    let input = input_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    with_string(context, 0, |key: &str| {
        Ok(ScriptHostValue::Bool(
            input.button_pressed(&script_input_button(key)),
        ))
    })
}

fn script_input_button(key: &str) -> InputButton {
    // A single digit is a logical key name. Raw codes remain explicit as KeyCode:n.
    if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() {
        return InputButton::Key(key.to_string());
    }
    parse_key_code(key)
        .map(InputButton::KeyCode)
        .unwrap_or_else(|| InputButton::Key(key.to_string()))
}

pub(super) fn mouse_pressed(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    // Numeric identities match the runtime host ABI: left=1, right=2, middle=3.
    let button = context.arguments.with_argument(0, |value| match value {
        ScriptHostValueRef::Int(1) => Ok(InputButton::MouseLeft),
        ScriptHostValueRef::Int(2) => Ok(InputButton::MouseRight),
        ScriptHostValueRef::Int(3) => Ok(InputButton::MouseMiddle),
        ScriptHostValueRef::Int(value) => Err(ScriptHostError::new(format!(
            "invalid mouse button {value}; expected 1 (left), 2 (right), or 3 (middle)"
        ))),
        value => Err(ScriptHostError::new(format!(
            "argument 0 expected mouse button integer, received {:?}",
            value.kind()
        ))),
    })?;
    let runtime = runtime_context_for_frame(context)?;
    let core = runtime.core_handle()?;
    let input = input_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    Ok(ScriptHostValue::Bool(input.button_pressed(&button)))
}

pub(super) fn mouse_x(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| snapshot.cursor_position[0])
}

pub(super) fn mouse_y(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| snapshot.cursor_position[1])
}

pub(super) fn mouse_delta_x(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| snapshot.mouse_motion_accumulator[0])
}

pub(super) fn mouse_delta_y(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| snapshot.mouse_motion_accumulator[1])
}

pub(super) fn mouse_wheel_x(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| {
        // Normalize each event before adding so mixed line/pixel frames remain meaningful.
        snapshot
            .mouse_wheel_events
            .iter()
            .map(|wheel| match wheel.unit {
                MouseScrollUnit::Line => wheel.x,
                MouseScrollUnit::Pixel => wheel.x * PIXEL_SCROLL_LINE_DELTA_SCALE,
            })
            .sum()
    })
}

pub(super) fn mouse_wheel_y(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    mouse_snapshot_value(context, |snapshot| snapshot.wheel_accumulator)
}

fn mouse_snapshot_value(
    context: &ScriptHostCallFrame<'_>,
    read: impl FnOnce(&InputFrameSnapshot) -> f32,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let core = runtime.core_handle()?;
    let input = input_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    Ok(ScriptHostValue::Float(read(&input.frame_snapshot()) as f64))
}

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;
