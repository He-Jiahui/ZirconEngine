use crate::core::framework::input::{InputButton, InputSnapshot};
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
    let snapshot = input.snapshot();
    Ok(ScriptHostValue::Bool(snapshot_key_pressed(&snapshot, key)))
}

fn snapshot_key_pressed(snapshot: &InputSnapshot, key: String) -> bool {
    parse_key_code(&key)
        .map(InputButton::KeyCode)
        .map(|button| snapshot.pressed_buttons.contains(&button))
        .unwrap_or_else(|| snapshot.pressed_buttons.contains(&InputButton::Key(key)))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::input::{InputButton, InputSnapshot};

    use super::snapshot_key_pressed;

    #[test]
    fn gameplay_key_query_reads_the_lightweight_snapshot_for_codes_and_names() {
        let snapshot = InputSnapshot {
            pressed_buttons: vec![
                InputButton::KeyCode(87),
                InputButton::Key("Jump".to_string()),
            ],
            ..InputSnapshot::default()
        };

        assert!(snapshot_key_pressed(&snapshot, "KeyCode:87".to_string()));
        assert!(snapshot_key_pressed(&snapshot, "Jump".to_string()));
        assert!(!snapshot_key_pressed(&snapshot, "Missing".to_string()));
    }
}
