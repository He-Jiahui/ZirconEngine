use zircon_runtime::core::framework::sound::{SoundError, SoundGainEffect, SoundParameterId};

use super::super::helpers::unsupported_automation_parameter;

pub(super) fn apply_gain_parameter(
    gain: &mut SoundGainEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => gain.gain = value,
        _ => return Err(unsupported_automation_parameter("gain effect", parameter)),
    }
    Ok(())
}
