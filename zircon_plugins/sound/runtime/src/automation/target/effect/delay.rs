use zircon_runtime::core::framework::sound::{SoundDelayEffect, SoundError, SoundParameterId};

use super::super::helpers::{non_negative_usize, unsupported_automation_parameter};

pub(super) fn apply_delay_parameter(
    delay: &mut SoundDelayEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "delay_frames" => delay.delay_frames = non_negative_usize(parameter, value)?,
        "feedback" => delay.feedback = value,
        _ => return Err(unsupported_automation_parameter("delay effect", parameter)),
    }
    Ok(())
}
