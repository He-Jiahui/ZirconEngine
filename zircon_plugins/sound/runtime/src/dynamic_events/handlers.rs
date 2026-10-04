//! 处理器注册表按优先级与稳定身份排序，并建立事件到处理器索引；派发时无需反复排序全表。
use std::{cmp::Ordering, collections::HashMap};

use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventHandlerDescriptor, SoundError,
};

#[derive(Debug, Default)]
pub(crate) struct DynamicEventHandlerRegistry {
    handlers: Vec<SoundDynamicEventHandlerDescriptor>,
    indices_by_event: HashMap<String, Vec<usize>>,
}

impl DynamicEventHandlerRegistry {
    pub(crate) fn handlers(&self) -> &[SoundDynamicEventHandlerDescriptor] {
        &self.handlers
    }

    pub(crate) fn indices_for_event(&self, event_id: &str) -> Option<&[usize]> {
        self.indices_by_event.get(event_id).map(Vec::as_slice)
    }

    pub(crate) fn handler(&self, index: usize) -> &SoundDynamicEventHandlerDescriptor {
        &self.handlers[index]
    }

    pub(crate) fn retain(
        &mut self,
        mut keep: impl FnMut(&SoundDynamicEventHandlerDescriptor) -> bool,
    ) {
        self.handlers.retain(|handler| keep(handler));
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.indices_by_event.clear();
        for (index, handler) in self.handlers.iter().enumerate() {
            self.indices_by_event
                .entry(handler.event_id.clone())
                .or_default()
                .push(index);
        }
    }

    #[cfg(test)]
    pub(crate) fn from_handlers(mut handlers: Vec<SoundDynamicEventHandlerDescriptor>) -> Self {
        handlers.sort_by(dynamic_event_handler_dispatch_order);
        let mut registry = Self {
            handlers,
            indices_by_event: HashMap::new(),
        };
        registry.rebuild_index();
        registry
    }
}

pub(crate) fn register_dynamic_event_handler(
    catalog: &SoundDynamicEventCatalog,
    registry: &mut DynamicEventHandlerRegistry,
    handler: SoundDynamicEventHandlerDescriptor,
) -> Result<(), SoundError> {
    validate_dynamic_event_handler(catalog, &handler)?;
    if let Some(existing) = registry.handlers.iter_mut().find(|existing| {
        existing.plugin_id == handler.plugin_id && existing.handler_id == handler.handler_id
    }) {
        *existing = handler;
    } else {
        registry.handlers.push(handler);
    }
    registry
        .handlers
        .sort_by(dynamic_event_handler_dispatch_order);
    registry.rebuild_index();
    Ok(())
}

pub(crate) fn dynamic_event_handler_dispatch_order(
    left: &SoundDynamicEventHandlerDescriptor,
    right: &SoundDynamicEventHandlerDescriptor,
) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| left.plugin_id.cmp(&right.plugin_id))
        .then_with(|| left.handler_id.cmp(&right.handler_id))
}

pub(crate) fn unregister_dynamic_event_handler(
    registry: &mut DynamicEventHandlerRegistry,
    plugin_id: &str,
    handler_id: &str,
) -> Result<(), SoundError> {
    let before = registry.handlers.len();
    registry.retain(|handler| handler.plugin_id != plugin_id || handler.handler_id != handler_id);
    if before == registry.handlers.len() {
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

#[cfg(test)]
#[path = "tests/handlers.rs"]
mod tests;
