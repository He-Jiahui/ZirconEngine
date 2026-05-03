use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationCurve, SoundAutomationInterpolation,
    SoundAutomationTarget, SoundEffectDescriptor, SoundEffectKind, SoundError,
    SoundListenerDescriptor, SoundParameterId, SoundSourceDescriptor, SoundTrackDescriptor,
    SoundVolumeDescriptor,
};

use crate::descriptor_validation::{
    validate_listener_descriptor, validate_source_descriptor, validate_volume_descriptor,
};
use crate::engine::validation::{validate_effect, validate_graph};
use crate::engine::SoundEngineState;

pub(crate) fn apply_automation_target(
    state: &mut SoundEngineState,
    target: SoundAutomationTarget,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match target {
        SoundAutomationTarget::Track(track) => {
            let mut graph = state.graph.clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            apply_track_parameter(track_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.graph = graph;
            Ok(())
        }
        SoundAutomationTarget::Effect { track, effect } => {
            let mut graph = state.graph.clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            let effect_descriptor = track_descriptor
                .effects
                .iter_mut()
                .find(|candidate| candidate.id == effect)
                .ok_or(SoundError::UnknownEffect { effect })?;
            apply_effect_parameter(effect_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.graph = graph;
            Ok(())
        }
        SoundAutomationTarget::Source(source_id) => {
            let mut descriptor = state
                .sources
                .get(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor
                .clone();
            apply_source_parameter(&mut descriptor, parameter, value)?;
            validate_source_descriptor(state, &descriptor)?;
            state
                .sources
                .get_mut(&source_id)
                .expect("validated source disappeared")
                .descriptor = descriptor;
            Ok(())
        }
        SoundAutomationTarget::Listener(listener) => {
            let mut descriptor = state
                .listeners
                .get(&listener)
                .ok_or(SoundError::UnknownListener { listener })?
                .clone();
            apply_listener_parameter(&mut descriptor, parameter, value)?;
            validate_listener_descriptor(state, &descriptor)?;
            state.listeners.insert(listener, descriptor);
            Ok(())
        }
        SoundAutomationTarget::Volume(volume) => {
            let mut descriptor = state
                .volumes
                .get(&volume)
                .ok_or(SoundError::UnknownVolume { volume })?
                .clone();
            apply_volume_parameter(&mut descriptor, parameter, value)?;
            validate_volume_descriptor(&descriptor)?;
            state.volumes.insert(volume, descriptor);
            Ok(())
        }
        SoundAutomationTarget::SynthParameter(target_parameter) => {
            if parameter.as_str() != "value" && parameter.as_str() != target_parameter.as_str() {
                return Err(unsupported_automation_parameter(
                    "synth parameter",
                    parameter,
                ));
            }
            state.parameters.insert(target_parameter, value);
            Ok(())
        }
    }
}

pub(crate) fn sample_automation_curve(
    curve: &SoundAutomationCurve,
    time_seconds: f32,
) -> Result<f32, SoundError> {
    ensure_finite_value("automation curve time", time_seconds)?;
    validate_automation_curve(curve)?;

    let first = curve
        .keyframes
        .first()
        .expect("validated automation curve has at least one keyframe");
    if time_seconds <= first.time_seconds {
        return Ok(first.value);
    }

    for window in curve.keyframes.windows(2) {
        let start = window[0];
        let end = window[1];
        if time_seconds <= end.time_seconds {
            return Ok(interpolate_automation_value(start, end, time_seconds));
        }
    }

    Ok(curve
        .keyframes
        .last()
        .expect("validated automation curve has at least one keyframe")
        .value)
}

pub(crate) fn validate_automation_binding(
    binding: &SoundAutomationBinding,
) -> Result<(), SoundError> {
    if binding.timeline_track_path.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "automation binding requires a timeline track path".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_automation_curve(curve: &SoundAutomationCurve) -> Result<(), SoundError> {
    if curve.keyframes.is_empty() {
        return Err(SoundError::InvalidParameter(
            "automation curve requires at least one keyframe".to_string(),
        ));
    }

    let mut previous_time = None;
    for keyframe in &curve.keyframes {
        ensure_finite_value("automation keyframe time", keyframe.time_seconds)?;
        ensure_finite_value("automation keyframe value", keyframe.value)?;
        if let Some(previous_time) = previous_time {
            if keyframe.time_seconds <= previous_time {
                return Err(SoundError::InvalidParameter(
                    "automation curve keyframes must be strictly increasing".to_string(),
                ));
            }
        }
        previous_time = Some(keyframe.time_seconds);
    }
    Ok(())
}

fn interpolate_automation_value(
    start: zircon_runtime::core::framework::sound::SoundAutomationKeyframe,
    end: zircon_runtime::core::framework::sound::SoundAutomationKeyframe,
    time_seconds: f32,
) -> f32 {
    match start.interpolation {
        SoundAutomationInterpolation::Step => start.value,
        SoundAutomationInterpolation::Linear | SoundAutomationInterpolation::SmoothStep => {
            let span = (end.time_seconds - start.time_seconds).max(f32::EPSILON);
            let mut amount = ((time_seconds - start.time_seconds) / span).clamp(0.0, 1.0);
            if start.interpolation == SoundAutomationInterpolation::SmoothStep {
                amount = amount * amount * (3.0 - 2.0 * amount);
            }
            start.value + (end.value - start.value) * amount
        }
    }
}

fn apply_track_parameter(
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

fn apply_effect_parameter(
    effect: &mut SoundEffectDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "enabled" => {
            effect.enabled = bool_from_value(value);
            return Ok(());
        }
        "bypass" => {
            effect.bypass = bool_from_value(value);
            return Ok(());
        }
        "wet" => {
            effect.wet = value;
            validate_effect(effect)?;
            return Ok(());
        }
        _ => {}
    }

    match &mut effect.kind {
        SoundEffectKind::Gain(gain) => match parameter.as_str() {
            "gain" => gain.gain = value,
            _ => return Err(unsupported_automation_parameter("gain effect", parameter)),
        },
        SoundEffectKind::Filter(filter) => match parameter.as_str() {
            "cutoff_hz" => filter.cutoff_hz = value,
            "resonance" => filter.resonance = value,
            "gain_db" => filter.gain_db = value,
            _ => return Err(unsupported_automation_parameter("filter effect", parameter)),
        },
        SoundEffectKind::Reverb(reverb) => match parameter.as_str() {
            "room_size" => reverb.room_size = value,
            "damping" => reverb.damping = value,
            "pre_delay_frames" => {
                reverb.pre_delay_frames = non_negative_usize(parameter, value)?;
            }
            "tail_frames" => reverb.tail_frames = non_negative_usize(parameter, value)?,
            _ => return Err(unsupported_automation_parameter("reverb effect", parameter)),
        },
        SoundEffectKind::ConvolutionReverb(convolution) => match parameter.as_str() {
            "latency_frames" => {
                convolution.latency_frames = non_negative_usize(parameter, value)?;
            }
            "fallback_to_algorithmic" => {
                convolution.fallback_to_algorithmic = bool_from_value(value)
            }
            _ => {
                return Err(unsupported_automation_parameter(
                    "convolution reverb effect",
                    parameter,
                ));
            }
        },
        SoundEffectKind::Compressor(compressor) => match parameter.as_str() {
            "threshold_db" => compressor.threshold_db = value,
            "ratio" => compressor.ratio = value,
            "attack_ms" => compressor.attack_ms = value,
            "release_ms" => compressor.release_ms = value,
            "makeup_gain_db" => compressor.makeup_gain_db = value,
            _ => {
                return Err(unsupported_automation_parameter(
                    "compressor effect",
                    parameter,
                ));
            }
        },
        SoundEffectKind::WaveShaper(shaper) => match parameter.as_str() {
            "drive" => shaper.drive = value,
            _ => {
                return Err(unsupported_automation_parameter(
                    "wave shaper effect",
                    parameter,
                ));
            }
        },
        SoundEffectKind::Flanger(flanger) => match parameter.as_str() {
            "delay_frames" => flanger.delay_frames = non_negative_usize(parameter, value)?,
            "depth_frames" => flanger.depth_frames = non_negative_usize(parameter, value)?,
            "rate_hz" => flanger.rate_hz = value,
            "feedback" => flanger.feedback = value,
            _ => {
                return Err(unsupported_automation_parameter(
                    "flanger effect",
                    parameter,
                ));
            }
        },
        SoundEffectKind::Phaser(phaser) => match parameter.as_str() {
            "rate_hz" => phaser.rate_hz = value,
            "depth" => phaser.depth = value,
            "feedback" => phaser.feedback = value,
            "phase_offset" => phaser.phase_offset = value,
            _ => return Err(unsupported_automation_parameter("phaser effect", parameter)),
        },
        SoundEffectKind::Chorus(chorus) => match parameter.as_str() {
            "voices" => chorus.voices = u8_from_value(parameter, value)?,
            "delay_frames" => chorus.delay_frames = non_negative_usize(parameter, value)?,
            "depth_frames" => chorus.depth_frames = non_negative_usize(parameter, value)?,
            "rate_hz" => chorus.rate_hz = value,
            _ => return Err(unsupported_automation_parameter("chorus effect", parameter)),
        },
        SoundEffectKind::Delay(delay) => match parameter.as_str() {
            "delay_frames" => delay.delay_frames = non_negative_usize(parameter, value)?,
            "feedback" => delay.feedback = value,
            _ => return Err(unsupported_automation_parameter("delay effect", parameter)),
        },
        SoundEffectKind::PanStereo(pan) => match parameter.as_str() {
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
                ));
            }
        },
        SoundEffectKind::Limiter(limiter) => match parameter.as_str() {
            "ceiling" => limiter.ceiling = value,
            _ => {
                return Err(unsupported_automation_parameter(
                    "limiter effect",
                    parameter,
                ));
            }
        },
    }
    validate_effect(effect)
}

fn apply_source_parameter(
    source: &mut SoundSourceDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => source.gain = value,
        "playing" => source.playing = bool_from_value(value),
        "looped" => source.looped = bool_from_value(value),
        "position_x" => source.position[0] = value,
        "position_y" => source.position[1] = value,
        "position_z" => source.position[2] = value,
        "forward_x" => source.forward[0] = value,
        "forward_y" => source.forward[1] = value,
        "forward_z" => source.forward[2] = value,
        "velocity_x" => source.velocity[0] = value,
        "velocity_y" => source.velocity[1] = value,
        "velocity_z" => source.velocity[2] = value,
        "spatial_blend" => source.spatial.spatial_blend = value,
        "min_distance" => source.spatial.min_distance = value,
        "max_distance" => source.spatial.max_distance = value,
        "cone_inner_degrees" => source.spatial.cone_inner_degrees = value,
        "cone_outer_degrees" => source.spatial.cone_outer_degrees = value,
        "doppler_factor" => source.spatial.doppler_factor = value,
        "occlusion_enabled" => source.spatial.occlusion_enabled = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("source", parameter)),
    }
    Ok(())
}

