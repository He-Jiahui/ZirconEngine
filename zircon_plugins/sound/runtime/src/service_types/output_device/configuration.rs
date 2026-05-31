use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn configure_output_device_impl(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        let mut config = self.config.lock().expect("sound config mutex poisoned");
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Err(error) = state.output_device.configure(descriptor.clone()) {
            if let SoundError::BackendUnavailable { detail } = &error {
                state
                    .output_device
                    .record_backend_unavailable(descriptor.backend, detail.clone());
            }
            return Err(error);
        }
        config.backend = descriptor.backend.clone();
        config.sample_rate_hz = descriptor.sample_rate_hz;
        config.channel_count = descriptor.channel_count;
        config.block_size_frames = descriptor.block_size_frames;
        state.graph.sample_rate_hz = config.sample_rate_hz;
        state.graph.channel_count = config.channel_count;
        state.effect_states.clear();
        state.track_states.clear();
        state.hrtf_states.clear();
        Ok(())
    }
}
