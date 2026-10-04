//! Kira 重启废弃旧播放与图句柄，容量按并行图代际预留；服务层记录一次性播放停止，并保留持续声源描述供下次启动重绑。
#[cfg(test)]
use std::cell::Cell;
use std::fmt::Debug;
use std::time::Duration;

use kira::{
    backend::Backend, track::MainTrackBuilder, AudioManager, AudioManagerSettings, Decibels,
};
use zircon_runtime::core::framework::sound::{
    SoundError, SoundOutputDeviceDescriptor, SoundPlaybackId,
};

use super::super::cpal_owner::OwnerCpalBackend;
use super::KiraEngine;
use crate::SoundConfig;

const STAGED_GRAPH_GENERATIONS: usize = 3;

pub(crate) struct PreparedKiraCommit<B: Backend> {
    replacement: KiraEngine<B>,
    detached: Vec<SoundPlaybackId>,
}

impl<B: Backend> PreparedKiraCommit<B> {
    pub(crate) fn detached(&self) -> &[SoundPlaybackId] {
        &self.detached
    }
}

#[cfg(test)]
thread_local! {
    static FAIL_NEXT_RETIREMENT_RESERVATION: Cell<bool> = const { Cell::new(false) };
}

impl<B: Backend> KiraEngine<B> {
    pub(crate) fn is_active(&self) -> bool {
        self.manager.is_some() && !self.provider_retiring
    }

    /// Reject ordinary consumer mutations while a provider retirement is pending.
    /// The manager and its handles stay installed so the owner can retry the
    /// same provider worker; callers must not enqueue new Kira work or mutate
    /// service metadata during that interval.
    pub(crate) fn ensure_control_available(&self) -> Result<(), SoundError> {
        if self.manager.is_none() || self.provider_retiring {
            return Err(Self::inactive_control_error());
        }
        Ok(())
    }

    /// Source descriptors remain editable before the first output is active,
    /// but become read-only while a real provider worker is still retiring.
    pub(crate) fn ensure_provider_not_retiring(&self) -> Result<(), SoundError> {
        if self.provider_retiring {
            return Err(Self::inactive_control_error());
        }
        Ok(())
    }

    fn inactive_control_error() -> SoundError {
        SoundError::BackendUnavailable {
            detail: "kira audio engine is inactive".to_string(),
        }
    }

    pub(crate) fn activate(&mut self, settings: AudioManagerSettings<B>) -> Result<(), SoundError>
    where
        B::Error: Debug,
    {
        let prepared = Self::prepare_activation(settings)?;
        let commit = self.prepare_commit(prepared)?;
        let (retired, _) = self.commit_prepared(commit);
        drop(retired);
        Ok(())
    }

    fn prepare_activation(settings: AudioManagerSettings<B>) -> Result<Self, SoundError>
    where
        B::Error: Debug,
    {
        let physical_sub_track_capacity = settings.capacities.sub_track_capacity;
        let physical_send_track_capacity = settings.capacities.send_track_capacity;
        let manager =
            AudioManager::new(settings).map_err(|error| SoundError::BackendUnavailable {
                detail: format!("kira backend activation failed: {error:?}"),
            })?;
        let mut prepared = Self::inactive();
        prepared.manager = Some(manager);
        prepared.logical_track_capacity = physical_sub_track_capacity.saturating_add(1);
        prepared.physical_sub_track_capacity = physical_sub_track_capacity;
        prepared.physical_send_track_capacity = physical_send_track_capacity;
        Ok(prepared)
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
        let mut prepared = Self::prepare_activation(settings)?;
        prepared.logical_track_capacity = max_tracks;
        prepared.logical_voice_capacity = max_voices;
        prepared.physical_voice_capacity = staged_voice_capacity;
        let commit = self.prepare_commit(prepared)?;
        let (retired, _) = self.commit_prepared(commit);
        drop(retired);
        Ok(())
    }

    pub(crate) fn prepare_with_limits(
        mut settings: AudioManagerSettings<B>,
        max_tracks: usize,
        max_voices: usize,
    ) -> Result<Self, SoundError>
    where
        B::Error: Debug,
    {
        let max_tracks = max_tracks.max(1);
        let max_voices = max_voices.max(1);
        let staged_capacity = max_tracks.saturating_mul(STAGED_GRAPH_GENERATIONS);
        let staged_voice_capacity = max_voices.saturating_mul(STAGED_GRAPH_GENERATIONS);
        settings.capacities.sub_track_capacity = staged_capacity;
        settings.capacities.send_track_capacity = staged_capacity;
        settings.main_track_builder = MainTrackBuilder::new()
            .volume(Decibels::SILENCE)
            .sound_capacity(staged_voice_capacity);
        let mut prepared = Self::prepare_activation(settings)?;
        prepared.logical_track_capacity = max_tracks;
        prepared.logical_voice_capacity = max_voices;
        prepared.physical_voice_capacity = staged_voice_capacity;
        Ok(prepared)
    }

