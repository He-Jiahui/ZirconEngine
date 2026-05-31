use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventDescriptor, SoundDynamicEventHandlerDescriptor,
    SoundError,
};

use crate::dynamic_events::catalog::{register_dynamic_event, unregister_dynamic_event};
use crate::engine::SoundDynamicEventExecutorKey;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn dynamic_event_catalog_impl(
        &self,
    ) -> Result<SoundDynamicEventCatalog, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .dynamic_events
            .clone())
    }

    pub(in crate::service_types) fn register_dynamic_event_impl(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        register_dynamic_event(&mut state.dynamic_events, descriptor)
    }

    pub(in crate::service_types) fn unregister_dynamic_event_impl(
        &self,
        event_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        unregister_dynamic_event(&mut state.dynamic_events, event_id)?;
        state
            .dynamic_event_handlers
            .retain(|handler| handler.event_id != event_id);
        let handlers = state.dynamic_event_handlers.clone();
        state
            .dynamic_event_executors
            .retain(|key, _| handler_exists(&handlers, key));
        state
            .pending_dynamic_events
            .retain(|event| event.event_id != event_id);
        Ok(())
    }
}

fn handler_exists(
    handlers: &[SoundDynamicEventHandlerDescriptor],
    key: &SoundDynamicEventExecutorKey,
) -> bool {
    handlers
        .iter()
        .any(|handler| handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id)
}
