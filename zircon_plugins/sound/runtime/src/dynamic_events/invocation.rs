use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventInvocation, SoundError,
};

pub(crate) fn submit_dynamic_event(
    catalog: &SoundDynamicEventCatalog,
    pending: &mut Vec<SoundDynamicEventInvocation>,
    invocation: SoundDynamicEventInvocation,
) -> Result<(), SoundError> {
    validate_dynamic_event_invocation(catalog, &invocation)?;
    pending.push(invocation);
    Ok(())
}

fn validate_dynamic_event_invocation(
    catalog: &SoundDynamicEventCatalog,
    invocation: &SoundDynamicEventInvocation,
) -> Result<(), SoundError> {
    if invocation.event_id.trim().is_empty() || invocation.payload_schema.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "dynamic event invocation requires event id and payload schema".to_string(),
        ));
    }
    if !invocation.time_seconds.is_finite() {
        return Err(SoundError::InvalidParameter(
            "dynamic event time must be finite".to_string(),
        ));
    }
    let descriptor = catalog
        .events
        .iter()
        .find(|event| event.id == invocation.event_id)
        .ok_or_else(|| SoundError::UnknownDynamicEvent {
            event_id: invocation.event_id.clone(),
        })?;
    if descriptor.payload_schema != invocation.payload_schema {
        return Err(SoundError::InvalidParameter(format!(
            "dynamic event {} expects payload schema {}",
            descriptor.id, descriptor.payload_schema
        )));
    }
    Ok(())
}
