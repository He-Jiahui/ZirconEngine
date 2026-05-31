use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId, SoundVolumeDescriptor};

use super::helpers::{i32_from_value, unsupported_automation_parameter};

pub(super) fn apply_volume_parameter(
    volume: &mut SoundVolumeDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "priority" => volume.priority = i32_from_value(parameter, value)?,
        "interior_gain" => volume.interior_gain = value,
        "exterior_gain" => volume.exterior_gain = value,
        "low_pass_cutoff_hz" => {
            volume.low_pass_cutoff_hz = (value > 0.0).then_some(value);
        }
        "reverb_send" => volume.reverb_send = value,
        "crossfade_distance" => volume.crossfade_distance = value,
        _ => return Err(unsupported_automation_parameter("volume", parameter)),
    }
    Ok(())
}
