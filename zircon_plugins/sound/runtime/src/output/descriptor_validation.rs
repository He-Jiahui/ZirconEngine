use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::{cpal, software};

pub(super) fn validate_backend_supported(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<(), SoundError> {
    if software::supports_software_backend(&descriptor.backend) {
        return Ok(());
    }
    if descriptor.backend == cpal::CPAL_BACKEND {
        return cpal::validate_cpal_backend_supported();
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
