use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::super::descriptor_validation::{
    validate_backend_supported, validate_output_device_descriptor,
};
use super::SoundOutputDeviceRuntimeState;

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn configure(
        &mut self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        self.descriptor = descriptor;
        self.state = zircon_runtime::core::framework::sound::SoundOutputDeviceState::Stopped;
        self.rendered_blocks = 0;
        self.rendered_frames = 0;
        self.callback_count = 0;
        self.last_callback_sequence = None;
        self.next_callback_sequence = 0;
        self.underrun_count = 0;
        self.last_error = None;
        self.unavailable_backend = None;
        self.unavailable_detail = None;
        Ok(())
    }

    /// Apply a descriptor that already passed backend/device admission. Keeping this mutation
    /// infallible lets the owner quiesce the old callback before changing the published device;
    /// a quiescence timeout therefore leaves both the descriptor and last-good stream intact.
    pub(crate) fn configure_admitted(&mut self, descriptor: SoundOutputDeviceDescriptor) {
        self.descriptor = descriptor;
        self.state = zircon_runtime::core::framework::sound::SoundOutputDeviceState::Stopped;
        self.rendered_blocks = 0;
        self.rendered_frames = 0;
        self.callback_count = 0;
        self.last_callback_sequence = None;
        self.next_callback_sequence = 0;
        self.underrun_count = 0;
        self.last_error = None;
        self.unavailable_backend = None;
        self.unavailable_detail = None;
    }
}
