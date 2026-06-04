use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackBlock, SoundBackendCapability, SoundError, SoundMixBlock,
    SoundOutputDeviceDescriptor, SoundOutputDeviceInfo, SoundOutputDeviceManager,
    SoundOutputDeviceStatus,
};

use super::super::DefaultSoundManager;

impl SoundOutputDeviceManager for DefaultSoundManager {
    fn configure_output_device(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        self.configure_output_device_impl(descriptor)
    }

    fn start_output_device(&self) -> Result<(), SoundError> {
        self.start_output_device_impl()
    }

    fn stop_output_device(&self) -> Result<(), SoundError> {
        self.stop_output_device_impl()
    }

    fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError> {
        self.output_device_status_impl()
    }

    fn available_output_devices(&self) -> Result<Vec<SoundOutputDeviceInfo>, SoundError> {
        self.available_output_devices_impl()
    }

    fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError> {
        self.render_output_device_block_impl()
    }

    fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError> {
        self.available_output_backends_impl()
    }

    fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError> {
        self.pull_output_backend_callback_impl()
    }
}
