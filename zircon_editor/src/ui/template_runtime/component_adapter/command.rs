use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEvent, UiComponentEventEnvelope,
    UiValue, UiValueKind,
};

pub(crate) const COMMAND_DOMAIN: &str = "command";
const COMMITTED_COMMAND_ID: &str = "committed_command_id";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const COMMAND_ID: &str = "command_id";

pub(crate) fn editor_command_binding_for_envelope(
    envelope: &UiComponentEventEnvelope,
) -> Result<EditorUiBinding, UiComponentAdapterError> {
    if envelope.target.domain != COMMAND_DOMAIN {
        return Err(UiComponentAdapterError::UnsupportedTargetDomain {
            domain: envelope.target.domain.clone(),
        });
    }

    let command_id = command_id_from_event(envelope)?;
    Ok(EditorUiBinding::new(
        &envelope.document_id,
        &envelope.control_id,
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command(command_id),
    ))
}

pub(crate) fn command_adapter_result(command_id: &str) -> UiComponentAdapterResult {
    UiComponentAdapterResult::changed()
        .dirty(false)
        .with_mutation_source(COMMAND_DOMAIN)
        .with_transaction(format!("command:{command_id}"))
        .with_status(format!("Executed editor command `{command_id}`"))
}

fn command_id_from_event(
    envelope: &UiComponentEventEnvelope,
) -> Result<String, UiComponentAdapterError> {
    match &envelope.event {
        UiComponentEvent::Commit { property, value } => {
            validate_command_property(envelope, property)?;
            let command_id =
                string_value(value).ok_or_else(|| UiComponentAdapterError::InvalidValueKind {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                    expected: UiValueKind::String,
                    actual: value.kind(),
                })?;
            if command_id.is_empty() {
                return Err(UiComponentAdapterError::RejectedInput {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                    reason: "command commit requires a non-empty command id".to_string(),
                });
            }
            Ok(command_id.to_string())
        }
        event => Err(UiComponentAdapterError::UnsupportedEvent {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            event_kind: event.kind(),
        }),
    }
}

fn validate_command_property(
    envelope: &UiComponentEventEnvelope,
    property: &str,
) -> Result<(), UiComponentAdapterError> {
    if matches!(
        property,
        COMMITTED_COMMAND_ID | SELECTED_COMMAND_ID | COMMAND_ID
    ) {
        return Ok(());
    }

    Err(UiComponentAdapterError::UnsupportedTargetPath {
        domain: envelope.target.domain.clone(),
        path: envelope.target.path.clone(),
    })
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) => Some(value.trim()),
        _ => None,
    }
}
