use super::super::{
    SoundBackendCallbackBlock, SoundBackendCapability, SoundError, SoundMixBlock,
    SoundOutputDeviceDescriptor, SoundOutputDeviceInfo, SoundOutputDeviceStatus,
};

pub trait SoundOutputDeviceManager {
    fn configure_output_device(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError>;
    fn start_output_device(&self) -> Result<(), SoundError>;
    fn stop_output_device(&self) -> Result<(), SoundError>;
    fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError>;
    fn available_output_devices(&self) -> Result<Vec<SoundOutputDeviceInfo>, SoundError>;
    fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError>;
    fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError>;
    fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError>;
}
