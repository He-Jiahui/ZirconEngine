//! 图校验用稳定拓扑检测父轨、发送及后效果旁链环路；就绪项按声明位置取出，保证同一图的验证顺序可复核。
//! Provides deterministic cycle detection for compiled track routes.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use zircon_runtime::core::framework::sound::{
    SoundEffectKind, SoundError, SoundMixerGraph, SoundTrackId,
};

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
    let track_positions = track_ids
        .iter()
        .copied()
        .enumerate()
        .map(|(position, track)| (track, position))
        .collect::<HashMap<_, _>>();

    for (source, target) in render_dependencies(graph) {
        outgoing.entry(source).or_default().push(target);
        *indegree.entry(target).or_default() += 1;
    }

    let mut ready = track_ids
        .iter()
        .enumerate()
        .filter_map(|(position, track)| {
            (indegree.get(track).copied().unwrap_or_default() == 0).then_some(Reverse(position))
        })
        .collect::<BinaryHeap<_>>();
    let mut order = Vec::with_capacity(track_ids.len());

    while let Some(Reverse(track_position)) = ready.pop() {
        let track = track_ids[track_position];
        order.push(track);
        if let Some(targets) = outgoing.get(&track) {
            for target in targets {
                let Some(target_indegree) = indegree.get_mut(target) else {
                    continue;
                };
                *target_indegree = target_indegree.saturating_sub(1);
                if *target_indegree == 0 {
                    if let Some(target_position) = track_positions.get(target) {
                        ready.push(Reverse(*target_position));
                    }
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

#[cfg(test)]
#[path = "tests/ordering.rs"]
mod tests;
