//! Output transitions prepare CPAL, Kira, graph and source handles outside the owner lock, then
//! fence and commit one generation while the previous generation is still available.
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(test)]
use std::sync::{
    mpsc::{Receiver, SyncSender},
    Mutex, OnceLock,
};

use zircon_runtime::core::framework::sound::{
    SoundError, SoundMixerGraph, SoundOutputDeviceDescriptor, SoundPlaybackId,
};

use super::super::DefaultSoundManager;
use crate::engine::{ActivePlayback, LoadedClip, SoundEngineState, SourceVoice};
use crate::kira_bridge::DefaultKiraEngine;
use crate::output::{validate_backend_supported, validate_output_device_descriptor};
use crate::poison_recovery::lock_recover;
use crate::service_types::sources::prepare_source_generation;
use crate::SoundConfig;

struct OutputGenerationSnapshot {
    config: SoundConfig,
    descriptor: SoundOutputDeviceDescriptor,
    graph: Arc<SoundMixerGraph>,
    graph_revision: u64,
    output_generation: u64,
    clips: HashMap<zircon_runtime::core::framework::sound::SoundClipId, LoadedClip>,
    sources: HashMap<zircon_runtime::core::framework::sound::SoundSourceId, SourceVoice>,
    playbacks: HashMap<SoundPlaybackId, ActivePlayback>,
    next_playback_id: u64,
    next_clip_id: u64,
    next_source_id: u64,
}

struct PreparedOutputGeneration {
    kira: DefaultKiraEngine,
    sources: HashMap<zircon_runtime::core::framework::sound::SoundSourceId, SourceVoice>,
    next_playback_id: u64,
}

#[cfg(test)]
type OwnerCommitBarrier = (SyncSender<()>, Receiver<()>);

#[cfg(test)]
static OWNER_COMMIT_BARRIER: OnceLock<Mutex<Option<OwnerCommitBarrier>>> = OnceLock::new();

#[cfg(test)]
type OwnerConfigAdmissionBarrier = (SyncSender<()>, Receiver<()>);

#[cfg(test)]
static OWNER_CONFIG_ADMISSION_BARRIER: OnceLock<Mutex<Option<OwnerConfigAdmissionBarrier>>> =
    OnceLock::new();

#[cfg(test)]
type OwnerGainWriterAdmissionBarrier = (SyncSender<()>, Receiver<()>, SyncSender<()>);

#[cfg(test)]
static OWNER_GAIN_WRITER_ADMISSION_BARRIER: OnceLock<
    Mutex<Option<OwnerGainWriterAdmissionBarrier>>,
> = OnceLock::new();

#[cfg(test)]
static OWNER_GAIN_WRITER_ADMITTED: OnceLock<Mutex<Option<SyncSender<()>>>> = OnceLock::new();

#[cfg(test)]
static OWNER_TEST_SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
static OWNER_RETIRED_DROP_BARRIER: OnceLock<Mutex<Option<OwnerConfigAdmissionBarrier>>> =
    OnceLock::new();

impl DefaultSoundManager {
    pub(in crate::service_types) fn start_output_device_impl(&self) -> Result<(), SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        let snapshot = {
            let mut state = lock_recover(&self.state);
            let descriptor = state.output_device.descriptor().clone();
            validate_output_device_descriptor(&descriptor)?;
            validate_backend_supported(&descriptor)?;
            if state.kira.is_active() {
                state.output_device.mark_started();
                return Ok(());
            }
            output_generation_snapshot(&state, &config, descriptor)
        };

        let prepared = match prepare_output_generation(&snapshot) {
            Ok(prepared) => prepared,
            Err(error) => {
                let mut state = lock_recover(&self.state);
                if state.kira.is_active() || !transition_snapshot_matches(&state, &snapshot) {
                    return Err(superseded_transition_error());
                }
                record_start_failure(&mut state, &snapshot.descriptor.backend, &error);
                return Err(error);
            }
        };

