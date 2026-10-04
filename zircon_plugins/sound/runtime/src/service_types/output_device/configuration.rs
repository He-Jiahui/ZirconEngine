//! 配置请求先在锁外完成精确设备格式准入；成功后按既有契约停机重配并更新声明。
use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::super::DefaultSoundManager;
use crate::kira_bridge::admit_output_descriptor;
use crate::output::{validate_backend_supported, validate_output_device_descriptor};
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn configure_output_device_impl(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        // CPAL identity lookup and supported-format enumeration stay outside both owner locks.
        admit_output_descriptor(&descriptor)?;
        self.configure_admitted_output_device_impl(descriptor)
    }

    fn configure_admitted_output_device_impl(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        let mut config = lock_recover(&self.config);
        #[cfg(test)]
        super::lifecycle::wait_for_owner_config_admission();
        let mut state = lock_recover(&self.state);
        // Exact CPAL admission happened before either owner lock. Provider
        // Drop is still performed by the retirement worker, so this fence
        // only observes or retains its ACK/census before publication.
        state.kira.quiesce_before_replace()?;
        state.output_device.configure_admitted(descriptor.clone());
        // `configure_output_device` intentionally stops on success. All fallible admission has
        // completed before this point, so a rejected request cannot retire the active generation.
        state.deactivate_kira();
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
        state.output_generation = state.output_generation.wrapping_add(1);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn configure_output_device_after_admission_for_test(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        self.configure_admitted_output_device_impl(descriptor)
    }
}
