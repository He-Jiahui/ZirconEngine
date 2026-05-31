use zircon_runtime::core::framework::sound::SoundOutputDeviceStatus;

use super::super::status::{latency_status_for_descriptor, push_status_diagnostic};
#[cfg(feature = "cpal-backend")]
use super::session::SoundOutputBackendSession;
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
        self.finalize_status(self.status_with_backend_session(status))
    }

    #[cfg(feature = "cpal-backend")]
    fn status_with_backend_session(
        &self,
        mut status: SoundOutputDeviceStatus,
    ) -> SoundOutputDeviceStatus {
        if let SoundOutputBackendSession::Cpal(session) = &self.backend_session {
            let cpal_status = session.status();
            status.rendered_blocks = status
                .rendered_blocks
                .saturating_add(cpal_status.rendered_blocks);
            status.rendered_frames = status
                .rendered_frames
                .saturating_add(cpal_status.rendered_frames);
            status.callback_count = status
                .callback_count
                .saturating_add(cpal_status.callback_count);
            status.last_callback_sequence = cpal_status
                .last_callback_sequence
                .or(status.last_callback_sequence);
            status.underrun_count = status
                .underrun_count
                .saturating_add(cpal_status.underrun_count);
            status.latency.queued_samples = cpal_status.queued_samples;
            status.latency.capacity_samples = cpal_status.capacity_samples;
            status.last_error = cpal_status.last_error.or(status.last_error);
        }
        status
    }

    #[cfg(not(feature = "cpal-backend"))]
    fn status_with_backend_session(
        &self,
        status: SoundOutputDeviceStatus,
    ) -> SoundOutputDeviceStatus {
        status
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
