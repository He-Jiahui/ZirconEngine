use zircon_runtime::core::framework::sound::{SoundBackendCapability, SoundError};

#[cfg(feature = "cpal-backend")]
use super::CPAL_BACKEND;

#[cfg(feature = "cpal-backend")]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: CPAL_BACKEND.to_string(),
        display_name: "CPAL Default Output".to_string(),
        realtime_capable: true,
        deterministic: false,
        min_sample_rate_hz: 8_000,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "uses the platform default output device through CPAL".to_string(),
            "availability depends on host audio devices and OS permissions".to_string(),
        ],
    }]
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    Vec::new()
}

#[cfg(feature = "cpal-backend")]
pub(crate) fn validate_cpal_backend_supported() -> Result<(), SoundError> {
    Ok(())
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn validate_cpal_backend_supported() -> Result<(), SoundError> {
    Err(SoundError::BackendUnavailable {
        detail: super::error::cpal_backend_unavailable_detail(),
    })
}
