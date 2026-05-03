use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundExternalSourceBlock, SoundListenerDescriptor, SoundMixerGraph,
    SoundSourceDescriptor, SoundSourceInput, SoundTrackId, SoundVolumeDescriptor, SoundVolumeShape,
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
    validate_vec3("source position", source.position)?;
    validate_vec3("source forward", source.forward)?;
    validate_vec3("source velocity", source.velocity)?;
    validate_spatial_settings(source)?;
    validate_source_parameter_bindings(source)?;
    if let SoundSourceInput::Clip(clip) = &source.input {
        if !state.clips.contains_key(clip) {
            return Err(SoundError::UnknownClip { clip: *clip });
        }
    }
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
            | "playing"
            | "looped"
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
