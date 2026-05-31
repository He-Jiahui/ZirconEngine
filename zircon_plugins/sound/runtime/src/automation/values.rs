use zircon_runtime::core::framework::sound::SoundError;

pub(crate) fn ensure_finite_value(label: &str, value: f32) -> Result<(), SoundError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must be finite"
        )))
    }
}
