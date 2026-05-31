use zircon_runtime::core::framework::sound::{
    SoundConvolutionReverbEffect, SoundError, SoundParameterId, SoundReverbEffect,
};

use super::super::helpers::{
    bool_from_value, non_negative_usize, unsupported_automation_parameter,
};

pub(super) fn apply_reverb_parameter(
    reverb: &mut SoundReverbEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "room_size" => reverb.room_size = value,
        "damping" => reverb.damping = value,
        "pre_delay_frames" => {
            reverb.pre_delay_frames = non_negative_usize(parameter, value)?;
        }
        "tail_frames" => reverb.tail_frames = non_negative_usize(parameter, value)?,
        _ => return Err(unsupported_automation_parameter("reverb effect", parameter)),
    }
    Ok(())
}

pub(super) fn apply_convolution_reverb_parameter(
    convolution: &mut SoundConvolutionReverbEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "latency_frames" => {
            convolution.latency_frames = non_negative_usize(parameter, value)?;
        }
        "fallback_to_algorithmic" => convolution.fallback_to_algorithmic = bool_from_value(value),
        _ => {
            return Err(unsupported_automation_parameter(
                "convolution reverb effect",
                parameter,
            ));
        }
    }
    Ok(())
}
