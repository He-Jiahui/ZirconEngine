use std::collections::{HashMap, HashSet};

use kira::{backend::Backend, track::SendTrackBuilder};
use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph, SoundTrackId};

use super::super::super::graph_compile::{CompiledSoundGraph, CompiledTrack, GraphSyncAction};
use super::{resource_limit_error, KiraEngine};

impl<B: Backend> KiraEngine<B> {
    pub(super) fn apply_structural_plan(
        &mut self,
        graph: &CompiledSoundGraph,
        actions: &[GraphSyncAction],
    ) -> Result<(), SoundError> {
        self.validate_parameter_actions(actions)?;
        let previous = self.graph.as_ref().ok_or_else(|| {
            SoundError::InvalidMixerGraph(
                "cannot apply a structural diff before graph install".to_string(),
            )
        })?;
        let mut remove_tracks = HashSet::new();
        let mut stage_tracks = HashSet::new();
        for action in actions {
            match action {
                GraphSyncAction::AddTrack { track } => {
                    stage_tracks.insert(track.id);
                }
                GraphSyncAction::RemoveTrack { track } => {
                    remove_tracks.insert(*track);
                }
                GraphSyncAction::RebuildSubtree { root } => {
                    remove_tracks.extend(subtree_ids(previous, *root));
                    stage_tracks.extend(subtree_ids_from_compiled(graph, *root));
                }
                GraphSyncAction::SetTrackVolume { .. }
                | GraphSyncAction::SetSendVolume { .. }
                | GraphSyncAction::SetTrackSendVolume { .. } => {}
                GraphSyncAction::RebuildGraph => {
                    return Err(SoundError::InvalidMixerGraph(
                        "full graph rebuild reached the incremental transaction path".to_string(),
                    ));
                }
            }
        }

        let staged_sends = self.stage_missing_send_tracks(graph)?;
        let staged_tracks = self.stage_track_handles(graph, &stage_tracks, &staged_sends)?;

        // Lookups were validated before staging, so no fallible operation remains after this point.
        self.apply_parameter_plan(actions)?;
        self.tracks
            .retain(|track, _| !remove_tracks.contains(track));
        self.tracks.extend(staged_tracks);
        self.send_tracks
            .retain(|target, _| graph.send_targets().contains(target));
        self.send_tracks.extend(staged_sends);
        Ok(())
    }

