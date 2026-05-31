use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph};

use super::effect::validate_effect;
use super::ordering::topological_track_order;
use super::references::validate_effect_references;
use super::track::{validate_track_controls, validate_track_send};

pub(crate) fn validate_graph(graph: &SoundMixerGraph) -> Result<(), SoundError> {
    if graph.master_track().is_none() {
        return Err(SoundError::InvalidMixerGraph(
            "mixer graph must contain the master track".to_string(),
        ));
    }
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    if track_ids.len() != graph.tracks.len() {
        return Err(SoundError::InvalidMixerGraph(
            "mixer graph contains duplicate track ids".to_string(),
        ));
    }
    for track in &graph.tracks {
        validate_track_controls(track)?;
        if let Some(parent) = track.parent {
            if !track_ids.contains(&parent) {
                return Err(SoundError::UnknownTrack { track: parent });
            }
            if parent == track.id {
                return Err(SoundError::InvalidMixerGraph(
                    "track cannot route to itself".to_string(),
                ));
            }
        }
        for send in &track.sends {
            validate_track_send(track, send)?;
            if !track_ids.contains(&send.target) {
                return Err(SoundError::UnknownTrack { track: send.target });
            }
            if send.target == track.id {
                return Err(SoundError::InvalidMixerGraph(
                    "track send cannot route to itself".to_string(),
                ));
            }
        }
        for effect in &track.effects {
            validate_effect_references(track.id, effect, &track_ids)?;
            validate_effect(effect)?;
        }
    }

    let order = topological_track_order(graph)?;
    if order.len() != graph.tracks.len() {
        return Err(SoundError::InvalidMixerGraph(
            "track routing contains a cycle".to_string(),
        ));
    }
    Ok(())
}
