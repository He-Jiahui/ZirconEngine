use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId};

pub(super) fn bool_from_value(value: f32) -> bool {
    value >= 0.5
}

pub(super) fn non_negative_usize(
    parameter: &SoundParameterId,
    value: f32,
) -> Result<usize, SoundError> {
    if value < 0.0 || value > usize::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must be a non-negative frame count",
            parameter.as_str()
        )));
    }
    Ok(value.round() as usize)
}

pub(super) fn u8_from_value(parameter: &SoundParameterId, value: f32) -> Result<u8, SoundError> {
    if value < 0.0 || value > u8::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in u8",
            parameter.as_str()
        )));
    }
    Ok(value.round() as u8)
}

pub(super) fn i32_from_value(parameter: &SoundParameterId, value: f32) -> Result<i32, SoundError> {
    if value < i32::MIN as f32 || value > i32::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in i32",
            parameter.as_str()
        )));
    }
    Ok(value.round() as i32)
}

pub(super) fn unsupported_automation_parameter(
    target: &str,
    parameter: &SoundParameterId,
) -> SoundError {
    SoundError::InvalidParameter(format!(
        "unsupported sound automation parameter {} for {target}",
        parameter.as_str()
    ))
}
