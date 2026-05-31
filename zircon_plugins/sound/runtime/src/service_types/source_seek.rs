use zircon_runtime::core::framework::sound::{SoundError, SoundSourceId, SoundSourceInput};

use crate::automation::values::ensure_finite_value;

use super::DefaultSoundManager;

impl DefaultSoundManager {
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
}
