use zircon_runtime::core::framework::sound::SoundError;

pub(super) fn validate_optional_seconds(
    label: &str,
    seconds: Option<f32>,
) -> Result<(), SoundError> {
    let Some(seconds) = seconds else {
        return Ok(());
    };
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(SoundError::InvalidParameter(format!(
            "{label} must be finite and non-negative"
        )));
    }
    Ok(())
}
