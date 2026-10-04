//! 脚本按键查询直达本帧 InputManager 状态，按名称或代码折叠到同一类型化按钮；调用时须存在当前运行时上下文。
use crate::core::framework::input::InputButton;
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
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
    parse_key_code(key)
        .map(InputButton::KeyCode)
        .unwrap_or_else(|| InputButton::Key(key.to_string()))
}

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;
