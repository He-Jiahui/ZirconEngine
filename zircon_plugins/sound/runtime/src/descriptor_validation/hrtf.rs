use zircon_runtime::core::framework::sound::{SoundError, SoundHrtfProfileDescriptor};

pub(crate) fn validate_hrtf_profile_descriptor(
    profile: &SoundHrtfProfileDescriptor,
) -> Result<(), SoundError> {
    if profile.profile_id.trim().is_empty() || profile.display_name.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "HRTF profile requires a non-empty id and display name".to_string(),
        ));
    }
    if profile.sample_rate_hz == 0 {
        return Err(SoundError::InvalidParameter(
            "HRTF profile sample rate must be non-zero".to_string(),
        ));
    }
    if profile.left_kernel.is_empty() || profile.right_kernel.is_empty() {
        return Err(SoundError::InvalidParameter(
            "HRTF profile kernels must be non-empty".to_string(),
        ));
    }
    if profile
        .left_kernel
        .iter()
        .chain(profile.right_kernel.iter())
        .any(|sample| !sample.is_finite())
    {
        return Err(SoundError::InvalidParameter(
            "HRTF profile kernel samples must be finite".to_string(),
        ));
    }
    if !profile
        .left_kernel
        .iter()
        .chain(profile.right_kernel.iter())
        .any(|sample| *sample != 0.0)
    {
        return Err(SoundError::InvalidParameter(
            "HRTF profile requires at least one non-zero kernel sample".to_string(),
        ));
    }
    Ok(())
}
