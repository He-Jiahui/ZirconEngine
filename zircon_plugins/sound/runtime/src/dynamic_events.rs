use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventDescriptor, SoundDynamicEventInvocation, SoundError,
};

pub(crate) fn validate_dynamic_event_catalog(
    catalog: &SoundDynamicEventCatalog,
) -> Result<(), SoundError> {
    if catalog.namespace.trim().is_empty() || catalog.version == 0 {
        return Err(SoundError::InvalidParameter(
            "dynamic event catalog requires a namespace and non-zero version".to_string(),
        ));
    }

    let mut event_ids = HashSet::new();
    for descriptor in &catalog.events {
        validate_dynamic_event_descriptor(descriptor)?;
        if !event_ids.insert(descriptor.id.as_str()) {
            return Err(SoundError::InvalidParameter(format!(
                "duplicate dynamic event id {}",
                descriptor.id
            )));
        }
    }
    Ok(())
}

pub(crate) fn register_dynamic_event(
    catalog: &mut SoundDynamicEventCatalog,
    descriptor: SoundDynamicEventDescriptor,
) -> Result<(), SoundError> {
    validate_dynamic_event_descriptor(&descriptor)?;
    if let Some(existing) = catalog
        .events
        .iter_mut()
        .find(|event| event.id == descriptor.id)
    {
        *existing = descriptor;
    } else {
        catalog.events.push(descriptor);
    }
    validate_dynamic_event_catalog(catalog)
}

pub(crate) fn unregister_dynamic_event(
    catalog: &mut SoundDynamicEventCatalog,
    event_id: &str,
) -> Result<(), SoundError> {
    let before = catalog.events.len();
    catalog.events.retain(|event| event.id != event_id);
    if before == catalog.events.len() {
        return Err(SoundError::UnknownDynamicEvent {
            event_id: event_id.to_string(),
        });
    }
    Ok(())
}

pub(crate) fn submit_dynamic_event(
    catalog: &SoundDynamicEventCatalog,
    pending: &mut Vec<SoundDynamicEventInvocation>,
    invocation: SoundDynamicEventInvocation,
) -> Result<(), SoundError> {
    validate_dynamic_event_invocation(catalog, &invocation)?;
    pending.push(invocation);
    Ok(())
}

fn validate_dynamic_event_descriptor(
    descriptor: &SoundDynamicEventDescriptor,
) -> Result<(), SoundError> {
    if descriptor.id.trim().is_empty()
        || descriptor.display_name.trim().is_empty()
        || descriptor.payload_schema.trim().is_empty()
    {
        return Err(SoundError::InvalidParameter(
            "dynamic event descriptor requires id, display name, and payload schema".to_string(),
        ));
    }
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
