use zircon_runtime::core::framework::sound::SoundOutputDeviceState;

use super::SoundOutputDeviceRuntimeState;

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn mark_started(&mut self) {
        self.state = SoundOutputDeviceState::Started;
        self.last_error = None;
        self.unavailable_backend = None;
        self.unavailable_detail = None;
    }

    pub(crate) fn stop(&mut self) {
        self.state = SoundOutputDeviceState::Stopped;
    }
}
