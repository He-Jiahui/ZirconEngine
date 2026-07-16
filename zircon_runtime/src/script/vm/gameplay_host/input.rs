use crate::core::framework::input::InputButton;
use crate::core::framework::script::{ScriptHostCallContext, ScriptHostError, ScriptHostValue};
use crate::core::manager::{input_manager_handle, resolve_manager_service};
use crate::script::current_script_runtime_call_context;

use super::values::{expect_string, parse_key_code, script_core_error};

pub(super) fn key_pressed(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let key = expect_string(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let core = runtime.core_handle()?;
    let input = input_manager_handle(&core)
        .and_then(|handle| resolve_manager_service(&core, handle))
        .map_err(script_core_error)?;
    let snapshot = input.frame_snapshot();
    let pressed = parse_key_code(&key)
        .map(InputButton::KeyCode)
        .map(|button| snapshot.buttons.pressed(&button))
        .unwrap_or_else(|| snapshot.buttons.pressed(&InputButton::Key(key)));
    Ok(ScriptHostValue::Bool(pressed))
}
