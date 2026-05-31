use zircon_runtime::core::framework::sound::SoundError;

pub(in crate::descriptor_validation) fn validate_vec3(
    label: &str,
    value: [f32; 3],
) -> Result<(), SoundError> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must contain finite coordinates"
        )))
    }
}
