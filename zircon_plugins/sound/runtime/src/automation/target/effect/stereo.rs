use zircon_runtime::core::framework::sound::{SoundError, SoundPanStereoEffect, SoundParameterId};

use super::super::helpers::{bool_from_value, unsupported_automation_parameter};

pub(super) fn apply_pan_stereo_parameter(
    pan: &mut SoundPanStereoEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "pan" => pan.pan = value,
        "width" => pan.width = value,
        "left_gain" => pan.left_gain = value,
        "right_gain" => pan.right_gain = value,
        "invert_left_phase" => pan.invert_left_phase = bool_from_value(value),
        "invert_right_phase" => pan.invert_right_phase = bool_from_value(value),
        _ => {
            return Err(unsupported_automation_parameter(
                "pan stereo effect",
                parameter,
            ))
        }
    }
    Ok(())
}
