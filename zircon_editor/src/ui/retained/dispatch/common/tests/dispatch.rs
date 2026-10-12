use super::*;
use crate::core::commands::{CommandEvalCtx, EditorCommandDispatchError, EditorCommandRegistry};

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
