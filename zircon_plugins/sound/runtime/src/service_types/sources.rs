use super::playback_validation::validate_playback_speed;
use super::DefaultSoundManager;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished, SoundSourceId,
    SoundSourceInput,
};

use crate::automation::ensure_finite_value;
use crate::descriptor_validation::validate_source_descriptor;
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

    pub(super) fn pause_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = false;
        Ok(())
    }

    pub(super) fn resume_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = true;
        Ok(())
    }

    pub(super) fn toggle_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = !voice.descriptor.playing;
        Ok(())
    }

    pub(super) fn set_source_gain_impl(
        &self,
        source: SoundSourceId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("source gain", gain)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.gain = gain;
        Ok(())
    }

    pub(super) fn set_source_speed_impl(
        &self,
        source: SoundSourceId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.speed = speed;
        Ok(())
    }

    pub(super) fn seek_source_seconds_impl(
        &self,
        source: SoundSourceId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("source seek seconds", seconds)?;
        if seconds < 0.0 {
            return Err(SoundError::InvalidParameter(
                "source seek seconds must be non-negative".to_string(),
            ));
        }
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let clamped_frame = {
            let voice = state
                .sources
                .get(&source)
                .ok_or(SoundError::UnknownSource { source_id: source })?;
            match &voice.descriptor.input {
                SoundSourceInput::Clip(clip_id) => {
                    let clip = state
                        .clips
                        .get(clip_id)
                        .ok_or(SoundError::UnknownClip { clip: *clip_id })?;
                    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
                    let frame_count = clip.asset.frame_count();
                    let start_frame = voice
                        .descriptor
                        .start_seconds
                        .map(|start_seconds| (start_seconds * sample_rate).round() as usize)
                        .unwrap_or_default()
                        .min(frame_count);
                    let range_end = voice
                        .descriptor
                        .duration_seconds
                        .map(|duration_seconds| {
                            let duration_frames =
                                (duration_seconds * sample_rate).round().max(0.0) as usize;
                            start_frame.saturating_add(duration_frames).min(frame_count)
                        })
                        .unwrap_or(frame_count);
                    ((seconds * sample_rate).round() as usize)
                        .max(start_frame)
                        .min(range_end)
                }
                SoundSourceInput::External(handle) => {
                    let block = state.external_sources.get(handle).ok_or_else(|| {
                        SoundError::InvalidParameter(format!(
                            "source seek requires submitted external block for {}",
                            handle.as_str()
                        ))
                    })?;
                    let frame_count = block.samples.len() / block.channel_count.max(1) as usize;
                    ((seconds * block.sample_rate_hz.max(1) as f32).round() as usize)
                        .min(frame_count)
                }
                SoundSourceInput::SynthParameter { .. } | SoundSourceInput::Silence => {
                    if seconds == 0.0 {
                        0
                    } else {
                        return Err(SoundError::InvalidParameter(
                            "source seek requires clip or external input".to_string(),
                        ));
                    }
                }
            }
        };
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.cursor_frame = clamped_frame;
        voice.cursor_position = clamped_frame as f64;
        Ok(())
    }

    pub(super) fn mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = true;
        Ok(())
    }

    pub(super) fn unmute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = false;
        Ok(())
    }

    pub(super) fn toggle_mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = !voice.descriptor.muted;
        Ok(())
    }
}
