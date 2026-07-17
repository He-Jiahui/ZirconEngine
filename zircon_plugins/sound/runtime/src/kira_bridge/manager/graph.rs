use std::collections::HashMap;
use std::fmt::Debug;

use kira::{
    backend::Backend,
    sound::PlaybackState,
    track::{SendTrackBuilder, TrackBuilder},
    Tween,
};
use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph, SoundTrackId};

use super::super::graph_compile::{
    compile_graph, compile_graph_update, CompiledSoundGraph, CompiledTrack, GraphSyncAction,
    GraphSyncPlan,
};
use super::KiraEngine;

mod transaction;

impl<B: Backend> KiraEngine<B> {
    pub(crate) fn sync_graph(&mut self, graph: &SoundMixerGraph) -> Result<(), SoundError> {
        let plan = match &self.graph {
            Some(previous) => compile_graph_update(previous, graph)?,
            None => {
                let compiled = compile_graph(graph)?;
                self.preflight_graph(&compiled)?;
                self.rebuild_graph(&compiled)?;
                self.graph = Some(graph.clone());
                return Ok(());
            }
        };
        self.apply_graph_update(graph, plan)
    }

    pub(crate) fn apply_graph_update(
        &mut self,
        graph: &SoundMixerGraph,
        plan: GraphSyncPlan,
    ) -> Result<(), SoundError> {
        self.preflight_graph(plan.compiled())?;
        let rebuilds_entire_graph = plan
            .diff()
            .actions()
            .iter()
            .any(|action| matches!(action, GraphSyncAction::RebuildGraph));
        let has_structural_change = plan.diff().actions().iter().any(|action| {
            matches!(
                action,
                GraphSyncAction::AddTrack { .. }
                    | GraphSyncAction::RemoveTrack { .. }
                    | GraphSyncAction::RebuildSubtree { .. }
                    | GraphSyncAction::RebuildGraph
            )
        });
        let has_active_playback = self
            .playbacks
            .values()
            .any(|playback| playback.state() != PlaybackState::Stopped);
        if has_structural_change && has_active_playback {
            return Err(SoundError::UnsupportedAdvancedFeature(
                "structural mixer graph edits require all active Kira playbacks to stop"
                    .to_string(),
            ));
        }
        if rebuilds_entire_graph {
            self.rebuild_graph(plan.compiled())?;
        } else if has_structural_change {
            self.apply_structural_plan(plan.compiled(), plan.diff().actions())?;
        } else {
            self.apply_parameter_plan(plan.diff().actions())?;
        }
        self.graph = Some(graph.clone());
        Ok(())
    }

    pub(crate) fn track_count(&self) -> usize {
        usize::from(self.is_active()) + self.tracks.len()
    }

    pub(crate) fn contains_track(&self, track: SoundTrackId) -> bool {
        (track == SoundTrackId::master() && self.is_active()) || self.tracks.contains_key(&track)
    }

    #[cfg(test)]
    pub(crate) fn installed_graph_for_test(&self) -> Option<&SoundMixerGraph> {
        self.graph.as_ref()
    }

    pub(crate) fn set_global_volume(&mut self, linear_gain: f32) -> Result<(), SoundError> {
        self.global_volume_gain = linear_gain;
        if !self.is_active() {
            return Ok(());
        }
        let graph_gain = self
            .graph
            .as_ref()
            .and_then(|graph| {
                graph
                    .tracks
                    .iter()
                    .find(|track| track.id == SoundTrackId::master())
            })
            .map(|track| {
                if track.controls.mute {
                    0.0
                } else {
                    track.controls.gain
                }
            })
            .unwrap_or(1.0);
        let volume = super::super::graph_compile::linear_gain_to_decibels(
            graph_gain * self.global_volume_gain,
        );
        self.manager_mut()?
            .main_track()
            .set_volume(volume, Tween::default());
        Ok(())
    }