        self.commit_prepared_output_generation(snapshot, prepared)
    }

    fn commit_prepared_output_generation(
        &self,
        snapshot: OutputGenerationSnapshot,
        prepared: PreparedOutputGeneration,
    ) -> Result<(), SoundError> {
        // Configuration is held through the state fence and visible commit. CPAL/Kira setup has
        // already completed; only the prepared ownership swap runs under the locks, closing the
        // config -> state TOCTOU without extending the fallible preparation window.
        let current_config = lock_recover(&self.config);
        #[cfg(test)]
        wait_for_owner_commit_admission();
        let retired = {
            let mut state = lock_recover(&self.state);
            if *current_config != snapshot.config || !transition_snapshot_matches(&state, &snapshot)
            {
                return Err(superseded_transition_error());
            }

            let PreparedOutputGeneration {
                kira,
                sources,
                next_playback_id,
            } = prepared;
            let prepared_commit = state.kira.prepare_commit(kira)?;
            let prepared_retirement =
                state.prepare_kira_deactivation(prepared_commit.detached())?;
            // The state fence may wait for the provider census, but the actual
            // cpal::Stream::drop runs on OwnerCpalBackend's retirement worker
            // outside this state/config owner lock. A timeout retains that
            // worker and rejects publication without dropping the owner here.
            state.kira.quiesce_before_replace()?;
            let (retired, detached) = state.kira.commit_prepared(prepared_commit);
            state.reconcile_prepared_kira_deactivation(detached, prepared_retirement);
            state.sources = sources;
            state.next_playback_id = next_playback_id;
            state
                .kira
                .apply_global_volume_after_commit(current_config.master_gain);
            state.output_device.mark_started();
            state.output_generation = state.output_generation.wrapping_add(1);
            retired
        };
        drop(current_config);
        // Retire the old Kira manager only after the state owner lock is released. All fallible
        // reservation and retirement-record preparation happened before the visible replacement.
        retire_after_owner_unlock(retired);
        Ok(())
    }

    pub(in crate::service_types) fn stop_output_device_impl(&self) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        // Provider Drop itself is worker-owned; this lock only records the
        // completed census or retains the pending join authority on timeout.
        state.kira.quiesce_before_replace()?;
        state.deactivate_kira();
        state.output_device.stop();
        state.output_generation = state.output_generation.wrapping_add(1);
        Ok(())
    }
}

fn output_generation_snapshot(
    state: &SoundEngineState,
    config: &SoundConfig,
    descriptor: SoundOutputDeviceDescriptor,
) -> OutputGenerationSnapshot {
    OutputGenerationSnapshot {
        config: config.clone(),
        descriptor,
        graph: Arc::clone(&state.graph),
        graph_revision: state.graph_revision,
        output_generation: state.output_generation,
        clips: state.clips.clone(),
        sources: state.sources.clone(),
        playbacks: state.playbacks.clone(),
        next_playback_id: state.next_playback_id,
        next_clip_id: state.next_clip_id,
        next_source_id: state.next_source_id,
    }
}

fn prepare_output_generation(
    snapshot: &OutputGenerationSnapshot,
) -> Result<PreparedOutputGeneration, SoundError> {
    let mut kira = DefaultKiraEngine::prepare_output(&snapshot.descriptor, &snapshot.config)?;
    kira.sync_graph(&snapshot.graph)?;
    let mut sources = snapshot.sources.clone();
    let mut next_playback_id = snapshot.next_playback_id;
    prepare_source_generation(
        &mut kira,
        &mut next_playback_id,
        &snapshot.clips,
        &mut sources,
    )?;
    // Source allocation happened while the replacement generation was muted. Keep it silent
    // until the state swap; commit then applies the requested gain without a fallible operation.
    kira.set_global_volume(0.0)?;
    Ok(PreparedOutputGeneration {
        kira,
        sources,
        next_playback_id,
    })
}

