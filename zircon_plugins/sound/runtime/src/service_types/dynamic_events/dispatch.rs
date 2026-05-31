use zircon_runtime::core::framework::sound::{SoundDynamicEventDelivery, SoundError};

use crate::dynamic_events::dispatch::dispatch_dynamic_events;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn dispatch_dynamic_events_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let handlers = state.dynamic_event_handlers.clone();
        Ok(dispatch_dynamic_events(
            &handlers,
            &mut state.pending_dynamic_events,
        ))
    }
}
