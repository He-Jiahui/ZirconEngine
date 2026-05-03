use zircon_runtime::core::framework::sound::{
    SoundError, SoundOutputDeviceDescriptor, SoundOutputDeviceState, SoundOutputDeviceStatus,
};

use crate::SoundConfig;

#[derive(Clone, Debug)]
pub(crate) struct SoundOutputDeviceRuntimeState {
    descriptor: SoundOutputDeviceDescriptor,
    state: SoundOutputDeviceState,
    rendered_blocks: u64,
    rendered_frames: u64,
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
            underrun_count: 0,
            last_error: None,
        }
    }

    pub(crate) fn configure(
        &mut self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        self.descriptor = descriptor;
        self.state = SoundOutputDeviceState::Stopped;
        self.rendered_blocks = 0;
        self.rendered_frames = 0;
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

    pub(crate) fn status(&self) -> SoundOutputDeviceStatus {
        SoundOutputDeviceStatus {
            descriptor: self.descriptor.clone(),
            state: self.state,
            rendered_blocks: self.rendered_blocks,
            rendered_frames: self.rendered_frames,
            underrun_count: self.underrun_count,
            last_error: self.last_error.clone(),
        }
    }
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
