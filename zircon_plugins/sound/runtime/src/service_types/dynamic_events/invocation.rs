use zircon_runtime::core::framework::sound::{SoundDynamicEventInvocation, SoundError};

use crate::dynamic_events::invocation::submit_dynamic_event;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn submit_dynamic_event_impl(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let catalog = state.dynamic_events.clone();
        submit_dynamic_event(&catalog, &mut state.pending_dynamic_events, invocation)
    }

    pub(in crate::service_types) fn drain_dynamic_events_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        Ok(state.pending_dynamic_events.drain(..).collect())
    }
}
