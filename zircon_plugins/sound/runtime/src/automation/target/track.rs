use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId, SoundTrackControls};

use super::parameter_values::{
    bool_from_value, non_negative_usize, unsupported_automation_parameter,
};

pub(super) fn apply_track_parameter(
    controls: &mut SoundTrackControls,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => controls.gain = value,
        "pan" => controls.pan = value,
        "left_gain" => controls.left_gain = value,
        "right_gain" => controls.right_gain = value,
        "delay_frames" => controls.delay_frames = non_negative_usize(parameter, value)?,
        "invert_left_phase" => controls.invert_left_phase = bool_from_value(value),
        "invert_right_phase" => controls.invert_right_phase = bool_from_value(value),
        "mute" => controls.mute = bool_from_value(value),
        "solo" => controls.solo = bool_from_value(value),
        "bypass_effects" => controls.bypass_effects = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("track", parameter)),
    }
    Ok(())
}
