use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId, SoundWaveShaperEffect};

use super::super::helpers::unsupported_automation_parameter;

pub(super) fn apply_wave_shaper_parameter(
    shaper: &mut SoundWaveShaperEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "drive" => shaper.drive = value,
        _ => {
            return Err(unsupported_automation_parameter(
                "wave shaper effect",
                parameter,
            ))
        }
    }
    Ok(())
}
