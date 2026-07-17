use std::collections::HashMap;

use super::DefaultSoundManager;
use kira::backend::Backend;

use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundError, SoundGameplayEmission, SoundGameplayEmissionRead, SoundPlaybackId,
    SoundPlaybackSettings, SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished,
    SoundSourceId, SoundSourceInput, SOUND_GAMEPLAY_EMISSION_CAPACITY,
};

use crate::descriptor_validation::source::validate_source_descriptor;
use crate::engine::{LoadedClip, SoundEngineState, SourceVoice};
use crate::kira_bridge::{static_sound_data, KiraEngine};

use super::playback_validation::playback_range_for_settings;

impl DefaultSoundManager {
    pub(super) fn create_source_impl(
        &self,
        mut source: SoundSourceDescriptor,
    ) -> Result<SoundSourceId, SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        validate_source_descriptor(&state, &source)?;
        validate_source_runtime_surface(&source)?;
        if !state
            .graph
            .tracks
            .iter()
            .any(|track| track.id == source.output_track)
        {
            return Err(SoundError::UnknownTrack {
                track: source.output_track,
            });
        }
        let source_id = source.id.unwrap_or_else(|| state.next_source_id());
        source.id = Some(source_id);
        let gameplay_emission = source.gameplay_emitter.filter(|_| {
            source.playing && !source.muted && source.gain.is_finite() && source.gain > 0.0
        });
        let mut voice = SourceVoice::new(source);
        sync_source_voice_in_state(&mut state, &mut voice)?;
        if let Some(gameplay_emitter) = gameplay_emission {
            let emitted_at_seconds = self
                .core
                .as_ref()
                .and_then(|core| core.upgrade())
                .map(|core| core.real_time().elapsed_secs_f64())
                .unwrap_or_default();
            let journal = state
                .gameplay_emissions
                .entry(gameplay_emitter.world)
                .or_default();
            if journal.events.len() == SOUND_GAMEPLAY_EMISSION_CAPACITY {
                journal.events.pop_front();
            }
            journal.next_sequence = journal.next_sequence.saturating_add(1);
            let sequence = journal.next_sequence;
            journal.events.push_back(SoundGameplayEmission {
                sequence,
                world: gameplay_emitter.world,
                source: gameplay_emitter.entity,
                position: voice.descriptor.position,
                strength: voice.descriptor.gain,
                max_range: (voice.descriptor.spatial.max_distance.is_finite()
                    && voice.descriptor.spatial.max_distance > 0.0)
                    .then_some(voice.descriptor.spatial.max_distance),
                emitted_at_seconds,
            });
        }
        state.sources.insert(source_id, voice);
        Ok(source_id)
    }

    pub(super) fn read_gameplay_emissions_impl(
        &self,
        world: WorldHandle,
        after_sequence: Option<u64>,
    ) -> Result<SoundGameplayEmissionRead, SoundError> {
        let state = crate::poison_recovery::lock_recover(&self.state);
        let Some(journal) = state.gameplay_emissions.get(&world) else {
            return Ok(SoundGameplayEmissionRead::default());
        };
        let next_sequence = journal.next_sequence;
        let Some(after_sequence) = after_sequence else {
            return Ok(SoundGameplayEmissionRead {
                next_sequence,
                ..SoundGameplayEmissionRead::default()
            });
        };
        let available_after = journal
            .events
            .front()
            .map(|event| event.sequence.saturating_sub(1))
            .unwrap_or(next_sequence);
        let effective_after = after_sequence.max(available_after);
        let events = journal
            .events
            .iter()
            .filter(|event| event.sequence > effective_after)
            .cloned()
            .collect();
        Ok(SoundGameplayEmissionRead {
            events,
            next_sequence,
            missed_events: available_after.saturating_sub(after_sequence),
        })
    }

    pub(super) fn update_source_impl(
        &self,
        source: SoundSourceDescriptor,
    ) -> Result<(), SoundError> {
        let source_id = source.id.ok_or_else(|| {
            SoundError::InvalidParameter("source update requires a source id".to_string())
        })?;
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        validate_source_descriptor(&state, &source)?;
        validate_source_runtime_surface(&source)?;
        let mut voice = state
            .sources
            .remove(&source_id)
            .ok_or(SoundError::UnknownSource { source_id })?;
        stop_bound_source(&mut state.kira, &mut voice)?;
        voice.descriptor = source;
        let result = sync_source_voice_in_state(&mut state, &mut voice);
        state.sources.insert(source_id, voice);
        result
    }

    pub(super) fn remove_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        let mut voice = state
            .sources
            .remove(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        stop_bound_source(&mut state.kira, &mut voice)
    }

    pub(super) fn stop_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.poll_kira_completions();
        let mut voice = state
            .sources
            .remove(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        stop_bound_source(&mut state.kira, &mut voice)?;
        let descriptor = voice.descriptor;
        let input = descriptor.input;
        let clip = match input {
            SoundSourceInput::Clip(clip) => Some(clip),
            SoundSourceInput::External(_)
            | SoundSourceInput::SynthParameter { .. }
            | SoundSourceInput::Silence => None,
        };
        state.finished_sources.push(SoundSourceFinished {
            source,
            input,
            clip,
            reason: SoundSourceFinishReason::Stopped,
            completion_action: descriptor.completion_action,
            output_track: descriptor.output_track,
        });
        Ok(())
    }
}

