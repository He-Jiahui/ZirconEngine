use zircon_runtime::core::framework::sound::{SoundDynamicEventHandlerDescriptor, SoundError};

use crate::dynamic_events::handlers::{
    register_dynamic_event_handler, unregister_dynamic_event_handler,
};
use crate::engine::SoundDynamicEventExecutorKey;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn dynamic_event_handlers_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
        Ok(crate::poison_recovery::lock_recover(&self.state)
            .dynamic_event_handlers
            .handlers()
            .to_vec())
    }

    pub(in crate::service_types) fn register_dynamic_event_handler_impl(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        let catalog = state.dynamic_events.clone();
        register_dynamic_event_handler(&catalog, &mut state.dynamic_event_handlers, handler)
    }

    pub(in crate::service_types) fn unregister_dynamic_event_handler_impl(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        unregister_dynamic_event_handler(&mut state.dynamic_event_handlers, plugin_id, handler_id)?;
        state
            .dynamic_event_executors
            .remove(&SoundDynamicEventExecutorKey::new(plugin_id, handler_id));
        Ok(())
    }
}
