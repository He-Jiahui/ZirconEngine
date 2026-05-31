use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId, SoundTrackDescriptor};

use super::helpers::{bool_from_value, non_negative_usize, unsupported_automation_parameter};

pub(super) fn apply_track_parameter(
    track: &mut SoundTrackDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => track.controls.gain = value,
        "pan" => track.controls.pan = value,
        "left_gain" => track.controls.left_gain = value,
        "right_gain" => track.controls.right_gain = value,
        "delay_frames" => track.controls.delay_frames = non_negative_usize(parameter, value)?,
        "invert_left_phase" => track.controls.invert_left_phase = bool_from_value(value),
        "invert_right_phase" => track.controls.invert_right_phase = bool_from_value(value),
        "mute" => track.controls.mute = bool_from_value(value),
        "solo" => track.controls.solo = bool_from_value(value),
        "bypass_effects" => track.controls.bypass_effects = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("track", parameter)),
    }
    Ok(())
}
