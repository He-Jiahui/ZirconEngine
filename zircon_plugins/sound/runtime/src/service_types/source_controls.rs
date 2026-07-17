use kira::backend::Backend;
use zircon_runtime::core::framework::sound::{SoundError, SoundSourceId};

use crate::automation::values::ensure_finite_value;
use crate::engine::{SoundEngineState, SourceVoice};
use crate::kira_bridge::{DefaultKiraEngine, KiraEngine};

use super::playback_validation::validate_playback_speed;
use super::sources::sync_source_voice_in_state;
use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn pause_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        update_bound_source(&mut state, source, pause_bound_source)
    }

    pub(super) fn resume_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        let mut voice = take_source(&mut state, source)?;
        let result = resume_bound_source(&mut state.kira, &mut voice)
            .and_then(|_| sync_source_voice_in_state(&mut state, &mut voice));
        if result.is_err() {
            voice.descriptor.playing = false;
        }
        state.sources.insert(source, voice);
        result
    }

    pub(super) fn toggle_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        let mut voice = take_source(&mut state, source)?;
        let result = if voice.descriptor.playing {
            pause_bound_source(&mut state.kira, &mut voice)
        } else {
            resume_bound_source(&mut state.kira, &mut voice)
                .and_then(|_| sync_source_voice_in_state(&mut state, &mut voice))
        };
        if result.is_err() {
            voice.descriptor.playing = false;
        }
        state.sources.insert(source, voice);
        result
    }

    pub(super) fn set_source_gain_impl(
        &self,
        source: SoundSourceId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("source gain", gain)?;
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        update_bound_source(&mut state, source, |kira, voice| {
            set_bound_source_gain(kira, voice, gain)
        })
    }

    pub(super) fn set_source_speed_impl(
        &self,
        source: SoundSourceId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        update_bound_source(&mut state, source, |kira, voice| {
            set_bound_source_speed(kira, voice, speed)
        })
    }

    pub(super) fn mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        update_bound_source(&mut state, source, |kira, voice| {
            mute_bound_source(kira, voice, true)
        })
    }

    pub(super) fn unmute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        update_bound_source(&mut state, source, |kira, voice| {
            mute_bound_source(kira, voice, false)
        })
    }

    pub(super) fn toggle_mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        let muted = state
            .sources
            .get(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?
            .descriptor
            .muted;
        update_bound_source(&mut state, source, |kira, voice| {
            mute_bound_source(kira, voice, !muted)
        })
    }
}

pub(crate) fn pause_bound_source<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
) -> Result<(), SoundError> {
    if let Some(playback) = voice.kira_playback {
        kira.pause(playback)?;
    }
    voice.descriptor.playing = false;
    Ok(())
}

pub(crate) fn resume_bound_source<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
) -> Result<(), SoundError> {
    if let Some(playback) = voice.kira_playback {
        kira.resume(playback)?;
    }
    voice.descriptor.playing = true;
    Ok(())
}

pub(crate) fn set_bound_source_gain<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
    gain: f32,
) -> Result<(), SoundError> {
    if let Some(playback) = voice.kira_playback {
        kira.set_volume(playback, if voice.descriptor.muted { 0.0 } else { gain })?;
    }
    voice.descriptor.gain = gain;
    Ok(())
}

pub(crate) fn set_bound_source_speed<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
    speed: f32,
) -> Result<(), SoundError> {
    if let Some(playback) = voice.kira_playback {
        kira.set_playback_rate(playback, speed)?;
    }
    voice.descriptor.speed = speed;
    Ok(())
}

pub(crate) fn mute_bound_source<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
    muted: bool,
) -> Result<(), SoundError> {
    if let Some(playback) = voice.kira_playback {
        kira.set_volume(playback, if muted { 0.0 } else { voice.descriptor.gain })?;
    }
    voice.descriptor.muted = muted;
    Ok(())
}

fn update_bound_source(
    state: &mut SoundEngineState,
    source: SoundSourceId,
    update: impl FnOnce(&mut DefaultKiraEngine, &mut SourceVoice) -> Result<(), SoundError>,
) -> Result<(), SoundError> {
    let mut voice = take_source(state, source)?;
    let result = update(&mut state.kira, &mut voice);
    state.sources.insert(source, voice);
    result
}

fn take_source(
    state: &mut SoundEngineState,
    source: SoundSourceId,
) -> Result<SourceVoice, SoundError> {
    state
        .sources
        .remove(&source)
        .ok_or(SoundError::UnknownSource { source_id: source })
}
