use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::super::DefaultSoundManager;
use crate::output::{validate_backend_supported, validate_output_device_descriptor};
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn configure_output_device_impl(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        let mut config = lock_recover(&self.config);
        let mut state = lock_recover(&self.state);
        state.deactivate_kira();
        state.output_device.configure(descriptor.clone())?;
        state.update_graph_format(
            descriptor.sample_rate_hz,
            descriptor.channel_count,
            descriptor.channel_layout.clone(),
        );
        state.hrtf_states.clear();
        config.backend = descriptor.backend.clone();
        config.sample_rate_hz = descriptor.sample_rate_hz;
        config.channel_count = descriptor.channel_count;
        config.channel_layout = descriptor.channel_layout.clone();
        config.block_size_frames = descriptor.block_size_frames;
        Ok(())
    }
}