    fn preflight_graph(&self, graph: &CompiledSoundGraph) -> Result<(), SoundError> {
        if graph.tracks().len() > self.logical_track_capacity {
            return Err(SoundError::BackendUnavailable {
                detail: format!(
                    "mixer graph has {} tracks but the configured limit is {}",
                    graph.tracks().len(),
                    self.logical_track_capacity
                ),
            });
        }
        if graph.send_targets().len() > self.logical_track_capacity {
            return Err(SoundError::BackendUnavailable {
                detail: format!(
                    "mixer graph has {} send targets but the configured limit is {}",
                    graph.send_targets().len(),
                    self.logical_track_capacity
                ),
            });
        }
        Ok(())
    }

    fn apply_parameter_plan(&mut self, actions: &[GraphSyncAction]) -> Result<(), SoundError> {
        for action in actions {
            match action {
                GraphSyncAction::SetTrackVolume {
                    track,
                    volume,
                    tween,
                    ..
                } => self.set_track_volume(*track, *volume, *tween)?,
                GraphSyncAction::SetSendVolume {
                    target,
                    volume,
                    tween,
                    ..
                } => self.set_send_volume(*target, *volume, *tween)?,
                GraphSyncAction::SetTrackSendVolume {
                    track,
                    target,
                    volume,
                    tween,
                    ..
                } => self.set_track_send_volume(*track, *target, *volume, *tween)?,
                GraphSyncAction::AddTrack { .. }
                | GraphSyncAction::RemoveTrack { .. }
                | GraphSyncAction::RebuildSubtree { .. }
                | GraphSyncAction::RebuildGraph => {}
            }
        }
        Ok(())
    }

    fn rebuild_graph(&mut self, graph: &CompiledSoundGraph) -> Result<(), SoundError> {
        let (tracks, send_tracks) = self.build_graph_handles(graph)?;
        self.set_compiled_master_volume(graph)?;
        self.tracks = tracks;
        self.send_tracks = send_tracks;
        Ok(())
    }

    fn build_graph_handles(
        &mut self,
        graph: &CompiledSoundGraph,
    ) -> Result<
        (
            HashMap<SoundTrackId, kira::track::TrackHandle>,
            HashMap<SoundTrackId, kira::track::SendTrackHandle>,
        ),
        SoundError,
    > {
        let mut send_tracks = HashMap::new();
        for target in graph.send_targets() {
            let volume = graph
                .send_target_linear_gain(*target)
                .map(super::super::graph_compile::linear_gain_to_decibels)
                .ok_or(SoundError::UnknownTrack { track: *target })?;
            let handle = self
                .manager_mut()?
                .add_send_track(SendTrackBuilder::new().volume(volume))
                .map_err(resource_limit_error)?;
            send_tracks.insert(*target, handle);
        }

        let mut tracks = HashMap::new();
        let mut pending = graph
            .tracks()
            .iter()
            .filter(|track| track.id != SoundTrackId::master())
            .cloned()
            .collect::<Vec<_>>();
        while !pending.is_empty() {
            let next = pending.iter().position(|track| {
                track.parent == Some(SoundTrackId::master())
                    || track
                        .parent
                        .is_some_and(|parent| tracks.contains_key(&parent))
            });
            let Some(index) = next else {
                return Err(SoundError::InvalidMixerGraph(
                    "track parent order could not be compiled".to_string(),
                ));
            };
            let track = pending.remove(index);
            let handle = self.build_track_handle(&track, &mut tracks, &send_tracks)?;
            tracks.insert(track.id, handle);
        }
        Ok((tracks, send_tracks))
    }

