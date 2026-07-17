use std::fmt::Debug;

use kira::{backend::Backend, track::MainTrackBuilder, AudioManager, AudioManagerSettings};
use zircon_runtime::core::framework::sound::{
    SoundError, SoundOutputDeviceDescriptor, SoundPlaybackId,
};

use super::KiraEngine;
use crate::SoundConfig;

const STAGED_GRAPH_GENERATIONS: usize = 3;

impl<B: Backend> KiraEngine<B> {
    pub(crate) fn is_active(&self) -> bool {
        self.manager.is_some()
    }

    pub(crate) fn activate(&mut self, settings: AudioManagerSettings<B>) -> Result<(), SoundError>
    where
        B::Error: Debug,
    {
        let _ = self.deactivate();
        self.logical_track_capacity = settings.capacities.sub_track_capacity.saturating_add(1);
        self.manager =
            Some(
                AudioManager::new(settings).map_err(|error| SoundError::BackendUnavailable {
                    detail: format!("kira backend activation failed: {error:?}"),
                })?,
            );
        Ok(())
    }

    pub(crate) fn activate_with_limits(
        &mut self,
        mut settings: AudioManagerSettings<B>,
        max_tracks: usize,
        max_voices: usize,
    ) -> Result<(), SoundError>
    where
        B::Error: Debug,
    {
        let max_tracks = max_tracks.max(1);
        let max_voices = max_voices.max(1);
        let staged_capacity = max_tracks.saturating_mul(STAGED_GRAPH_GENERATIONS);
        let staged_voice_capacity = max_voices.saturating_mul(STAGED_GRAPH_GENERATIONS);
        settings.capacities.sub_track_capacity = staged_capacity;
        settings.capacities.send_track_capacity = staged_capacity;
        settings.main_track_builder = MainTrackBuilder::new().sound_capacity(staged_voice_capacity);
        self.activate(settings)?;
        self.logical_track_capacity = max_tracks;
        self.logical_voice_capacity = max_voices;
        self.physical_voice_capacity = staged_voice_capacity;
        Ok(())
    }

    pub(crate) fn deactivate(&mut self) -> Vec<SoundPlaybackId> {
        let mut detached = self.playbacks.keys().copied().collect::<Vec<_>>();
        detached.sort_by_key(|playback| playback.raw());
        self.playbacks.clear();
        self.tracks.clear();
        self.send_tracks.clear();
        self.graph = None;
        self.manager = None;
        detached
    }

    pub(super) fn manager_mut(&mut self) -> Result<&mut AudioManager<B>, SoundError> {
        self.manager
            .as_mut()
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: "kira audio engine is inactive".to_string(),
            })
    }

    #[cfg(test)]
    pub(crate) fn with_backend_mut<T>(
        &mut self,
        operation: impl FnOnce(&mut B) -> T,
    ) -> Result<T, SoundError> {
        Ok(operation(self.manager_mut()?.backend_mut()))
    }

    #[cfg(test)]
    pub(crate) fn set_logical_capacities_for_test(&mut self, max_tracks: usize, max_voices: usize) {
        self.logical_track_capacity = max_tracks;
        self.logical_voice_capacity = max_voices;
    }
}

impl KiraEngine<kira::DefaultBackend> {
    pub(crate) fn activate_output(
        &mut self,
        descriptor: &SoundOutputDeviceDescriptor,
        config: &SoundConfig,
    ) -> Result<(), SoundError> {
        let backend_settings = super::super::device::backend_settings(descriptor)?;
        let settings = AudioManagerSettings {
            backend_settings,
            internal_buffer_size: config.block_size_frames.max(1),
            ..AudioManagerSettings::default()
        };
        self.global_volume_gain = config.master_gain;
        self.activate_with_limits(settings, config.max_tracks, config.max_voices)?;
        self.set_global_volume(config.master_gain)
    }
}
