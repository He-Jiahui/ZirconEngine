use super::DefaultSoundManager;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished, SoundSourceId,
    SoundSourceInput,
};

use crate::descriptor_validation::source::validate_source_descriptor;
use crate::engine::SourceVoice;

impl DefaultSoundManager {
    pub(super) fn create_source_impl(
        &self,
        mut source: SoundSourceDescriptor,
    ) -> Result<SoundSourceId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
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

    pub(super) fn update_source_impl(
        &self,
        source: SoundSourceDescriptor,
    ) -> Result<(), SoundError> {
        let source_id = source.id.ok_or_else(|| {
            SoundError::InvalidParameter("source update requires a source id".to_string())
        })?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
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
            .expect("sound state mutex poisoned")
            .sources
            .remove(&source)
            .map(|_| ())
            .ok_or(SoundError::UnknownSource { source_id: source })
    }

    pub(super) fn stop_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
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
