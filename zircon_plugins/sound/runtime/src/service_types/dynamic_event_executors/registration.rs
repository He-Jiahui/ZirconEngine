use zircon_runtime::core::framework::sound::{SoundDynamicEventDelivery, SoundError};

use crate::engine::{SoundDynamicEventExecutor, SoundDynamicEventExecutorKey};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub fn register_dynamic_event_executor<F>(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        executor: F,
    ) -> Result<(), SoundError>
    where
        F: Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static,
    {
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if !state.dynamic_event_handlers.iter().any(|handler| {
            handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id
        }) {
            return Err(SoundError::UnknownDynamicEventHandler {
                plugin_id: key.plugin_id,
                handler_id: key.handler_id,
            });
        }
        state
            .dynamic_event_executors
            .insert(key, SoundDynamicEventExecutor::new(executor));
        Ok(())
    }
}
