use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock,
};

pub(crate) fn validate_external_source_handle(
    handle: &ExternalAudioSourceHandle,
) -> Result<(), SoundError> {
    if handle.as_str().trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "external source handle must be non-empty".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_external_source_block(
    block: &SoundExternalSourceBlock,
) -> Result<(), SoundError> {
    if block.sample_rate_hz == 0 || block.channel_count == 0 {
        return Err(SoundError::InvalidParameter(
            "external source block sample rate and channel count must be positive".to_string(),
        ));
    }
    if block.samples.iter().any(|sample| !sample.is_finite()) {
        return Err(SoundError::InvalidParameter(
            "external source block samples must be finite".to_string(),
        ));
    }
    if block.samples.len() % block.channel_count as usize != 0 {
        return Err(SoundError::InvalidParameter(
            "external source block samples must contain whole frames".to_string(),
        ));
    }
    Ok(())
}
