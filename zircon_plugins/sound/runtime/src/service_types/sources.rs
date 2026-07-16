use super::DefaultSoundManager;

use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::sound::{
    SoundError, SoundGameplayEmission, SoundGameplayEmissionRead, SoundSourceDescriptor,
    SoundSourceFinishReason, SoundSourceFinished, SoundSourceId, SoundSourceInput,
    SOUND_GAMEPLAY_EMISSION_CAPACITY,
};

use crate::descriptor_validation::source::validate_source_descriptor;
use crate::engine::SourceVoice;

impl DefaultSoundManager {
    pub(super) fn create_source_impl(
        &self,
        mut source: SoundSourceDescriptor,
    ) -> Result<SoundSourceId, SoundError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        validate_source_descriptor(&state, &source)?;
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
                position: source.position,
                strength: source.gain,
                max_range: (source.spatial.max_distance.is_finite()
                    && source.spatial.max_distance > 0.0)
                    .then_some(source.spatial.max_distance),
                emitted_at_seconds,
            });
        }
        state.sources.insert(
            source_id,
            SourceVoice {
                descriptor: source,
                cursor_frame: 0,
                cursor_position: 0.0,
                pending_finish: None,
            },
        );
        Ok(source_id)
    }

    pub(super) fn read_gameplay_emissions_impl(
        &self,
        world: WorldHandle,
        after_sequence: Option<u64>,
    ) -> Result<SoundGameplayEmissionRead, SoundError> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        validate_source_descriptor(&state, &source)?;
        let voice = state
            .sources
            .get_mut(&source_id)
            .ok_or(SoundError::UnknownSource { source_id })?;
        voice.descriptor = source;
        Ok(())
    }

    pub(super) fn remove_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .sources
            .remove(&source)
            .map(|_| ())
            .ok_or(SoundError::UnknownSource { source_id: source })
    }

    pub(super) fn stop_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let voice = state
            .sources
            .remove(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
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
