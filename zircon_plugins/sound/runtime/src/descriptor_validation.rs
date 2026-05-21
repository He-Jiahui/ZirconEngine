use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock, SoundHrtfProfileDescriptor,
    SoundListenerDescriptor, SoundMixerGraph, SoundSourceDescriptor, SoundSourceInput,
    SoundTrackId, SoundVolumeDescriptor, SoundVolumeShape,
};

use crate::engine::SoundEngineState;

pub(crate) fn validate_source_descriptor(
    state: &SoundEngineState,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    let track_ids = state
        .graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    validate_source_descriptor_for_tracks(state, &track_ids, source)
}

pub(crate) fn validate_source_descriptor_for_graph(
    state: &SoundEngineState,
    graph: &SoundMixerGraph,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    validate_source_descriptor_for_tracks(state, &track_ids, source)
}

pub(crate) fn validate_source_descriptor_for_tracks(
    state: &SoundEngineState,
    track_ids: &HashSet<SoundTrackId>,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    if !source.gain.is_finite() {
        return Err(SoundError::InvalidParameter(
            "source gain must be finite".to_string(),
        ));
    }
    if !source.speed.is_finite() || source.speed <= 0.0 {
        return Err(SoundError::InvalidParameter(
            "source speed must be finite and greater than zero".to_string(),
        ));
    }
    validate_optional_seconds("source start seconds", source.start_seconds)?;
    validate_optional_seconds("source duration seconds", source.duration_seconds)?;
    validate_source_clip_range(state, source)?;
    validate_vec3("source position", source.position)?;
    validate_vec3("source forward", source.forward)?;
    validate_vec3("source velocity", source.velocity)?;
    validate_spatial_settings(source)?;
    validate_source_parameter_bindings(source)?;
    validate_source_input(state, &source.input)?;
    if !track_ids.contains(&source.output_track) {
        return Err(SoundError::UnknownTrack {
            track: source.output_track,
        });
    }
    for send in &source.sends {
        if !send.gain.is_finite() {
            return Err(SoundError::InvalidParameter(
                "source send gain must be finite".to_string(),
            ));
        }
        if !track_ids.contains(&send.target) {
            return Err(SoundError::UnknownTrack { track: send.target });
        }
    }
    Ok(())
}

fn validate_optional_seconds(label: &str, seconds: Option<f32>) -> Result<(), SoundError> {
    let Some(seconds) = seconds else {
        return Ok(());
    };
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(SoundError::InvalidParameter(format!(
            "{label} must be finite and non-negative"
        )));
    }
    Ok(())
}

