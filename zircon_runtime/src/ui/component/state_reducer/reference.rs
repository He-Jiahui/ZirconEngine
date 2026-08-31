use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiDragPayload,
    UiDragPayloadKind, UiDragSourceMetadata, UiValidationState, UiValue,
};

pub(super) fn drop_reference(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    payload: UiDragPayload,
) -> Result<(), UiComponentEventError> {
    if !descriptor.accepts_drag_payload(payload.kind) {
        state.validation = UiValidationState::error(format!(
            "rejected drop payload `{}` for {}",
            payload.kind.as_str(),
            descriptor.id
        ));
        return Err(UiComponentEventError::RejectedDrop {
            component_id: descriptor.id.clone(),
            payload_kind: payload.kind.as_str().to_string(),
        });
    }

    let (kind, reference, source) = into_reference_parts(payload);
    let value = match kind {
        UiDragPayloadKind::Asset => UiValue::AssetRef(reference),
        UiDragPayloadKind::SceneInstance | UiDragPayloadKind::Object => {
            UiValue::InstanceRef(reference)
        }
    };
    if let Some(source) = source {
        state.reference_sources.insert(property.clone(), source);
    } else {
        state.reference_sources.remove(&property);
    }
    state.values.insert(property, value);
    Ok(())
}

fn into_reference_parts(
    payload: UiDragPayload,
) -> (UiDragPayloadKind, String, Option<UiDragSourceMetadata>) {
    (payload.kind, payload.reference, payload.source)
}

pub(super) fn clear_reference(state: &mut UiComponentState, property: String) {
    state.reference_sources.remove(&property);
    state.values.insert(property, UiValue::Null);
}

pub(super) fn ensure_reference_value(
    state: &mut UiComponentState,
    property: String,
) -> Result<(), UiComponentEventError> {
    match state.values.get(&property) {
        Some(UiValue::AssetRef(reference)) | Some(UiValue::InstanceRef(reference))
            if !reference.is_empty() =>
        {
            Ok(())
        }
        _ => {
            state.validation =
                UiValidationState::error(format!("reference property `{property}` is empty"));
            Err(UiComponentEventError::MissingReference { property })
        }
    }
}

#[cfg(test)]
#[path = "reference/owned_drag_payload_tests.rs"]
mod owned_drag_payload_tests;
