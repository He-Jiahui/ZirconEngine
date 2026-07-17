#[cfg(test)]
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use kira::{Decibels, Easing, Tween};
use zircon_runtime::core::framework::sound::{
    SoundError, SoundMixerGraph, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

use super::graph_validation::validate_graph;

mod routes;

use routes::expanded_post_effect_sends;

pub(crate) const PARAMETER_TWEEN_DURATION: Duration = Duration::from_millis(10);
const STAGED_CHILD_GENERATIONS: usize = 3;

#[cfg(test)]
thread_local! {
    static GRAPH_COMPILE_INVOCATIONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompiledTrack {
    pub(crate) id: SoundTrackId,
    pub(crate) parent: Option<SoundTrackId>,
    pub(crate) sends: Vec<SoundTrackSend>,
    pub(crate) volume: Decibels,
    pub(crate) child_capacity: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompiledSoundGraph {
    tracks: Vec<CompiledTrack>,
    send_targets: Vec<SoundTrackId>,
    send_target_linear_gains: HashMap<SoundTrackId, f32>,
}

impl CompiledSoundGraph {
    pub(crate) fn tracks(&self) -> &[CompiledTrack] {
        &self.tracks
    }

    pub(crate) fn send_targets(&self) -> &[SoundTrackId] {
        &self.send_targets
    }

    pub(crate) fn send_target_linear_gain(&self, target: SoundTrackId) -> Option<f32> {
        self.send_target_linear_gains.get(&target).copied()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GraphSyncAction {
    SetTrackVolume {
        track: SoundTrackId,
        linear_gain: f32,
        volume: Decibels,
        tween: Tween,
    },
    SetSendVolume {
        target: SoundTrackId,
        linear_gain: f32,
        volume: Decibels,
        tween: Tween,
    },
    SetTrackSendVolume {
        track: SoundTrackId,
        target: SoundTrackId,
        linear_gain: f32,
        volume: Decibels,
        tween: Tween,
    },
    AddTrack {
        track: CompiledTrack,
    },
    RemoveTrack {
        track: SoundTrackId,
    },
    RebuildSubtree {
        root: SoundTrackId,
    },
    RebuildGraph,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphDiff {
    actions: Vec<GraphSyncAction>,
}

impl GraphDiff {
    pub(crate) fn actions(&self) -> &[GraphSyncAction] {
        &self.actions
    }
}

pub(crate) struct GraphSyncPlan {
    compiled: CompiledSoundGraph,
    diff: GraphDiff,
}

impl GraphSyncPlan {
    pub(crate) fn compiled(&self) -> &CompiledSoundGraph {
        &self.compiled
    }

    pub(crate) fn diff(&self) -> &GraphDiff {
        &self.diff
    }
}

pub(crate) fn linear_gain_to_decibels(gain: f32) -> Decibels {
    if gain <= 0.0 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * gain.log10())
    }
}

pub(crate) fn compile_graph(graph: &SoundMixerGraph) -> Result<CompiledSoundGraph, SoundError> {
    #[cfg(test)]
    GRAPH_COMPILE_INVOCATIONS.with(|count| count.set(count.get() + 1));

    validate_graph(graph)?;
    validate_m1_surface(graph)?;

    let child_counts = graph.tracks.iter().filter_map(|track| track.parent).fold(
        HashMap::<SoundTrackId, usize>::new(),
        |mut counts, parent| {
            *counts.entry(parent).or_default() += 1;
            counts
        },
    );
    let track_lookup = graph
        .tracks
        .iter()
        .map(|track| (track.id, track))
        .collect::<HashMap<_, _>>();
    let expanded_sends = expanded_post_effect_sends(&track_lookup)?;
    let tracks = graph
        .tracks
        .iter()
        .map(|track| CompiledTrack {
            id: track.id,
            parent: track.parent,
            sends: expanded_sends.get(&track.id).cloned().unwrap_or_default(),
            volume: linear_gain_to_decibels(if track.controls.mute {
                0.0
            } else {
                track.controls.gain
            }),
            child_capacity: staged_child_capacity(
                child_counts.get(&track.id).copied().unwrap_or_default(),
            ),
        })
        .collect();
    let mut send_targets = graph
        .tracks
        .iter()
        .flat_map(|track| track.sends.iter().map(|send| send.target))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    send_targets.sort_by_key(|track| track.raw());
    let send_target_linear_gains = send_targets
        .iter()
        .copied()
        .map(|target| Ok((target, effective_track_linear_gain(&track_lookup, target)?)))
        .collect::<Result<HashMap<_, _>, SoundError>>()?;
    Ok(CompiledSoundGraph {
        tracks,
        send_targets,
        send_target_linear_gains,
    })
}

pub(crate) fn diff_graphs(
    before: &SoundMixerGraph,
    after: &SoundMixerGraph,
) -> Result<GraphDiff, SoundError> {
    let compiled_after = compile_graph(after)?;
    diff_compiled_graphs(before, after, &compiled_after)
}

pub(crate) fn compile_graph_update(
    before: &SoundMixerGraph,
    after: &SoundMixerGraph,
) -> Result<GraphSyncPlan, SoundError> {
    let compiled = compile_graph(after)?;
    let diff = diff_compiled_graphs(before, after, &compiled)?;
    Ok(GraphSyncPlan { compiled, diff })
}

fn diff_compiled_graphs(
    before: &SoundMixerGraph,
    after: &SoundMixerGraph,
    compiled_after: &CompiledSoundGraph,
) -> Result<GraphDiff, SoundError> {
    if before.sample_rate_hz != after.sample_rate_hz
        || before.channel_count != after.channel_count
        || before.channel_layout != after.channel_layout
    {
        return Ok(GraphDiff {
            actions: vec![GraphSyncAction::RebuildGraph],
        });
    }
    let before_tracks = before
        .tracks
        .iter()
        .map(|track| (track.id, track))
        .collect::<HashMap<_, _>>();
    let after_tracks = after
        .tracks
        .iter()
        .map(|track| (track.id, track))
        .collect::<HashMap<_, _>>();
    let before_expanded_sends = expanded_post_effect_sends(&before_tracks)?;
    let after_expanded_sends = compiled_after
        .tracks()
        .iter()
        .map(|track| (track.id, track.sends.as_slice()))
        .collect::<HashMap<_, _>>();
    let mut structural_candidates = after
        .tracks
        .iter()
        .filter_map(|track| {
            before_tracks.get(&track.id).and_then(|previous| {
                let before_targets = before_expanded_sends
                    .get(&track.id)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let after_targets = after_expanded_sends
                    .get(&track.id)
                    .copied()
                    .unwrap_or_default();
                (previous.parent != track.parent
                    || send_route_topology_changed(before_targets, after_targets))
                .then_some(track.id)
            })
        })
        .collect::<HashSet<_>>();
    for track in &before.tracks {
        if after_tracks.contains_key(&track.id) {
            continue;
        }
        insert_rebuildable_parent(&mut structural_candidates, track.parent, &after_tracks);
    }
    for track in &after.tracks {
        match before_tracks.get(&track.id).copied() {
            None => {
                insert_rebuildable_parent(&mut structural_candidates, track.parent, &after_tracks);
            }
            Some(previous) if previous.parent != track.parent => {
                insert_rebuildable_parent(
                    &mut structural_candidates,
                    previous.parent,
                    &after_tracks,
                );
                insert_rebuildable_parent(&mut structural_candidates, track.parent, &after_tracks);
            }
            Some(_) => {}
        }
    }
    let rebuilt_roots = structural_candidates
        .iter()
        .copied()
        .filter(|candidate| !has_ancestor_in(after, *candidate, &structural_candidates))
        .collect::<HashSet<_>>();
    let rebuilt_before = rebuilt_roots
        .iter()
        .flat_map(|root| subtree_ids(before, *root))
        .collect::<HashSet<_>>();
    let rebuilt_after = rebuilt_roots
        .iter()
        .flat_map(|root| subtree_ids(after, *root))
        .collect::<HashSet<_>>();
    let mut actions = Vec::new();

    let mut removed = before
        .tracks
        .iter()
        .filter(|track| {
            !after_tracks.contains_key(&track.id) && !rebuilt_before.contains(&track.id)
        })
        .collect::<Vec<_>>();
    removed.sort_by_key(|track| std::cmp::Reverse(track_depth(before, track.id)));
    actions.extend(
        removed
            .into_iter()
            .map(|track| GraphSyncAction::RemoveTrack { track: track.id }),
    );

    let mut roots = rebuilt_roots.iter().copied().collect::<Vec<_>>();
    roots.sort_by_key(|track| track_depth(after, *track));
    actions.extend(
        roots
            .into_iter()
            .map(|root| GraphSyncAction::RebuildSubtree { root }),
    );

    for track in &after.tracks {
        let Some(previous) = before_tracks.get(&track.id).copied() else {
            continue;
        };
        if rebuilt_after.contains(&track.id) {
            continue;
        }
        let previous_gain = if previous.controls.mute {
            0.0
        } else {
            previous.controls.gain
        };
        let next_gain = if track.controls.mute {
            0.0
        } else {
            track.controls.gain
        };
        if previous_gain != next_gain {
            actions.push(GraphSyncAction::SetTrackVolume {
                track: track.id,
                linear_gain: next_gain,
                volume: linear_gain_to_decibels(next_gain),
                tween: parameter_tween(),
            });
        }

        let before_sends = before_expanded_sends
            .get(&track.id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let after_sends = after_expanded_sends
            .get(&track.id)
            .copied()
            .unwrap_or_default();
        for after_send in after_sends {
            let Some(before_send) = before_sends
                .iter()
                .find(|before_send| before_send.target == after_send.target)
            else {
                continue;
            };
            if before_send.gain != after_send.gain {
                actions.push(GraphSyncAction::SetTrackSendVolume {
                    track: track.id,
                    target: after_send.target,
                    linear_gain: after_send.gain,
                    volume: linear_gain_to_decibels(after_send.gain),
                    tween: parameter_tween(),
                });
            }
        }
    }

    let before_send_targets = send_target_ids(before);
    for target in compiled_after.send_targets().iter().copied() {
        if !before_send_targets.contains(&target) {
            continue;
        }
        let before_gain = effective_track_linear_gain(&before_tracks, target)?;
        let after_gain = compiled_after
            .send_target_linear_gain(target)
            .ok_or(SoundError::UnknownTrack { track: target })?;
        if before_gain != after_gain {
            actions.push(GraphSyncAction::SetSendVolume {
                target,
                linear_gain: after_gain,
                volume: linear_gain_to_decibels(after_gain),
                tween: parameter_tween(),
            });
        }
    }

    let mut added = compiled_after
        .tracks()
        .iter()
        .filter(|track| !before_tracks.contains_key(&track.id))
        .cloned()
        .collect::<Vec<_>>();
    added.sort_by_key(|track| track_depth(after, track.id));
    actions.extend(
        added
            .into_iter()
            .filter(|track| !rebuilt_after.contains(&track.id))
            .map(|track| GraphSyncAction::AddTrack { track }),
    );
    Ok(GraphDiff { actions })
}

fn staged_child_capacity(child_count: usize) -> usize {
    child_count.saturating_mul(STAGED_CHILD_GENERATIONS).max(1)
}

fn insert_rebuildable_parent(
    candidates: &mut HashSet<SoundTrackId>,
    parent: Option<SoundTrackId>,
    after_tracks: &HashMap<SoundTrackId, &SoundTrackDescriptor>,
) {
    if let Some(parent) = parent.filter(|parent| *parent != SoundTrackId::master()) {
        if after_tracks.contains_key(&parent) {
            candidates.insert(parent);
        }
    }
}

fn send_target_ids(graph: &SoundMixerGraph) -> HashSet<SoundTrackId> {
    graph
        .tracks
        .iter()
        .flat_map(|track| track.sends.iter().map(|send| send.target))
        .collect()
}

fn send_route_topology_changed(before: &[SoundTrackSend], after: &[SoundTrackSend]) -> bool {
    before.len() != after.len()
        || before
            .iter()
            .zip(after)
            .any(|(before, after)| before.target != after.target)
}

fn effective_track_linear_gain(
    tracks: &HashMap<SoundTrackId, &SoundTrackDescriptor>,
    target: SoundTrackId,
) -> Result<f32, SoundError> {
    let mut gain = 1.0;
    let mut current = Some(target);
    while let Some(track_id) = current {
        if track_id == SoundTrackId::master() {
            break;
        }
        let track = tracks
            .get(&track_id)
            .ok_or(SoundError::UnknownTrack { track: track_id })?;
        gain *= if track.controls.mute {
            0.0
        } else {
            track.controls.gain
        };
        current = track.parent;
    }
    Ok(gain)
}

#[cfg(test)]
pub(crate) fn reset_graph_compile_invocations() {
    GRAPH_COMPILE_INVOCATIONS.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn graph_compile_invocations() -> usize {
    GRAPH_COMPILE_INVOCATIONS.with(Cell::get)
}

fn parameter_tween() -> Tween {
    Tween {
        duration: PARAMETER_TWEEN_DURATION,
        easing: Easing::Linear,
        ..Tween::default()
    }
}

fn track_depth(graph: &SoundMixerGraph, track: SoundTrackId) -> usize {
    let parents = graph
        .tracks
        .iter()
        .map(|candidate| (candidate.id, candidate.parent))
        .collect::<HashMap<_, _>>();
    let mut depth = 0;
    let mut cursor = parents.get(&track).copied().flatten();
    while let Some(parent) = cursor {
        depth += 1;
        cursor = parents.get(&parent).copied().flatten();
    }
    depth
}

fn has_ancestor_in(
    graph: &SoundMixerGraph,
    track: SoundTrackId,
    candidates: &HashSet<SoundTrackId>,
) -> bool {
    let parents = graph
        .tracks
        .iter()
        .map(|candidate| (candidate.id, candidate.parent))
        .collect::<HashMap<_, _>>();
    let mut cursor = parents.get(&track).copied().flatten();
    while let Some(parent) = cursor {
        if candidates.contains(&parent) {
            return true;
        }
        cursor = parents.get(&parent).copied().flatten();
    }
    false
}

fn subtree_ids(graph: &SoundMixerGraph, root: SoundTrackId) -> HashSet<SoundTrackId> {
    let parents = graph
        .tracks
        .iter()
        .map(|track| (track.id, track.parent))
        .collect::<HashMap<_, _>>();
    let mut subtree = HashSet::from([root]);
    loop {
        let before = subtree.len();
        for (track, parent) in &parents {
            if parent.is_some_and(|parent| subtree.contains(&parent)) {
                subtree.insert(*track);
            }
        }
        if subtree.len() == before {
            return subtree;
        }
    }
}

fn validate_m1_surface(graph: &SoundMixerGraph) -> Result<(), SoundError> {
    for track in &graph.tracks {
        if !track.effects.is_empty() {
            return Err(SoundError::UnsupportedAdvancedFeature(
                "sound effects are enabled by Sound M2".to_string(),
            ));
        }
        let controls = track.controls;
        if controls.pan != 0.0
            || controls.left_gain != 1.0
            || controls.right_gain != 1.0
            || controls.delay_frames != 0
            || controls.invert_left_phase
            || controls.invert_right_phase
            || controls.solo
            || controls.bypass_effects
        {
            return Err(SoundError::UnsupportedAdvancedFeature(
                "advanced track controls are enabled by Sound M2".to_string(),
            ));
        }
        if track.sends.iter().any(|send| send.pre_effects) {
            return Err(SoundError::UnsupportedAdvancedFeature(
                "pre-effect sends are enabled by Sound M2".to_string(),
            ));
        }
    }
    Ok(())
}
