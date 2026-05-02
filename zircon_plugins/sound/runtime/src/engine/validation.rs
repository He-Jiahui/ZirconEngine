use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundError, SoundMixerGraph, SoundTrackId,
};

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

pub(crate) fn validate_effect(effect: &SoundEffectDescriptor) -> Result<(), SoundError> {
    if !(0.0..=1.0).contains(&effect.wet) {
        return Err(SoundError::InvalidEffect(format!(
            "effect {} wet mix must be between 0 and 1",
            effect.display_name
        )));
    }
    match &effect.kind {
        SoundEffectKind::Compressor(compressor) if compressor.ratio < 1.0 => Err(
            SoundError::InvalidEffect("compressor ratio must be at least 1".to_string()),
        ),
        SoundEffectKind::Filter(filter) if filter.cutoff_hz <= 0.0 => Err(
            SoundError::InvalidEffect("filter cutoff must be positive".to_string()),
        ),
        SoundEffectKind::Limiter(limiter) if limiter.ceiling <= 0.0 => Err(
            SoundError::InvalidEffect("limiter ceiling must be positive".to_string()),
        ),
        SoundEffectKind::Chorus(chorus) if chorus.voices == 0 => Err(SoundError::InvalidEffect(
            "chorus must have at least one voice".to_string(),
        )),
        SoundEffectKind::PanStereo(pan) if pan.width < 0.0 => Err(SoundError::InvalidEffect(
            "stereo width cannot be negative".to_string(),
        )),
        _ => Ok(()),
    }
}

pub(crate) fn track_render_order(graph: &SoundMixerGraph) -> Vec<SoundTrackId> {
    topological_track_order(graph)
        .unwrap_or_else(|_| graph.tracks.iter().map(|track| track.id).collect())
}

fn validate_effect_references(
    track: SoundTrackId,
    effect: &SoundEffectDescriptor,
    track_ids: &HashSet<SoundTrackId>,
) -> Result<(), SoundError> {
    if let SoundEffectKind::Compressor(compressor) = &effect.kind {
        if let Some(sidechain) = compressor.sidechain {
            if !track_ids.contains(&sidechain.track) {
                return Err(SoundError::UnknownTrack {
                    track: sidechain.track,
                });
            }
            if !sidechain.pre_effects && sidechain.track == track {
                return Err(SoundError::InvalidMixerGraph(
                    "post-effect sidechain cannot read from the same track".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn topological_track_order(graph: &SoundMixerGraph) -> Result<Vec<SoundTrackId>, SoundError> {
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<Vec<_>>();
    let mut outgoing = track_ids
        .iter()
        .copied()
        .map(|track| (track, Vec::new()))
        .collect::<HashMap<_, _>>();
    let mut indegree = track_ids
        .iter()
        .copied()
        .map(|track| (track, 0_usize))
        .collect::<HashMap<_, _>>();

    for (source, target) in render_dependencies(graph) {
        outgoing.entry(source).or_default().push(target);
        *indegree.entry(target).or_default() += 1;
    }

    let mut ready = track_ids
        .iter()
        .copied()
        .filter(|track| indegree.get(track).copied().unwrap_or_default() == 0)
        .collect::<Vec<_>>();
    let mut order = Vec::with_capacity(track_ids.len());

    while let Some(track) = ready.first().copied() {
        ready.remove(0);
        order.push(track);
        if let Some(targets) = outgoing.get(&track) {
            for target in targets {
                let Some(target_indegree) = indegree.get_mut(target) else {
                    continue;
                };
                *target_indegree = target_indegree.saturating_sub(1);
                if *target_indegree == 0 {
                    ready.push(*target);
                    ready.sort_by_key(|candidate| {
                        track_ids
                            .iter()
                            .position(|track| track == candidate)
                            .unwrap_or(usize::MAX)
                    });
                }
            }
        }
    }

    if order.len() == track_ids.len() {
        Ok(order)
    } else {
        Err(SoundError::InvalidMixerGraph(
            "track routing contains a cycle".to_string(),
        ))
    }
}

fn render_dependencies(graph: &SoundMixerGraph) -> Vec<(SoundTrackId, SoundTrackId)> {
    let mut edges = Vec::new();
    for track in &graph.tracks {
        if let Some(parent) = track.parent {
            edges.push((track.id, parent));
        }
        for send in &track.sends {
            edges.push((track.id, send.target));
        }
        for effect in &track.effects {
            if let SoundEffectKind::Compressor(compressor) = &effect.kind {
                if let Some(sidechain) = compressor.sidechain {
                    if !sidechain.pre_effects {
                        edges.push((sidechain.track, track.id));
                    }
                }
            }
        }
    }
    edges
}
