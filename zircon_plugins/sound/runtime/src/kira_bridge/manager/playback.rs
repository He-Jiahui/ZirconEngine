use kira::{
    backend::Backend,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    sound::PlaybackState,
    Tween,
};
use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId, SoundTrackId};

use super::KiraEngine;

impl<B: Backend> KiraEngine<B> {
    pub(crate) fn play(
        &mut self,
        playback: SoundPlaybackId,
        track: SoundTrackId,
        data: StaticSoundData,
    ) -> Result<(), SoundError> {
        if self.playbacks.len() >= self.logical_voice_capacity {
            return Err(SoundError::BackendUnavailable {
                detail: format!(
                    "kira voice capacity {} is exhausted",
                    self.logical_voice_capacity
                ),
            });
        }
        let handle = if track == SoundTrackId::master() {
            self.manager_mut()?.main_track().play(data)
        } else {
            self.tracks
                .get_mut(&track)
                .ok_or(SoundError::UnknownTrack { track })?
                .play(data)
        }
        .map_err(|error| SoundError::BackendUnavailable {
            detail: format!("kira playback allocation failed: {error:?}"),
        })?;
        self.playbacks.insert(playback, handle);
        Ok(())
    }

    pub(crate) fn contains_playback(&self, playback: SoundPlaybackId) -> bool {
        self.playbacks.contains_key(&playback)
    }

    pub(crate) fn playback_state(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<PlaybackState, SoundError> {
        self.playbacks
            .get(&playback)
            .map(StaticSoundHandle::state)
            .ok_or(SoundError::UnknownPlayback { playback })
    }

    pub(crate) fn drain_finished_playbacks(&mut self) -> Vec<SoundPlaybackId> {
        let mut finished = self
            .playbacks
            .iter()
            .filter_map(|(playback, handle)| {
                (handle.state() == PlaybackState::Stopped).then_some(*playback)
            })
            .collect::<Vec<_>>();
        finished.sort_by_key(|playback| playback.raw());
        for playback in &finished {
            self.playbacks.remove(playback);
        }
        finished
    }

    pub(crate) fn pause(&mut self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.playback_mut(playback)?.pause(Tween::default());
        Ok(())
    }

    pub(crate) fn resume(&mut self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.playback_mut(playback)?.resume(Tween::default());
        Ok(())
    }

    pub(crate) fn seek_to(
        &mut self,
        playback: SoundPlaybackId,
        seconds: f64,
    ) -> Result<(), SoundError> {
        self.playback_mut(playback)?.seek_to(seconds);
        Ok(())
    }

    pub(crate) fn set_volume(
        &mut self,
        playback: SoundPlaybackId,
        linear_gain: f32,
    ) -> Result<(), SoundError> {
        self.playback_mut(playback)?.set_volume(
            super::super::graph_compile::linear_gain_to_decibels(linear_gain),
            Tween::default(),
        );
        Ok(())
    }

    pub(crate) fn set_playback_rate(
        &mut self,
        playback: SoundPlaybackId,
        playback_rate: f32,
    ) -> Result<(), SoundError> {
        self.playback_mut(playback)?
            .set_playback_rate(playback_rate as f64, Tween::default());
        Ok(())
    }

    pub(crate) fn playback_position(&self, playback: SoundPlaybackId) -> Result<f64, SoundError> {
        self.playbacks
            .get(&playback)
            .map(StaticSoundHandle::position)
            .ok_or(SoundError::UnknownPlayback { playback })
    }

    pub(crate) fn stop(&mut self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut handle = self
            .playbacks
            .remove(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        handle.stop(Tween::default());
        Ok(())
    }

    fn playback_mut(
        &mut self,
        playback: SoundPlaybackId,
    ) -> Result<&mut StaticSoundHandle, SoundError> {
        self.playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })
    }
}
