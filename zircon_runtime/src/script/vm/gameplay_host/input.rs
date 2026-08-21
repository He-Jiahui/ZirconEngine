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
mod tests {
    use crate::core::framework::input::InputButton;

    use super::script_input_button;

    #[test]
    fn gameplay_key_query_compiles_codes_and_names_to_typed_buttons() {
        assert_eq!(script_input_button("KeyCode:87"), InputButton::KeyCode(87));
        assert_eq!(
            script_input_button("Jump"),
            InputButton::Key("Jump".to_string())
        );
    }

    #[test]
    fn gameplay_key_query_uses_direct_manager_lookup_without_snapshot_clone() {
        let source = include_str!("input.rs");
        let key_pressed = source
            .split("pub(super) fn key_pressed")
            .nth(1)
            .and_then(|source| source.split("fn script_input_button").next())
            .expect("key_pressed source section");

        assert!(key_pressed.contains("input.button_pressed"));
        assert!(key_pressed.contains("script_input_button(key)"));
        assert!(!key_pressed.contains("input.snapshot()"));
    }
}
