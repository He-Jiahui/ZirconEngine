use zircon_runtime::core::framework::sound::{
    SoundError, SoundSourceFinished, SoundSourceId, SoundSourceInput, SoundSourceStatus,
};

use crate::engine::{SoundEngineState, SourceVoice};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn source_empty_impl(&self, source: SoundSourceId) -> Result<bool, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        if state.sources.contains_key(&source) {
            return Ok(false);
        }
        if state
            .finished_sources
            .iter()
            .any(|finished| finished.source == source)
        {
            return Ok(true);
        }
        Err(SoundError::UnknownSource { source_id: source })
    }

    pub(super) fn source_status_impl(
        &self,
        source: SoundSourceId,
    ) -> Result<SoundSourceStatus, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        let (range_start_frame, range_end_frame, cursor_seconds) =
            source_status_range_and_cursor_seconds(&state, voice);
        Ok(SoundSourceStatus {
            source,
            input: voice.descriptor.input.clone(),
            playing: voice.descriptor.playing,
            muted: voice.descriptor.muted,
            looped: voice.descriptor.looped,
            completion_action: voice.descriptor.completion_action,
            gain: voice.descriptor.gain,
            speed: voice.descriptor.speed,
            range_start_frame,
            range_end_frame,
            cursor_frame: voice.cursor_frame,
            cursor_seconds,
            output_track: voice.descriptor.output_track,
        })
    }

    pub(super) fn drain_finished_sources_impl(
        &self,
    ) -> Result<Vec<SoundSourceFinished>, SoundError> {
        Ok(std::mem::take(
            &mut self
                .state
                .lock()
                .expect("sound state mutex poisoned")
                .finished_sources,
        ))
    }
}

fn source_status_range_and_cursor_seconds(
    state: &SoundEngineState,
    voice: &SourceVoice,
) -> (usize, Option<usize>, f32) {
    match &voice.descriptor.input {
        SoundSourceInput::Clip(clip_id) => {
            let Some(clip) = state.clips.get(clip_id) else {
                return (0, None, 0.0);
            };
            let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
            let frame_count = clip.asset.frame_count();
            let start_frame = voice
                .descriptor
                .start_seconds
                .map(|seconds| (seconds * sample_rate).round().max(0.0) as usize)
                .unwrap_or_default()
                .min(frame_count);
            let end_frame = voice.descriptor.duration_seconds.map(|seconds| {
                let duration_frames = (seconds * sample_rate).round().max(0.0) as usize;
                start_frame.saturating_add(duration_frames).min(frame_count)
            });
            (
                start_frame,
                end_frame,
                voice.cursor_position as f32 / sample_rate,
            )
        }
        SoundSourceInput::External(handle) => {
            let sample_rate = state
                .external_sources
                .get(handle)
                .map(|block| block.sample_rate_hz.max(1) as f32)
                .unwrap_or(1.0);
            (0, None, voice.cursor_position as f32 / sample_rate)
        }
        SoundSourceInput::SynthParameter { .. } | SoundSourceInput::Silence => (0, None, 0.0),
    }
}
