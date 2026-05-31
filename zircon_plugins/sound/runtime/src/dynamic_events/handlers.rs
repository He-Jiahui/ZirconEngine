use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventHandlerDescriptor, SoundError,
};

pub(crate) fn register_dynamic_event_handler(
    catalog: &SoundDynamicEventCatalog,
    handlers: &mut Vec<SoundDynamicEventHandlerDescriptor>,
    handler: SoundDynamicEventHandlerDescriptor,
) -> Result<(), SoundError> {
    validate_dynamic_event_handler(catalog, &handler)?;
    if let Some(existing) = handlers.iter_mut().find(|existing| {
        existing.plugin_id == handler.plugin_id && existing.handler_id == handler.handler_id
    }) {
        *existing = handler;
    } else {
        handlers.push(handler);
    }
    Ok(())
}

pub(crate) fn unregister_dynamic_event_handler(
    handlers: &mut Vec<SoundDynamicEventHandlerDescriptor>,
    plugin_id: &str,
    handler_id: &str,
) -> Result<(), SoundError> {
    let before = handlers.len();
    handlers.retain(|handler| handler.plugin_id != plugin_id || handler.handler_id != handler_id);
    if before == handlers.len() {
        return Err(SoundError::UnknownDynamicEventHandler {
            plugin_id: plugin_id.to_string(),
            handler_id: handler_id.to_string(),
        });
    }
    Ok(())
}

fn validate_dynamic_event_handler(
    catalog: &SoundDynamicEventCatalog,
    handler: &SoundDynamicEventHandlerDescriptor,
) -> Result<(), SoundError> {
    if handler.plugin_id.trim().is_empty()
        || handler.handler_id.trim().is_empty()
        || handler.event_id.trim().is_empty()
        || handler.display_name.trim().is_empty()
    {
        return Err(SoundError::InvalidParameter(
            "dynamic event handler requires plugin id, handler id, event id, and display name"
                .to_string(),
        ));
    }
    if !catalog
        .events
        .iter()
        .any(|event| event.id == handler.event_id)
    {
        return Err(SoundError::UnknownDynamicEvent {
            event_id: handler.event_id.clone(),
        });
    }
    Ok(())
}
