use zircon_runtime_interface::ZrPluginEventCallbackFnV1;

use crate::service_types::DefaultSoundManager;

use super::executor::sound_dynamic_event_callback_executor;

impl DefaultSoundManager {
    pub fn register_dynamic_event_abi_callback(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        callback: ZrPluginEventCallbackFnV1,
    ) -> Result<(), zircon_runtime::core::framework::sound::SoundError> {
        self.register_dynamic_event_executor(
            plugin_id,
            handler_id,
            sound_dynamic_event_callback_executor(callback),
        )
    }
}
