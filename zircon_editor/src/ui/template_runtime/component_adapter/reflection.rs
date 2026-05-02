use crate::ui::binding_dispatch::{apply_inspector_draft_field, EditorBindingDispatchError};
use crate::ui::workbench::state::EditorState;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentBindingTarget, UiComponentEvent,
    UiComponentEventEnvelope, UiComponentProjectionPatch, UiValue, UiValueKind,
};

const REFLECTION_DOMAIN: &str = "reflection";
const SELECTED_COMPONENT_SUBJECT: &str = "component://selected";
const SELECTED_ENTITY_SUBJECT: &str = "entity://selected";
const INSPECTOR_SELECTED_ENTITY_SUBJECT: &str = "entity://selected";
const VALUE_PROPERTY: &str = "value";

pub(crate) fn apply_reflection_component_envelope(
    state: &mut EditorState,
    envelope: &UiComponentEventEnvelope,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    if envelope.target.domain != REFLECTION_DOMAIN {
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

    match subject {
        SELECTED_COMPONENT_SUBJECT | SELECTED_ENTITY_SUBJECT => {
            apply_selected_entity_reflection_envelope(state, envelope)
        }
        _ => Err(UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: format!("unsupported reflection subject {subject}"),
        }),
    }
}

fn apply_selected_entity_reflection_envelope(
    state: &mut EditorState,
    envelope: &UiComponentEventEnvelope,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    let (property, value) = match &envelope.event {
        UiComponentEvent::ValueChanged { property, value }
        | UiComponentEvent::Commit { property, value } => (property, value),
        event => {
            return Err(UiComponentAdapterError::UnsupportedEvent {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                event_kind: event.kind(),
            });
        }
    };
    if property != VALUE_PROPERTY {
        return Err(UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: format!("reflection adapter does not support property {property}"),
        });
    }

    if envelope.target.path == "transform.translation" {
        return apply_translation_vector(state, envelope, value);
    }

    let inspector_path = match envelope.target.path.as_str() {
        "name"
        | "parent"
        | "transform.translation.x"
        | "transform.translation.y"
        | "transform.translation.z" => envelope.target.path.as_str(),
        _ => {
            return Err(UiComponentAdapterError::UnsupportedTargetPath {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
            });
        }
    };

    let mut inspector_envelope = envelope.clone();
    inspector_envelope.target =
        UiComponentBindingTarget::inspector(INSPECTOR_SELECTED_ENTITY_SUBJECT, inspector_path);
    super::inspector::apply_inspector_component_envelope(state, &inspector_envelope)
        .map(|result| reflection_result(envelope, value, result))
}

fn apply_translation_vector(
    state: &mut EditorState,
    envelope: &UiComponentEventEnvelope,
    value: &UiValue,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    let UiValue::Vec3(translation) = value else {
        return Err(UiComponentAdapterError::InvalidValueKind {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            expected: UiValueKind::Vec3,
            actual: value.kind(),
        });
    };

    for (axis, value) in ["x", "y", "z"].into_iter().zip(translation.iter()) {
        apply_inspector_draft_field(
            state,
            INSPECTOR_SELECTED_ENTITY_SUBJECT,
            &format!("transform.translation.{axis}"),
            value.to_string(),
        )
        .map_err(|error| dispatch_error_to_adapter_error(error, envelope))?;
    }

    Ok(UiComponentAdapterResult::changed()
        .with_transaction("reflection:transform.translation")
        .with_mutation_source("reflection")
        .with_status("Reflection transform.translation updated via selected entity inspector")
        .with_patch(
            UiComponentProjectionPatch::new(envelope.control_id.clone())
                .with_state_value(envelope.target.path.clone(), value.clone())
                .with_attribute(VALUE_PROPERTY, value.clone()),
        ))
}

fn reflection_result(
    envelope: &UiComponentEventEnvelope,
    value: &UiValue,
    mut result: UiComponentAdapterResult,
) -> UiComponentAdapterResult {
    result.transaction_id = Some(format!("reflection:{}", envelope.target.path));
    result.mutation_source = Some("reflection".to_string());
    result.status_text = Some(format!(
        "Reflection {} updated via selected entity inspector",
        envelope.target.path
    ));
    result.patches = vec![UiComponentProjectionPatch::new(envelope.control_id.clone())
        .with_state_value(envelope.target.path.clone(), value.clone())
        .with_attribute(VALUE_PROPERTY, value.clone())];
    result
}

fn dispatch_error_to_adapter_error(
    error: EditorBindingDispatchError,
    envelope: &UiComponentEventEnvelope,
) -> UiComponentAdapterError {
    match error {
        EditorBindingDispatchError::UnsupportedInspectorField(_) => {
            UiComponentAdapterError::UnsupportedTargetPath {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
            }
        }
        other => UiComponentAdapterError::HostMutation {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: other.to_string(),
        },
    }
}