fn apply_listener_parameter(
    listener: &mut SoundListenerDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "active" => listener.active = bool_from_value(value),
        "doppler_tracking" => listener.doppler_tracking = bool_from_value(value),
        "position_x" => listener.position[0] = value,
        "position_y" => listener.position[1] = value,
        "position_z" => listener.position[2] = value,
        "forward_x" => listener.forward[0] = value,
        "forward_y" => listener.forward[1] = value,
        "forward_z" => listener.forward[2] = value,
        "up_x" => listener.up[0] = value,
        "up_y" => listener.up[1] = value,
        "up_z" => listener.up[2] = value,
        "velocity_x" => listener.velocity[0] = value,
        "velocity_y" => listener.velocity[1] = value,
        "velocity_z" => listener.velocity[2] = value,
        "left_ear_offset_x" => listener.left_ear_offset[0] = value,
        "left_ear_offset_y" => listener.left_ear_offset[1] = value,
        "left_ear_offset_z" => listener.left_ear_offset[2] = value,
        "right_ear_offset_x" => listener.right_ear_offset[0] = value,
        "right_ear_offset_y" => listener.right_ear_offset[1] = value,
        "right_ear_offset_z" => listener.right_ear_offset[2] = value,
        _ => return Err(unsupported_automation_parameter("listener", parameter)),
    }
    Ok(())
}

fn apply_volume_parameter(
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

pub(crate) fn ensure_finite_value(label: &str, value: f32) -> Result<(), SoundError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must be finite"
        )))
    }
}

fn bool_from_value(value: f32) -> bool {
    value >= 0.5
}

fn non_negative_usize(parameter: &SoundParameterId, value: f32) -> Result<usize, SoundError> {
    if value < 0.0 || value > usize::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must be a non-negative frame count",
            parameter.as_str()
        )));
    }
    Ok(value.round() as usize)
}

fn u8_from_value(parameter: &SoundParameterId, value: f32) -> Result<u8, SoundError> {
    if value < 0.0 || value > u8::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in u8",
            parameter.as_str()
        )));
    }
    Ok(value.round() as u8)
}

fn i32_from_value(parameter: &SoundParameterId, value: f32) -> Result<i32, SoundError> {
    if value < i32::MIN as f32 || value > i32::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in i32",
            parameter.as_str()
        )));
    }
    Ok(value.round() as i32)
}

fn unsupported_automation_parameter(target: &str, parameter: &SoundParameterId) -> SoundError {
    SoundError::InvalidParameter(format!(
        "unsupported sound automation parameter {} for {target}",
        parameter.as_str()
    ))
}
