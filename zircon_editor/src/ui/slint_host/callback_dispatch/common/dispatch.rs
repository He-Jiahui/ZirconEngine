use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use crate::core::editor_event::{EditorEventEnvelope, EditorEventRuntime, EditorEventSource};
use crate::ui::slint_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use crate::ui::workbench::event::{dispatch_editor_host_binding, EditorHostEvent};
use crate::ui::workbench::model::operation_path_for_menu_action;
use serde_json::{Number, Value};
use zircon_runtime_interface::ui::binding::UiBindingValue;

pub(crate) fn dispatch_envelope(
    runtime: &EditorEventRuntime,
    envelope: EditorEventEnvelope,
) -> Result<UiHostEventEffects, String> {
    let record = runtime.dispatch_envelope(envelope)?;
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

pub(crate) fn dispatch_editor_binding(
    runtime: &EditorEventRuntime,
    binding: EditorUiBinding,
) -> Result<UiHostEventEffects, String> {
    if let Some(invocation) = operation_invocation_for_binding(&binding)? {
        let record = runtime.invoke_operation(EditorOperationSource::UiBinding, invocation)?;
        let mut effects = UiHostEventEffects::default();
        apply_record_effects(&mut effects, &record);
        return Ok(effects);
    }

    let record = runtime.dispatch_binding(binding, EditorEventSource::Slint)?;
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

fn operation_invocation_for_binding(
    binding: &EditorUiBinding,
) -> Result<Option<EditorOperationInvocation>, String> {
    match binding.payload() {
        EditorUiBindingPayload::EditorOperation {
            operation_id,
            arguments,
        } => {
            let path = EditorOperationPath::parse(operation_id.clone())
                .map_err(|error| error.to_string())?;
            Ok(Some(
                EditorOperationInvocation::new(path)
                    .with_arguments(ui_binding_arguments_to_json(arguments)),
            ))
        }
        EditorUiBindingPayload::MenuAction { .. } => {
            let EditorHostEvent::Menu(action) =
                dispatch_editor_host_binding(binding).map_err(|error| error.to_string())?;
            Ok(operation_path_for_menu_action(&action).map(EditorOperationInvocation::new))
        }
        _ => Ok(None),
    }
}

fn ui_binding_arguments_to_json(arguments: &[UiBindingValue]) -> Value {
    if arguments.is_empty() {
        return Value::Null;
    }
    Value::Array(arguments.iter().map(ui_binding_value_to_json).collect())
}

fn ui_binding_value_to_json(value: &UiBindingValue) -> Value {
    match value {
        UiBindingValue::String(value) => Value::String(value.clone()),
        UiBindingValue::Unsigned(value) => Value::Number(Number::from(*value)),
        UiBindingValue::Signed(value) => Value::Number(Number::from(*value)),
        UiBindingValue::Float(value) => Number::from_f64(*value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        UiBindingValue::Bool(value) => Value::Bool(*value),
        UiBindingValue::Null => Value::Null,
        UiBindingValue::Array(values) => {
            Value::Array(values.iter().map(ui_binding_value_to_json).collect())
        }
    }
}
