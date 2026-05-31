use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundMixerGraph, SoundSourceDescriptor, SoundTrackId,
};

use crate::engine::SoundEngineState;

use super::super::common::validate_vec3;
use super::bindings::validate_source_parameter_bindings;
use super::clip_range::validate_source_clip_range;
use super::input::validate_source_input;
use super::spatial::validate_spatial_settings;
use super::values::validate_optional_seconds;

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

fn validate_source_descriptor_for_tracks(
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
