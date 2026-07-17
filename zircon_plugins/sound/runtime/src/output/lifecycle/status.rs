use zircon_runtime::core::framework::sound::SoundOutputDeviceStatus;

use super::super::status::{latency_status_for_descriptor, push_status_diagnostic};
use super::SoundOutputDeviceRuntimeState;

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn status(&self) -> SoundOutputDeviceStatus {
        let status = SoundOutputDeviceStatus {
            descriptor: self.descriptor.clone(),
            state: self.state,
            latency: latency_status_for_descriptor(&self.descriptor, None, None),
            rendered_blocks: self.rendered_blocks,
            rendered_frames: self.rendered_frames,
            callback_count: self.callback_count,
            last_callback_sequence: self.last_callback_sequence,
            underrun_count: self.underrun_count,
            last_error: self.last_error.clone(),
            diagnostics: Vec::new(),
        };
        self.finalize_status(status)
    }

    fn finalize_status(&self, mut status: SoundOutputDeviceStatus) -> SoundOutputDeviceStatus {
        if let Some((_, detail)) = self.unavailable_backend_status() {
            push_status_diagnostic(&mut status, format!("sound backend unavailable: {detail}"));
        }
        if let Some(last_error) = status.last_error.clone() {
            push_status_diagnostic(&mut status, last_error);
        }
        status
    }
}