    fn build_track_handle(
        &mut self,
        track: &CompiledTrack,
        tracks: &mut HashMap<SoundTrackId, kira::track::TrackHandle>,
        send_tracks: &HashMap<SoundTrackId, kira::track::SendTrackHandle>,
    ) -> Result<kira::track::TrackHandle, SoundError> {
        let builder = self.track_builder(track, send_tracks)?;
        let parent = track.parent.ok_or_else(|| {
            SoundError::InvalidMixerGraph("non-master track must have a parent".to_string())
        })?;
        if parent == SoundTrackId::master() {
            self.manager_mut()?
                .add_sub_track(builder)
                .map_err(resource_limit_error)
        } else {
            tracks
                .get_mut(&parent)
                .ok_or_else(|| {
                    SoundError::InvalidMixerGraph("compiled parent track is missing".to_string())
                })?
                .add_sub_track(builder)
                .map_err(resource_limit_error)
        }
    }

    fn track_builder(
        &self,
        track: &CompiledTrack,
        send_tracks: &HashMap<SoundTrackId, kira::track::SendTrackHandle>,
    ) -> Result<TrackBuilder, SoundError> {
        let mut builder = TrackBuilder::new()
            .volume(track.volume)
            .sound_capacity(self.physical_voice_capacity)
            .sub_track_capacity(track.child_capacity)
            .persist_until_sounds_finish(true);
        for send in &track.sends {
            let handle = send_tracks
                .get(&send.target)
                .ok_or(SoundError::UnknownTrack { track: send.target })?;
            builder = builder.with_send(
                handle.id(),
                super::super::graph_compile::linear_gain_to_decibels(send.gain),
            );
        }
        Ok(builder)
    }

    fn set_compiled_master_volume(&mut self, graph: &CompiledSoundGraph) -> Result<(), SoundError> {
        if let Some(track) = graph
            .tracks()
            .iter()
            .find(|track| track.id == SoundTrackId::master())
        {
            let volume = super::super::graph_compile::linear_gain_to_decibels(
                decibels_to_linear(track.volume) * self.global_volume_gain,
            );
            self.manager_mut()?
                .main_track()
                .set_volume(volume, Tween::default());
        }
        Ok(())
    }

    fn set_track_volume(
        &mut self,
        track: SoundTrackId,
        volume: kira::Decibels,
        tween: Tween,
    ) -> Result<(), SoundError> {
        if track == SoundTrackId::master() {
            let volume = super::super::graph_compile::linear_gain_to_decibels(
                decibels_to_linear(volume) * self.global_volume_gain,
            );
            self.manager_mut()?.main_track().set_volume(volume, tween);
            return Ok(());
        }
        self.tracks
            .get_mut(&track)
            .ok_or(SoundError::UnknownTrack { track })?
            .set_volume(volume, tween);
        Ok(())
    }

    fn set_send_volume(
        &mut self,
        target: SoundTrackId,
        volume: kira::Decibels,
        tween: Tween,
    ) -> Result<(), SoundError> {
        self.send_tracks
            .get_mut(&target)
            .ok_or(SoundError::UnknownTrack { track: target })?
            .set_volume(volume, tween);
        Ok(())
    }

    fn set_track_send_volume(
        &mut self,
        track: SoundTrackId,
        target: SoundTrackId,
        volume: kira::Decibels,
        tween: Tween,
    ) -> Result<(), SoundError> {
        let send_id = self
            .send_tracks
            .get(&target)
            .ok_or(SoundError::UnknownTrack { track: target })?
            .id();
        self.tracks
            .get_mut(&track)
            .ok_or(SoundError::UnknownTrack { track })?
            .set_send(send_id, volume, tween)
            .map_err(|_| {
                SoundError::InvalidMixerGraph(format!(
                    "track {} has no compiled send route to {}",
                    track.raw(),
                    target.raw()
                ))
            })
    }
}

fn decibels_to_linear(volume: kira::Decibels) -> f32 {
    if volume == kira::Decibels::SILENCE {
        0.0
    } else {
        10.0_f32.powf(volume.0 / 20.0)
    }
}

fn resource_limit_error(error: impl Debug) -> SoundError {
    SoundError::BackendUnavailable {
        detail: format!("kira resource limit reached: {error:?}"),
    }
}
