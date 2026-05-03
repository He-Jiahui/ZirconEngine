use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackReport, SoundBackendCapability, SoundError, SoundOutputDeviceDescriptor,
    SoundOutputDeviceState, SoundOutputDeviceStatus,
};

use crate::SoundConfig;

pub(crate) const SOFTWARE_NULL_BACKEND: &str = "software-null";

#[derive(Clone, Debug)]
pub(crate) struct SoundOutputDeviceRuntimeState {
    descriptor: SoundOutputDeviceDescriptor,
    state: SoundOutputDeviceState,
    rendered_blocks: u64,
    rendered_frames: u64,
    callback_count: u64,
    last_callback_sequence: Option<u64>,
    next_callback_sequence: u64,
    underrun_count: u64,
    last_error: Option<String>,
}

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn new(config: &SoundConfig) -> Self {
        Self {
            descriptor: SoundOutputDeviceDescriptor::software(
                config.backend.clone(),
                config.sample_rate_hz,
                config.channel_count,
                config.block_size_frames,
            ),
            state: SoundOutputDeviceState::Stopped,
            rendered_blocks: 0,
            rendered_frames: 0,
            callback_count: 0,
            last_callback_sequence: None,
            next_callback_sequence: 0,
            underrun_count: 0,
            last_error: None,
        }
    }

    pub(crate) fn configure(
        &mut self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        self.descriptor = descriptor;
        self.state = SoundOutputDeviceState::Stopped;
        self.rendered_blocks = 0;
        self.rendered_frames = 0;
        self.callback_count = 0;
        self.last_callback_sequence = None;
        self.next_callback_sequence = 0;
        self.underrun_count = 0;
        self.last_error = None;
        Ok(())
    }

    pub(crate) fn start(&mut self) {
        self.state = SoundOutputDeviceState::Started;
        self.last_error = None;
    }

    pub(crate) fn stop(&mut self) {
        self.state = SoundOutputDeviceState::Stopped;
    }

    pub(crate) fn block_size_frames(&self) -> Result<usize, SoundError> {
        if self.state != SoundOutputDeviceState::Started {
            return Err(SoundError::BackendUnavailable {
                detail: "sound output device is stopped".to_string(),
            });
        }
        Ok(self.descriptor.block_size_frames)
    }

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

    pub(crate) fn status(&self) -> SoundOutputDeviceStatus {
        SoundOutputDeviceStatus {
            descriptor: self.descriptor.clone(),
            state: self.state,
            rendered_blocks: self.rendered_blocks,
            rendered_frames: self.rendered_frames,
            callback_count: self.callback_count,
            last_callback_sequence: self.last_callback_sequence,
            underrun_count: self.underrun_count,
            last_error: self.last_error.clone(),
        }
    }
}

pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: SOFTWARE_NULL_BACKEND.to_string(),
        display_name: "Deterministic Software Null Output".to_string(),
        realtime_capable: false,
        deterministic: true,
        min_sample_rate_hz: 1,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "headless backend for tests and editor preview".to_string(),
            "pulls blocks from the software mixer without opening an OS device".to_string(),
        ],
    }]
}

fn validate_backend_supported(descriptor: &SoundOutputDeviceDescriptor) -> Result<(), SoundError> {
    if descriptor.backend == SOFTWARE_NULL_BACKEND || descriptor.backend.starts_with("software-") {
        return Ok(());
    }
    Err(SoundError::BackendUnavailable {
        detail: format!(
            "sound output backend `{}` is not available",
            descriptor.backend
        ),
    })
}

pub(crate) fn validate_output_device_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<(), SoundError> {
    if descriptor.id.as_str().trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output device id must be non-empty".to_string(),
        ));
    }
    if descriptor.backend.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output backend must be non-empty".to_string(),
        ));
    }
    if descriptor.display_name.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output display name must be non-empty".to_string(),
        ));
    }
    if descriptor.sample_rate_hz == 0 {
        return Err(SoundError::InvalidParameter(
            "output sample rate must be non-zero".to_string(),
        ));
    }
    if descriptor.channel_count == 0 {
        return Err(SoundError::InvalidParameter(
            "output channel count must be non-zero".to_string(),
        ));
    }
    if descriptor.block_size_frames == 0 {
        return Err(SoundError::InvalidParameter(
            "output block size must be non-zero".to_string(),
        ));
    }
    if descriptor.latency_blocks == 0 {
        return Err(SoundError::InvalidParameter(
            "output latency blocks must be non-zero".to_string(),
        ));
    }
    Ok(())
}
