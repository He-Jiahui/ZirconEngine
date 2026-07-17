use zircon_runtime::core::framework::sound::SoundError;

use crate::engine::SoundDynamicEventExecutorKey;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub fn unregister_dynamic_event_executor(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        state
            .dynamic_event_executors
            .remove(&key)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownDynamicEventHandler {
                plugin_id: plugin_id.to_string(),
                handler_id: handler_id.to_string(),
            })
    }
}
