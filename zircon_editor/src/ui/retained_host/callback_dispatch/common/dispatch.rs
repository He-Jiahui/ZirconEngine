use std::collections::BTreeMap;

use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use crate::core::editor_event::{EditorEventEnvelope, EditorEventSource};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;
use crate::ui::workbench::event::operation_path_for_menu_action;
use crate::ui::workbench::event::{dispatch_editor_host_binding, EditorHostEvent};
use serde_json::{Number, Value};
use zircon_runtime_interface::ui::{
    binding::UiBindingValue, component::UiValue, dispatch::UiTemplateActionInvocation,
};

pub(crate) fn dispatch_envelope(
    runtime: &EditorHostEventController,
    envelope: EditorEventEnvelope,
) -> Result<UiHostEventEffects, String> {
    let record = runtime
        .dispatch_envelope(envelope)
        .map_err(|error| error.to_string())?;
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

pub(crate) fn dispatch_editor_binding(
    runtime: &EditorHostEventController,
    binding: EditorUiBinding,
) -> Result<UiHostEventEffects, String> {
    if is_reference_preview_action(&binding) {
        let record = runtime
            .dispatch_binding(binding, EditorEventSource::RetainedHost)
            .map_err(|error| error.to_string())?;
        let mut effects = UiHostEventEffects::default();
        apply_record_effects(&mut effects, &record);
        return Ok(effects);
    }

    if let Some(invocation) = operation_invocation_for_binding(&binding)? {
        let record = runtime
            .invoke_operation_with_binding_path(
                EditorOperationSource::UiBinding,
                invocation,
                Some(binding.path().native_prefix()),
            )
            .map_err(|error| error.to_string())?;
        let mut effects = UiHostEventEffects::default();
        apply_record_effects(&mut effects, &record);
        return Ok(effects);
    }

    let record = runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .map_err(|error| error.to_string())?;
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

pub(crate) fn dispatch_template_action_invocation(
    runtime: &EditorHostEventController,
    action: &UiTemplateActionInvocation,
) -> Result<UiHostEventEffects, String> {
    if let Some(binding) = editor_binding_for_template_action(action)? {
        return dispatch_editor_binding(runtime, binding);
    }
    let operation = EditorOperationPath::parse(action.target_id().to_string())
        .map_err(|error| error.to_string())?;
    let invocation = EditorOperationInvocation::new(operation)
        .with_arguments(ui_template_action_payload_to_json(&action.payload));
    let record = runtime
        .invoke_operation(EditorOperationSource::UiBinding, invocation)
        .map_err(|error| error.to_string())?;
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

fn editor_binding_for_template_action(
    action: &UiTemplateActionInvocation,
) -> Result<Option<EditorUiBinding>, String> {
    if !action.is_action() {
        return Ok(None);
    }
    if !action.payload.is_empty() {
        return Err(format!(
            "editor command action {} must not declare a route payload",
            action.target_id()
        ));
    }
    Ok(Some(EditorUiBinding::new(
        "TemplateAction",
        action.target_id(),
        crate::ui::binding::EditorUiEventKind::Click,
        EditorUiBindingPayload::EditorCommand {
            command_id: action.target_id().to_string(),
        },
    )))
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
    value.to_json_value()
}

fn ui_template_action_payload_to_json(payload: &BTreeMap<String, UiValue>) -> Value {
    Value::Object(
        payload
            .iter()
            .map(|(key, value)| (key.clone(), ui_value_to_json(value)))
            .collect(),
    )
}

fn ui_value_to_json(value: &UiValue) -> Value {
    match value {
        UiValue::Bool(value) => Value::Bool(*value),
        UiValue::Int(value) => Value::Number(Number::from(*value)),
        UiValue::Float(value) => Number::from_f64(*value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        UiValue::String(value)
        | UiValue::Color(value)
        | UiValue::AssetRef(value)
        | UiValue::InstanceRef(value)
        | UiValue::Enum(value) => Value::String(value.clone()),
        UiValue::Vec2(value) => Value::Array(
            value
                .iter()
                .map(|value| {
                    Number::from_f64(*value)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                })
                .collect(),
        ),
        UiValue::Vec3(value) => Value::Array(
            value
                .iter()
                .map(|value| {
                    Number::from_f64(*value)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                })
                .collect(),
        ),
        UiValue::Vec4(value) => Value::Array(
            value
                .iter()
                .map(|value| {
                    Number::from_f64(*value)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                })
                .collect(),
        ),
        UiValue::Array(values) => Value::Array(values.iter().map(ui_value_to_json).collect()),
        UiValue::Map(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), ui_value_to_json(value)))
                .collect(),
        ),
        UiValue::Flags(values) => Value::Array(values.iter().cloned().map(Value::String).collect()),
        UiValue::Null => Value::Null,
    }
}

fn is_reference_preview_action(binding: &EditorUiBinding) -> bool {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => is_workbench_preview_action(action_id),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::{
        CommandEvalCtx, EditorCommandDispatchError, EditorCommandRegistry,
    };

    #[test]
    fn template_action_payload_preserves_typed_object_arguments() {
        let payload = BTreeMap::from([
            ("surface_entity".to_string(), UiValue::Int(73)),
            ("force_full_rebuild".to_string(), UiValue::Bool(true)),
            (
                "nested".to_string(),
                UiValue::Map(BTreeMap::from([(
                    "kind".to_string(),
                    UiValue::String("tile".to_string()),
                )])),
            ),
        ]);

        assert_eq!(
            ui_template_action_payload_to_json(&payload),
            serde_json::json!({
                "surface_entity": 73,
                "force_full_rebuild": true,
                "nested": { "kind": "tile" },
            })
        );
    }

    #[test]
    fn template_editor_action_projects_to_the_canonical_editor_command_payload() {
        let action = UiTemplateActionInvocation::action("view.console.clear");
        let binding = editor_binding_for_template_action(&action)
            .expect("action identity should be valid")
            .expect("editor action should project to a binding");

        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id }
                if command_id == "view.console.clear"
        ));
    }

    #[test]
    fn template_editor_action_keeps_registry_disabled_command_policy() {
        let action = UiTemplateActionInvocation::action("runtime.play_mode.exit");
        let binding = editor_binding_for_template_action(&action)
            .expect("action identity should be valid")
            .expect("editor action should project to a binding");
        let EditorUiBindingPayload::EditorCommand { command_id } = binding.payload() else {
            panic!("template editor action must project to an EditorCommand payload");
        };

        let error = EditorCommandRegistry::default_workbench()
            .event_for_command(command_id, &CommandEvalCtx::interactive())
            .expect_err("exit-play must be disabled while the editor is not playing");

        assert!(matches!(
            error,
            EditorCommandDispatchError::DisabledByWhen { command_id }
                if command_id.as_str() == "runtime.play_mode.exit"
        ));
    }
}