fn transition_snapshot_matches(
    state: &SoundEngineState,
    snapshot: &OutputGenerationSnapshot,
) -> bool {
    state.output_generation == snapshot.output_generation
        && state.graph_revision == snapshot.graph_revision
        && state.graph.as_ref() == snapshot.graph.as_ref()
        && state.output_device.descriptor() == &snapshot.descriptor
        && state.clips.len() == snapshot.clips.len()
        && state
            .clips
            .keys()
            .all(|clip| snapshot.clips.contains_key(clip))
        && state.sources == snapshot.sources
        && state.playbacks == snapshot.playbacks
        && state.next_playback_id == snapshot.next_playback_id
        && state.next_clip_id == snapshot.next_clip_id
        && state.next_source_id == snapshot.next_source_id
}

fn superseded_transition_error() -> SoundError {
    SoundError::BackendUnavailable {
        detail: "sound output transition was superseded before commit".to_string(),
    }
}

fn record_start_failure(state: &mut SoundEngineState, backend: &str, error: &SoundError) {
    state
        .output_device
        .record_backend_unavailable(backend.to_string(), error.to_string());
}

#[cfg(test)]
#[path = "tests/lifecycle.rs"]
mod tests;

#[cfg(test)]
fn install_owner_commit_barrier(entered: SyncSender<()>, release: Receiver<()>) {
    let slot = OWNER_COMMIT_BARRIER.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap() = Some((entered, release));
}

#[cfg(test)]
pub(crate) fn install_owner_config_admission_barrier(
    entered: SyncSender<()>,
    release: Receiver<()>,
) {
    let slot = OWNER_CONFIG_ADMISSION_BARRIER.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap() = Some((entered, release));
}

#[cfg(test)]
pub(crate) fn wait_for_owner_config_admission() {
    let barrier = OWNER_CONFIG_ADMISSION_BARRIER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take();
    if let Some((entered, release)) = barrier {
        entered.send(()).unwrap();
        release.recv().unwrap();
    }
}

#[cfg(test)]
pub(crate) fn install_owner_gain_writer_admission_barrier(
    entered: SyncSender<()>,
    proceed: Receiver<()>,
    acquired: SyncSender<()>,
) {
    let slot = OWNER_GAIN_WRITER_ADMISSION_BARRIER.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap() = Some((entered, proceed, acquired));
}

#[cfg(test)]
pub(crate) fn wait_for_owner_gain_writer_admission_attempt() {
    let Some((entered, proceed, acquired)) = OWNER_GAIN_WRITER_ADMISSION_BARRIER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take()
    else {
        return;
    };
    entered.send(()).unwrap();
    proceed.recv().unwrap();
    *OWNER_GAIN_WRITER_ADMITTED
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap() = Some(acquired);
}

#[cfg(test)]
pub(crate) fn signal_owner_gain_writer_admitted() {
    if let Some(acquired) = OWNER_GAIN_WRITER_ADMITTED
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take()
    {
        acquired.send(()).unwrap();
    }
}

#[cfg(test)]
pub(crate) fn install_owner_retired_drop_barrier(entered: SyncSender<()>, release: Receiver<()>) {
    let slot = OWNER_RETIRED_DROP_BARRIER.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap() = Some((entered, release));
}

fn retire_after_owner_unlock(retired: DefaultKiraEngine) {
    #[cfg(test)]
    if let Some((entered, release)) = OWNER_RETIRED_DROP_BARRIER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take()
    {
        entered.send(()).unwrap();
        release.recv().unwrap();
    }
    drop(retired);
}

#[cfg(test)]
fn owner_test_serial() -> std::sync::MutexGuard<'static, ()> {
    OWNER_TEST_SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

#[cfg(test)]
fn wait_for_owner_commit_admission() {
    let barrier = OWNER_COMMIT_BARRIER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take();
    if let Some((entered, release)) = barrier {
        entered.send(()).unwrap();
        release.recv().unwrap();
    }
}
