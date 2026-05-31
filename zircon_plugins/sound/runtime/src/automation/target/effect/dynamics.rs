use zircon_runtime::core::framework::sound::{
    SoundCompressorEffect, SoundError, SoundLimiterEffect, SoundParameterId,
};

use super::super::helpers::unsupported_automation_parameter;

pub(super) fn apply_compressor_parameter(
    compressor: &mut SoundCompressorEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "threshold_db" => compressor.threshold_db = value,
        "ratio" => compressor.ratio = value,
        "attack_ms" => compressor.attack_ms = value,
        "release_ms" => compressor.release_ms = value,
        "makeup_gain_db" => compressor.makeup_gain_db = value,
        _ => {
            return Err(unsupported_automation_parameter(
                "compressor effect",
                parameter,
            ))
        }
    }
    Ok(())
}

pub(super) fn apply_limiter_parameter(
    limiter: &mut SoundLimiterEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "ceiling" => limiter.ceiling = value,
        _ => {
            return Err(unsupported_automation_parameter(
                "limiter effect",
                parameter,
            ))
        }
    }
    Ok(())
}
