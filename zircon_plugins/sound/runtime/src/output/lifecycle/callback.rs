use zircon_runtime::core::framework::sound::{SoundBackendCallbackReport, SoundError};

use super::SoundOutputDeviceRuntimeState;

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn record_rendered_block(&mut self, frames: usize, sample_count: usize) {
        self.rendered_blocks = self.rendered_blocks.saturating_add(1);
        self.rendered_frames = self.rendered_frames.saturating_add(frames as u64);
        let expected_samples = frames.saturating_mul(self.descriptor.channel_count as usize);
        if sample_count != expected_samples {
            self.underrun_count = self.underrun_count.saturating_add(1);
        }
        self.last_error = None;
    }

    pub(crate) fn record_error(&mut self, error: &SoundError) {
        self.underrun_count = self.underrun_count.saturating_add(1);
        self.last_error = Some(error.to_string());
    }

    pub(crate) fn record_backend_unavailable(
        &mut self,
        backend: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.clear_backend_session();
        let detail = detail.into();
        self.unavailable_backend = Some(backend.into());
        self.unavailable_detail = Some(detail.clone());
        self.state = zircon_runtime::core::framework::sound::SoundOutputDeviceState::Stopped;
        self.last_error = Some(format!("sound backend unavailable: {detail}"));
    }

    pub(crate) fn unavailable_backend_status(&self) -> Option<(&str, &str)> {
        Some((
            self.unavailable_backend.as_deref()?,
            self.unavailable_detail.as_deref()?,
        ))
    }

    pub(crate) fn unavailable_backend_error(&self) -> Option<SoundError> {
        let (_, detail) = self.unavailable_backend_status()?;
        Some(SoundError::BackendUnavailable {
            detail: detail.to_string(),
        })
    }

    pub(crate) fn record_callback_block(
        &mut self,
        requested_frames: usize,
        rendered_frames: usize,
        sample_count: usize,
    ) -> SoundBackendCallbackReport {
        let sequence_index = self.next_callback_sequence;
        self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
        self.callback_count = self.callback_count.saturating_add(1);
        self.last_callback_sequence = Some(sequence_index);
        self.record_rendered_block(rendered_frames, sample_count);
        let expected_samples =
            requested_frames.saturating_mul(self.descriptor.channel_count as usize);
        SoundBackendCallbackReport {
            device: self.descriptor.id.clone(),
            backend: self.descriptor.backend.clone(),
            sequence_index,
            requested_frames,
            rendered_frames,
            sample_count,
            underrun: rendered_frames != requested_frames || sample_count != expected_samples,
            error: None,
        }
    }

    pub(crate) fn record_callback_error(
        &mut self,
        requested_frames: usize,
        error: &SoundError,
    ) -> SoundBackendCallbackReport {
        let sequence_index = self.next_callback_sequence;
        self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
        self.callback_count = self.callback_count.saturating_add(1);
        self.last_callback_sequence = Some(sequence_index);
        self.record_error(error);
        SoundBackendCallbackReport {
            device: self.descriptor.id.clone(),
            backend: self.descriptor.backend.clone(),
            sequence_index,
            requested_frames,
            rendered_frames: 0,
            sample_count: 0,
            underrun: true,
            error: Some(error.to_string()),
        }
    }
}
