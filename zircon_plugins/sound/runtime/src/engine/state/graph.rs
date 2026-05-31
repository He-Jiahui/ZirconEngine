use zircon_runtime::core::framework::sound::{SoundError, SoundTrackDescriptor, SoundTrackId};

use super::SoundEngineState;

impl SoundEngineState {
    pub(crate) fn add_or_replace_track(
        &mut self,
        track: SoundTrackDescriptor,
    ) -> Result<(), SoundError> {
        if track.id == SoundTrackId::master() && track.parent.is_some() {
            return Err(SoundError::InvalidMixerGraph(
                "master track cannot have a parent track".to_string(),
            ));
        }
        let mut graph = self.graph.clone();
        if let Some(existing) = graph
            .tracks
            .iter_mut()
            .find(|existing| existing.id == track.id)
        {
            *existing = track;
        } else {
            graph.tracks.push(track);
        }
        super::super::validation::validate_graph(&graph)?;
        self.graph = graph;
        Ok(())
    }

    pub(crate) fn remove_track(&mut self, track: SoundTrackId) -> Result<(), SoundError> {
        if track == SoundTrackId::master() {
            return Err(SoundError::InvalidMixerGraph(
                "master track cannot be removed".to_string(),
            ));
        }
        let mut graph = self.graph.clone();
        let before = graph.tracks.len();
        graph.tracks.retain(|existing| existing.id != track);
        if before == graph.tracks.len() {
            return Err(SoundError::UnknownTrack { track });
        }
        super::super::validation::validate_graph(&graph)?;
        self.graph = graph;
        for playback in self.playbacks.values_mut() {
            if playback.output_track == track {
                playback.output_track = SoundTrackId::master();
            }
        }
        for source in self.sources.values_mut() {
            if source.descriptor.output_track == track {
                source.descriptor.output_track = SoundTrackId::master();
            }
        }
        Ok(())
    }
}