pub(crate) fn sync_preconfigured_sources(state: &mut SoundEngineState) -> Result<(), SoundError> {
    let sources = state.sources.keys().copied().collect::<Vec<_>>();
    for source in sources {
        let mut voice = state
            .sources
            .remove(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        let result = sync_source_voice_in_state(state, &mut voice);
        state.sources.insert(source, voice);
        result?;
    }
    Ok(())
}

pub(crate) fn sync_source_voice<B: Backend>(
    kira: &mut KiraEngine<B>,
    next_playback_id: &mut u64,
    clips: &HashMap<SoundClipId, LoadedClip>,
    voice: &mut SourceVoice,
) -> Result<(), SoundError> {
    if !voice.descriptor.playing {
        return Ok(());
    }
    validate_source_runtime_surface(&voice.descriptor)?;
    if matches!(voice.descriptor.input, SoundSourceInput::Silence) {
        return Ok(());
    }
    if voice
        .kira_playback
        .is_some_and(|playback| kira.contains_playback(playback))
    {
        return Ok(());
    }
    voice.kira_playback = None;
    let clip_id = match &voice.descriptor.input {
        SoundSourceInput::Clip(clip_id) => *clip_id,
        SoundSourceInput::Silence => return Ok(()),
        SoundSourceInput::External(_) | SoundSourceInput::SynthParameter { .. } => {
            return Err(SoundError::UnsupportedAdvancedFeature(
                "source input has no Kira M1 runtime adapter".to_string(),
            ));
        }
    };
    let clip = clips
        .get(&clip_id)
        .ok_or(SoundError::UnknownClip { clip: clip_id })?;
    let settings = playback_settings_for_source(&voice.descriptor);
    let range = playback_range_for_settings(clip, &settings)?;
    voice.cursor_frame = range.start_frame;
    voice.cursor_position = range.start_frame as f64;
    if !kira.is_active() {
        return Ok(());
    }
    let data = static_sound_data(
        &clip.kira_data,
        &settings,
        range.start_frame,
        range.end_frame,
    );
    *next_playback_id = next_playback_id.saturating_add(1);
    let playback = SoundPlaybackId::new(*next_playback_id);
    kira.play(playback, voice.descriptor.output_track, data)?;
    voice.kira_playback = Some(playback);
    Ok(())
}

pub(crate) fn sync_source_voice_in_state(
    state: &mut SoundEngineState,
    voice: &mut SourceVoice,
) -> Result<(), SoundError> {
    let SoundEngineState {
        kira,
        next_playback_id,
        clips,
        ..
    } = state;
    sync_source_voice(kira, next_playback_id, clips, voice)
}

pub(crate) fn stop_bound_source<B: Backend>(
    kira: &mut KiraEngine<B>,
    voice: &mut SourceVoice,
) -> Result<(), SoundError> {
    let Some(playback) = voice.kira_playback.take() else {
        return Ok(());
    };
    if kira.contains_playback(playback) {
        kira.stop(playback)?;
    }
    Ok(())
}

fn playback_settings_for_source(source: &SoundSourceDescriptor) -> SoundPlaybackSettings {
    SoundPlaybackSettings {
        gain: source.gain,
        speed: source.speed,
        looped: source.looped,
        completion_action: source.completion_action,
        paused: !source.playing,
        muted: source.muted,
        start_seconds: source.start_seconds,
        duration_seconds: source.duration_seconds,
        output_track: source.output_track,
        pan: 0.0,
    }
}

fn validate_source_runtime_surface(source: &SoundSourceDescriptor) -> Result<(), SoundError> {
    if !source.playing {
        return Ok(());
    }
    match &source.input {
        SoundSourceInput::External(_) => Err(SoundError::UnsupportedAdvancedFeature(
            "external source playback is enabled by Sound M3".to_string(),
        )),
        SoundSourceInput::SynthParameter { .. } => Err(SoundError::UnsupportedAdvancedFeature(
            "synth source playback is enabled by Sound M3".to_string(),
        )),
        SoundSourceInput::Clip(_) | SoundSourceInput::Silence => Ok(()),
    }
}
