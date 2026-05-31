use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundEffectKind, SoundError, SoundMixerGraph, SoundTrackId,
};

pub(crate) fn track_render_order(graph: &SoundMixerGraph) -> Vec<SoundTrackId> {
    topological_track_order(graph)
        .unwrap_or_else(|_| graph.tracks.iter().map(|track| track.id).collect())
}

pub(super) fn topological_track_order(
    graph: &SoundMixerGraph,
) -> Result<Vec<SoundTrackId>, SoundError> {
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
