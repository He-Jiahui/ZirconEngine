use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use crate::kira_bridge::KIRA_CPAL_BACKEND;

pub(crate) fn validate_backend_supported(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<(), SoundError> {
    if descriptor.backend == KIRA_CPAL_BACKEND {
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
    if !descriptor
        .channel_layout
        .matches_channel_count(descriptor.channel_count)
    {
        return Err(SoundError::InvalidParameter(format!(
            "output channel layout `{}` does not match channel count {}",
            descriptor.channel_layout.name, descriptor.channel_count
        )));
    }
    if !descriptor.channel_layout.is_valid_contract_layout() {
        return Err(SoundError::InvalidParameter(format!(
            "output channel layout `{}` must use canonical speaker metadata",
            descriptor.channel_layout.name
        )));
    }
    if descriptor.channel_count > 2 {
        return Err(SoundError::UnsupportedAdvancedFeature(format!(
            "{}-channel sound output is enabled after Kira adds multichannel backend support",
            descriptor.channel_count
        )));
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
