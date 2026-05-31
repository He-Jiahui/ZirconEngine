use zircon_runtime::core::framework::sound::{SoundError, SoundSourceDescriptor, SoundSourceInput};

use crate::engine::SoundEngineState;

pub(super) fn validate_source_clip_range(
    state: &SoundEngineState,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    let SoundSourceInput::Clip(clip_id) = &source.input else {
        return Ok(());
    };
    let Some(duration_seconds) = source.duration_seconds else {
        return Ok(());
    };
    let Some(clip) = state.clips.get(clip_id) else {
        return Ok(());
    };
    let frame_count = clip.asset.frame_count();
    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
    let start_frame = (source.start_seconds.unwrap_or_default() * sample_rate)
        .round()
        .max(0.0) as usize;
    let duration_frames = (duration_seconds * sample_rate).round().max(0.0) as usize;
    let start_frame = start_frame.min(frame_count);
    let end_frame = start_frame.saturating_add(duration_frames).min(frame_count);
    if end_frame <= start_frame {
        return Err(SoundError::InvalidParameter(
            "source duration must cover at least one frame".to_string(),
        ));
    }
    Ok(())
}
