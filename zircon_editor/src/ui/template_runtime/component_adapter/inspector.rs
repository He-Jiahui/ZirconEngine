use crate::ui::binding_dispatch::{apply_inspector_draft_field, EditorBindingDispatchError};
use crate::ui::workbench::state::EditorState;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEvent, UiComponentEventEnvelope,
    UiComponentProjectionPatch, UiValue, UiValueKind,
};

const INSPECTOR_DOMAIN: &str = "inspector";
const SELECTED_ENTITY_SUBJECT: &str = "entity://selected";

pub(crate) fn apply_inspector_component_envelope(
    state: &mut EditorState,
    envelope: &UiComponentEventEnvelope,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    if envelope.target.domain != "inspector" {
        return Err(UiComponentAdapterError::UnsupportedTargetDomain {
            domain: envelope.target.domain.clone(),
        });
    }

    let subject = envelope.target.subject.as_deref().ok_or_else(|| {
        UiComponentAdapterError::MissingSource {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            source_name: "subject".to_string(),
        }
    })?;

    if subject != SELECTED_ENTITY_SUBJECT {
        return Err(UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: format!("inspector adapter requires subject {SELECTED_ENTITY_SUBJECT}"),
        });
    }
    validate_field_path(envelope)?;

    match &envelope.event {
        UiComponentEvent::ValueChanged { property, value }
        | UiComponentEvent::Commit { property, value } => {
            validate_value_property(envelope, property)?;
            apply_field_value(state, envelope, value)?;
            Ok(changed_result(envelope, value))
        }
        _ => Err(UiComponentAdapterError::UnsupportedEvent {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            event_kind: envelope.event_kind,
        }),
    }
}

fn validate_field_path(envelope: &UiComponentEventEnvelope) -> Result<(), UiComponentAdapterError> {
    match envelope.target.path.as_str() {
        "name"
        | "parent"
        | "transform.translation.x"
        | "transform.translation.y"
        | "transform.translation.z" => Ok(()),
        _ => Err(UiComponentAdapterError::UnsupportedTargetPath {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
        }),
    }
}

fn validate_value_property(
    envelope: &UiComponentEventEnvelope,
    property: &str,
) -> Result<(), UiComponentAdapterError> {
    if property == "value" {
        return Ok(());
    }

    Err(UiComponentAdapterError::RejectedInput {
        domain: envelope.target.domain.clone(),
        path: envelope.target.path.clone(),
        reason: format!("inspector adapter does not support property {property}"),
    })
}

fn apply_field_value(
    state: &mut EditorState,
    envelope: &UiComponentEventEnvelope,
    value: &UiValue,
) -> Result<(), UiComponentAdapterError> {
    let value = field_string_value(value, envelope)?;
    apply_inspector_draft_field(state, SELECTED_ENTITY_SUBJECT, &envelope.target.path, value)
        .map_err(|error| dispatch_error_to_adapter_error(error, envelope))?;
    Ok(())
}

fn changed_result(
    envelope: &UiComponentEventEnvelope,
    value: &UiValue,
) -> UiComponentAdapterResult {
    UiComponentAdapterResult::changed()
        .with_transaction(format!("inspector:{}", envelope.target.path))
        .with_mutation_source("inspector")
        .with_status(format!(
            "Inspector {} updated via Runtime UI component binding",
            envelope.target.path
        ))
        .with_patch(
            UiComponentProjectionPatch::new(envelope.control_id.clone())
                .with_state_value(envelope.target.path.clone(), value.clone())
                .with_attribute("value", value.clone()),
        )
}

fn field_string_value(
    value: &UiValue,
    envelope: &UiComponentEventEnvelope,
) -> Result<String, UiComponentAdapterError> {
    match value {
        UiValue::String(value) => Ok(value.clone()),
        UiValue::Int(value) => Ok(value.to_string()),
        UiValue::Float(value) => Ok(value.to_string()),
        other => Err(UiComponentAdapterError::InvalidValueKind {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            expected: UiValueKind::String,
            actual: other.kind(),
        }),
    }
}

fn dispatch_error_to_adapter_error(
    error: EditorBindingDispatchError,
    envelope: &UiComponentEventEnvelope,
) -> UiComponentAdapterError {
    match error {
        EditorBindingDispatchError::UnsupportedInspectorField(_) => {
            UiComponentAdapterError::UnsupportedTargetPath {
                domain: INSPECTOR_DOMAIN.to_string(),
                path: envelope.target.path.clone(),
            }
        }
        other => UiComponentAdapterError::HostMutation {
            domain: INSPECTOR_DOMAIN.to_string(),
            path: envelope.target.path.clone(),
            reason: other.to_string(),
        },
    }
}
