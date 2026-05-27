use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackSettings};

use crate::automation::ensure_finite_value;
use crate::engine::LoadedClip;

pub(super) fn validate_playback_speed(speed: f32) -> Result<f32, SoundError> {
    ensure_finite_value("playback speed", speed)?;
    if speed <= 0.0 {
        return Err(SoundError::InvalidParameter(
            "playback speed must be greater than zero".to_string(),
        ));
    }
    Ok(speed)
}

pub(super) fn validate_playback_settings(
    settings: &SoundPlaybackSettings,
) -> Result<(), SoundError> {
    ensure_finite_value("playback gain", settings.gain)?;
    ensure_finite_value("playback pan", settings.pan)?;
    validate_playback_speed(settings.speed)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PlaybackRange {
    pub(super) start_frame: usize,
    pub(super) end_frame: Option<usize>,
}

pub(super) fn playback_range_for_settings(
    clip: &LoadedClip,
    settings: &SoundPlaybackSettings,
) -> Result<PlaybackRange, SoundError> {
    let frame_count = clip.asset.frame_count();
    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
    let start_frame = seconds_to_frame(
        "playback start seconds",
        settings.start_seconds,
        sample_rate,
    )?
    .unwrap_or_default()
    .min(frame_count);
    let end_frame = seconds_to_frame(
        "playback duration seconds",
        settings.duration_seconds,
        sample_rate,
    )?
    .map(|duration_frames| start_frame.saturating_add(duration_frames).min(frame_count));
    if matches!(end_frame, Some(end) if end <= start_frame) {
        return Err(SoundError::InvalidParameter(
            "playback duration must cover at least one frame".to_string(),
        ));
    }
    Ok(PlaybackRange {
        start_frame,
        end_frame,
    })
}

fn seconds_to_frame(
    label: &str,
    seconds: Option<f32>,
    sample_rate: f32,
) -> Result<Option<usize>, SoundError> {
    let Some(seconds) = seconds else {
        return Ok(None);
    };
    ensure_finite_value(label, seconds)?;
    if seconds < 0.0 {
        return Err(SoundError::InvalidParameter(format!(
            "{label} must be non-negative"
        )));
    }
    Ok(Some((seconds * sample_rate).round() as usize))
}