    /// Reserve and order every detached id while the current generation is still installed.
    /// Failure drops only the replacement; the active manager and its voices remain untouched.
    pub(crate) fn prepare_commit(
        &self,
        replacement: Self,
    ) -> Result<PreparedKiraCommit<B>, SoundError> {
        let mut detached = Vec::new();
        reserve_detached_ids(&mut detached, self.playbacks.len())?;
        detached.extend(self.playbacks.keys().copied());
        detached.sort_by_key(|playback| playback.raw());
        Ok(PreparedKiraCommit {
            replacement,
            detached,
        })
    }

    /// Install a fully prepared generation. This operation performs no collection, sorting,
    /// or capacity growth after the visible replacement. The retired generation is returned so
    /// an owner can drop it after releasing its state lock.
    pub(crate) fn commit_prepared(
        &mut self,
        prepared: PreparedKiraCommit<B>,
    ) -> (Self, Vec<SoundPlaybackId>) {
        let PreparedKiraCommit {
            replacement,
            detached,
        } = prepared;
        let old = std::mem::replace(self, replacement);
        (old, detached)
    }

    pub(crate) fn deactivate(&mut self) -> Vec<SoundPlaybackId> {
        let mut detached = self.playbacks.keys().copied().collect::<Vec<_>>();
        detached.sort_by_key(|playback| playback.raw());
        self.playbacks.clear();
        self.tracks.clear();
        self.send_tracks.clear();
        self.graph = None;
        self.manager = None;
        self.provider_retiring = false;
        detached
    }

    pub(super) fn manager_mut(&mut self) -> Result<&mut AudioManager<B>, SoundError> {
        self.ensure_control_available()?;
        self.manager
            .as_mut()
            .ok_or_else(Self::inactive_control_error)
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

    #[cfg(test)]
    pub(crate) fn mark_provider_retiring_for_test(&mut self) {
        self.provider_retiring = true;
    }
}

fn reserve_detached_ids(
    detached: &mut Vec<SoundPlaybackId>,
    count: usize,
) -> Result<(), SoundError> {
    #[cfg(test)]
    if FAIL_NEXT_RETIREMENT_RESERVATION.with(|fail| fail.replace(false)) {
        return Err(SoundError::BackendUnavailable {
            detail: "injected retired-playback reservation failure".to_string(),
        });
    }

    detached
        .try_reserve_exact(count)
        .map_err(|error| SoundError::BackendUnavailable {
            detail: format!("unable to reserve retired playback ids: {error}"),
        })
}

#[cfg(test)]
pub(crate) fn fail_next_retirement_reservation_for_test() {
    FAIL_NEXT_RETIREMENT_RESERVATION.with(|fail| fail.set(true));
}

impl KiraEngine<OwnerCpalBackend> {
    const CPAL_QUIESCENCE_DEADLINE: Duration = Duration::from_secs(1);

    /// Stop admitting callbacks before replacement or stop becomes visible. A callback timeout
    /// leaves the old stream installed and resumes admission; a provider timeout retains the
    /// same provider worker and join authority for a later retry. Both paths preserve the
    /// last-good state until a complete retirement can be committed.
    pub(crate) fn quiesce_before_replace(&mut self) -> Result<(), SoundError> {
        let result = self.manager.as_mut().map(|manager| {
            manager
                .backend_mut()
                .quiesce(Self::CPAL_QUIESCENCE_DEADLINE)
        });
        match result {
            None | Some(Ok(())) => {
                self.provider_retiring = false;
            }
            Some(Err(error)) => {
                // The provider worker owns the real stream after the callback
                // gate closes. A deadline therefore makes this generation
                // unavailable to consumers while retaining the manager and
                // pending join authority for the next transition retry. No
                // stream is dropped under a state or owner lock.
                let detail = if error.is_provider_phase() {
                    let outcome = error.retiring_outcome_unknown();
                    format!(
                        "{error}; RetiringOutcomeUnknown census={:?}",
                        outcome.census
                    )
                } else {
                    error.to_string()
                };
                self.provider_retiring = error.is_provider_phase();
                return Err(SoundError::BackendUnavailable { detail });
            }
        }
        Ok(())
    }

    /// Build a muted replacement generation. CPAL enumeration, exact format admission, Kira
    /// setup and stream start all happen before the caller can commit this value.
    pub(crate) fn prepare_output(
        descriptor: &SoundOutputDeviceDescriptor,
        config: &SoundConfig,
    ) -> Result<Self, SoundError> {
        let backend_settings = super::super::device::backend_settings(descriptor)?;
        let settings = AudioManagerSettings {
            backend_settings,
            internal_buffer_size: config.block_size_frames.max(1),
            ..AudioManagerSettings::default()
        };
        let mut prepared =
            Self::prepare_with_limits(settings, config.max_tracks, config.max_voices)?;
        prepared.set_global_volume(0.0)?;
        Ok(prepared)
    }

    pub(crate) fn activate_output(
        &mut self,
        descriptor: &SoundOutputDeviceDescriptor,
        config: &SoundConfig,
    ) -> Result<(), SoundError> {
        let prepared = Self::prepare_output(descriptor, config)?;
        let commit = self.prepare_commit(prepared)?;
        self.quiesce_before_replace()?;
        let (retired, _) = self.commit_prepared(commit);
        drop(retired);
        self.apply_global_volume_after_commit(config.master_gain);
        Ok(())
    }
}