    fn validate_parameter_actions(&self, actions: &[GraphSyncAction]) -> Result<(), SoundError> {
        for action in actions {
            match action {
                GraphSyncAction::SetTrackVolume { track, .. }
                    if *track != SoundTrackId::master() && !self.tracks.contains_key(track) =>
                {
                    return Err(SoundError::UnknownTrack { track: *track });
                }
                GraphSyncAction::SetSendVolume { target, .. }
                    if !self.send_tracks.contains_key(target) =>
                {
                    return Err(SoundError::UnknownTrack { track: *target });
                }
                GraphSyncAction::SetTrackSendVolume { track, target, .. }
                    if !self.tracks.contains_key(track)
                        || !self.send_tracks.contains_key(target) =>
                {
                    return Err(SoundError::UnknownTrack {
                        track: if self.tracks.contains_key(track) {
                            *target
                        } else {
                            *track
                        },
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn stage_missing_send_tracks(
        &mut self,
        graph: &CompiledSoundGraph,
    ) -> Result<HashMap<SoundTrackId, kira::track::SendTrackHandle>, SoundError> {
        let mut staged: HashMap<SoundTrackId, kira::track::SendTrackHandle> = HashMap::new();
        for target in graph.send_targets() {
            if self.send_tracks.contains_key(target) {
                continue;
            }
            let volume = graph
                .send_target_linear_gain(*target)
                .map(super::super::super::graph_compile::linear_gain_to_decibels)
                .ok_or(SoundError::UnknownTrack { track: *target })?;
            let handle = self
                .manager_mut()?
                .add_send_track(SendTrackBuilder::new().volume(volume))
                .map_err(resource_limit_error)?;
            staged.insert(*target, handle);
        }
        Ok(staged)
    }

    fn stage_track_handles(
        &mut self,
        graph: &CompiledSoundGraph,
        stage_ids: &HashSet<SoundTrackId>,
        staged_sends: &HashMap<SoundTrackId, kira::track::SendTrackHandle>,
    ) -> Result<HashMap<SoundTrackId, kira::track::TrackHandle>, SoundError> {
        let mut staged: HashMap<SoundTrackId, kira::track::TrackHandle> = HashMap::new();
        let mut pending = graph
            .tracks()
            .iter()
            .filter(|track| stage_ids.contains(&track.id))
            .cloned()
            .collect::<Vec<CompiledTrack>>();
        while !pending.is_empty() {
            let next = pending.iter().position(|track| {
                track.parent == Some(SoundTrackId::master())
                    || track.parent.is_some_and(|parent| {
                        staged.contains_key(&parent)
                            || (!stage_ids.contains(&parent) && self.tracks.contains_key(&parent))
                    })
            });
            let Some(index) = next else {
                return Err(SoundError::InvalidMixerGraph(
                    "staged subtree parent order could not be compiled".to_string(),
                ));
            };
            let track = pending.remove(index);
            let builder = self.track_builder_with_staged_sends(&track, staged_sends)?;
            let parent = track.parent.ok_or_else(|| {
                SoundError::InvalidMixerGraph("non-master track must have a parent".to_string())
            })?;
            let handle = if parent == SoundTrackId::master() {
                self.manager_mut()?
                    .add_sub_track(builder)
                    .map_err(resource_limit_error)?
            } else if let Some(parent) = staged.get_mut(&parent) {
                parent
                    .add_sub_track(builder)
                    .map_err(resource_limit_error)?
            } else {
                self.tracks
                    .get_mut(&parent)
                    .ok_or(SoundError::UnknownTrack { track: parent })?
                    .add_sub_track(builder)
                    .map_err(resource_limit_error)?
            };
            staged.insert(track.id, handle);
        }
        Ok(staged)
    }

    fn track_builder_with_staged_sends(
        &self,
        track: &CompiledTrack,
        staged_sends: &HashMap<SoundTrackId, kira::track::SendTrackHandle>,
    ) -> Result<kira::track::TrackBuilder, SoundError> {
        let mut builder = kira::track::TrackBuilder::new()
            .volume(track.volume)
            .sound_capacity(self.physical_voice_capacity)
            .sub_track_capacity(track.child_capacity)
            .persist_until_sounds_finish(true);
        for send in &track.sends {
            let handle = staged_sends
                .get(&send.target)
                .or_else(|| self.send_tracks.get(&send.target))
                .ok_or(SoundError::UnknownTrack { track: send.target })?;
            builder = builder.with_send(
                handle.id(),
                super::super::super::graph_compile::linear_gain_to_decibels(send.gain),
            );
        }
        Ok(builder)
    }
}

fn subtree_ids(graph: &SoundMixerGraph, root: SoundTrackId) -> HashSet<SoundTrackId> {
    let parents = graph
        .tracks
        .iter()
        .map(|track| (track.id, track.parent))
        .collect::<HashMap<_, _>>();
    subtree_ids_from_parents(&parents, root)
}

fn subtree_ids_from_compiled(
    graph: &CompiledSoundGraph,
    root: SoundTrackId,
) -> HashSet<SoundTrackId> {
    let parents = graph
        .tracks()
        .iter()
        .map(|track| (track.id, track.parent))
        .collect::<HashMap<_, _>>();
    subtree_ids_from_parents(&parents, root)
}

fn subtree_ids_from_parents(
    parents: &HashMap<SoundTrackId, Option<SoundTrackId>>,
    root: SoundTrackId,
) -> HashSet<SoundTrackId> {
    let mut subtree = HashSet::from([root]);
    loop {
        let before = subtree.len();
        for (track, parent) in parents {
            if parent.is_some_and(|parent| subtree.contains(&parent)) {
                subtree.insert(*track);
            }
        }
        if subtree.len() == before {
            return subtree;
        }
    }
}
