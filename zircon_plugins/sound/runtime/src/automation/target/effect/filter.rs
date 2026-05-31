use zircon_runtime::core::framework::sound::{SoundError, SoundFilterEffect, SoundParameterId};

use super::super::helpers::unsupported_automation_parameter;

pub(super) fn apply_filter_parameter(
    filter: &mut SoundFilterEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "cutoff_hz" => filter.cutoff_hz = value,
        "resonance" => filter.resonance = value,
        "gain_db" => filter.gain_db = value,
        _ => return Err(unsupported_automation_parameter("filter effect", parameter)),
    }
    Ok(())
}