fn validate_source_clip_range(
    state: &SoundEngineState,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    let SoundSourceInput::Clip(clip_id) = &source.input else {
        return Ok(());
    };
    let Some(duration_seconds) = source.duration_seconds else {
        return Ok(());
    };
    let Some(clip) = state.clips.get(clip_id) else {
        return Ok(());
    };
    let frame_count = clip.asset.frame_count();
    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
    let start_frame = (source.start_seconds.unwrap_or_default() * sample_rate)
        .round()
        .max(0.0) as usize;
    let duration_frames = (duration_seconds * sample_rate).round().max(0.0) as usize;
    let start_frame = start_frame.min(frame_count);
    let end_frame = start_frame.saturating_add(duration_frames).min(frame_count);
    if end_frame <= start_frame {
        return Err(SoundError::InvalidParameter(
            "source duration must cover at least one frame".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_external_source_handle(
    handle: &ExternalAudioSourceHandle,
) -> Result<(), SoundError> {
    if handle.as_str().trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "external source handle must be non-empty".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_external_source_block(
    block: &SoundExternalSourceBlock,
) -> Result<(), SoundError> {
    if block.sample_rate_hz == 0 || block.channel_count == 0 {
        return Err(SoundError::InvalidParameter(
            "external source block sample rate and channel count must be positive".to_string(),
        ));
    }
    if block.samples.iter().any(|sample| !sample.is_finite()) {
        return Err(SoundError::InvalidParameter(
            "external source block samples must be finite".to_string(),
        ));
    }
    if block.samples.len() % block.channel_count as usize != 0 {
        return Err(SoundError::InvalidParameter(
            "external source block samples must contain whole frames".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_hrtf_profile_descriptor(
    profile: &SoundHrtfProfileDescriptor,
) -> Result<(), SoundError> {
    if profile.profile_id.trim().is_empty() || profile.display_name.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "HRTF profile requires a non-empty id and display name".to_string(),
        ));
    }
    if profile.sample_rate_hz == 0 {
        return Err(SoundError::InvalidParameter(
            "HRTF profile sample rate must be non-zero".to_string(),
        ));
    }
    if profile.left_kernel.is_empty() || profile.right_kernel.is_empty() {
        return Err(SoundError::InvalidParameter(
            "HRTF profile kernels must be non-empty".to_string(),
        ));
    }
    if profile
        .left_kernel
        .iter()
        .chain(profile.right_kernel.iter())
        .any(|sample| !sample.is_finite())
    {
        return Err(SoundError::InvalidParameter(
            "HRTF profile kernel samples must be finite".to_string(),
        ));
    }
    if !profile
        .left_kernel
        .iter()
        .chain(profile.right_kernel.iter())
        .any(|sample| *sample != 0.0)
    {
        return Err(SoundError::InvalidParameter(
            "HRTF profile requires at least one non-zero kernel sample".to_string(),
        ));
    }
    Ok(())
}

fn validate_source_input(
    state: &SoundEngineState,
    input: &SoundSourceInput,
) -> Result<(), SoundError> {
    match input {
        SoundSourceInput::Clip(clip) => {
            if !state.clips.contains_key(clip) {
                return Err(SoundError::UnknownClip { clip: *clip });
            }
        }
        SoundSourceInput::External(handle) => validate_external_source_handle(handle)?,
        SoundSourceInput::SynthParameter {
            parameter,
            default_value,
        } => {
            if parameter.as_str().trim().is_empty() || !default_value.is_finite() {
                return Err(SoundError::InvalidParameter(
                    "synth source input requires a non-empty parameter id and finite default value"
                        .to_string(),
                ));
            }
        }
        SoundSourceInput::Silence => {}
    }
    Ok(())
}

pub(crate) fn validate_listener_descriptor(
    state: &SoundEngineState,
    listener: &SoundListenerDescriptor,
) -> Result<(), SoundError> {
    validate_vec3("listener position", listener.position)?;
    validate_vec3("listener forward", listener.forward)?;
    validate_vec3("listener up", listener.up)?;
    validate_vec3("listener left ear offset", listener.left_ear_offset)?;
    validate_vec3("listener right ear offset", listener.right_ear_offset)?;
    validate_vec3("listener velocity", listener.velocity)?;
    if !state
        .graph
        .tracks
        .iter()
        .any(|track| track.id == listener.mixer_target)
    {
        return Err(SoundError::UnknownTrack {
            track: listener.mixer_target,
        });
    }
    Ok(())
}

pub(crate) fn validate_volume_descriptor(volume: &SoundVolumeDescriptor) -> Result<(), SoundError> {
    if !volume.interior_gain.is_finite()
        || !volume.exterior_gain.is_finite()
        || !volume.reverb_send.is_finite()
        || !volume.crossfade_distance.is_finite()
        || volume.crossfade_distance < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "volume gains, reverb send, and crossfade distance must be finite".to_string(),
        ));
    }
    if let Some(cutoff) = volume.low_pass_cutoff_hz {
        if !cutoff.is_finite() || cutoff <= 0.0 {
            return Err(SoundError::InvalidParameter(
                "volume low-pass cutoff must be positive and finite".to_string(),
            ));
        }
    }
    match &volume.shape {
        SoundVolumeShape::Sphere { center, radius } => {
            validate_vec3("volume sphere center", *center)?;
            if !radius.is_finite() || *radius < 0.0 {
                return Err(SoundError::InvalidParameter(
                    "volume sphere radius must be non-negative and finite".to_string(),
                ));
            }
        }
        SoundVolumeShape::Box { center, extents } => {
            validate_vec3("volume box center", *center)?;
            validate_vec3("volume box extents", *extents)?;
            if extents.iter().any(|extent| *extent < 0.0) {
                return Err(SoundError::InvalidParameter(
                    "volume box extents must be non-negative".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_source_parameter_bindings(source: &SoundSourceDescriptor) -> Result<(), SoundError> {
    for binding in &source.parameter_bindings {
        if binding.source_parameter.as_str().trim().is_empty()
            || binding.synth_parameter.as_str().trim().is_empty()
        {
            return Err(SoundError::InvalidParameter(
                "source parameter bindings require non-empty parameter ids".to_string(),
            ));
        }
        if !is_supported_source_parameter_binding(binding.source_parameter.as_str()) {
            return Err(SoundError::InvalidParameter(format!(
                "unsupported source parameter binding {}",
                binding.source_parameter.as_str()
            )));
        }
    }
    Ok(())
}

fn is_supported_source_parameter_binding(parameter: &str) -> bool {
    matches!(
        parameter,
        "gain"
            | "speed"
            | "playing"
            | "looped"
            | "muted"
            | "position_x"
            | "position_y"
            | "position_z"
            | "forward_x"
            | "forward_y"
            | "forward_z"
            | "velocity_x"
            | "velocity_y"
            | "velocity_z"
            | "spatial_blend"
            | "spatial_scale"
            | "min_distance"
            | "max_distance"
            | "cone_inner_degrees"
            | "cone_outer_degrees"
            | "doppler_factor"
            | "occlusion_enabled"
    )
}

fn validate_spatial_settings(source: &SoundSourceDescriptor) -> Result<(), SoundError> {
    let spatial = source.spatial;
    if !(0.0..=1.0).contains(&spatial.spatial_blend)
        || !spatial.min_distance.is_finite()
        || !spatial.max_distance.is_finite()
        || spatial.min_distance < 0.0
        || spatial.max_distance < spatial.min_distance
        || spatial
            .spatial_scale
            .is_some_and(|scale| !scale.is_finite() || scale < 0.0)
        || !spatial.cone_inner_degrees.is_finite()
        || !spatial.cone_outer_degrees.is_finite()
        || spatial.cone_inner_degrees < 0.0
        || spatial.cone_outer_degrees < spatial.cone_inner_degrees
        || spatial.cone_outer_degrees > 360.0
        || !spatial.doppler_factor.is_finite()
        || spatial.doppler_factor < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "source spatial settings are outside the supported range".to_string(),
        ));
    }
    Ok(())
}

fn validate_vec3(label: &str, value: [f32; 3]) -> Result<(), SoundError> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must contain finite coordinates"
        )))
    }
}
