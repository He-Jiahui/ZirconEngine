use zircon_runtime::core::framework::sound::{
    SoundChorusEffect, SoundError, SoundFlangerEffect, SoundParameterId, SoundPhaserEffect,
};

use super::super::helpers::{non_negative_usize, u8_from_value, unsupported_automation_parameter};

pub(super) fn apply_flanger_parameter(
    flanger: &mut SoundFlangerEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "delay_frames" => flanger.delay_frames = non_negative_usize(parameter, value)?,
        "depth_frames" => flanger.depth_frames = non_negative_usize(parameter, value)?,
        "rate_hz" => flanger.rate_hz = value,
        "feedback" => flanger.feedback = value,
        _ => {
            return Err(unsupported_automation_parameter(
                "flanger effect",
                parameter,
            ))
        }
    }
    Ok(())
}

pub(super) fn apply_phaser_parameter(
    phaser: &mut SoundPhaserEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "rate_hz" => phaser.rate_hz = value,
        "depth" => phaser.depth = value,
        "feedback" => phaser.feedback = value,
        "phase_offset" => phaser.phase_offset = value,
        _ => return Err(unsupported_automation_parameter("phaser effect", parameter)),
    }
    Ok(())
}

pub(super) fn apply_chorus_parameter(
    chorus: &mut SoundChorusEffect,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "voices" => chorus.voices = u8_from_value(parameter, value)?,
        "delay_frames" => chorus.delay_frames = non_negative_usize(parameter, value)?,
        "depth_frames" => chorus.depth_frames = non_negative_usize(parameter, value)?,
        "rate_hz" => chorus.rate_hz = value,
        _ => return Err(unsupported_automation_parameter("chorus effect", parameter)),
    }
    Ok(())
}
