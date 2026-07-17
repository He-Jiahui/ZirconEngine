use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use crate::automation::values::ensure_finite_value;
use crate::poison_recovery::lock_recover;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn seek_playback_seconds_impl(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback seek seconds", seconds)?;
        if seconds < 0.0 {
            return Err(SoundError::InvalidParameter(
                "playback seek seconds must be non-negative".to_string(),
            ));
        }
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let (sample_rate, frame_count, range_start_frame, range_end_frame) = {
            let active = state
                .playbacks
                .get(&playback)
                .ok_or(SoundError::UnknownPlayback { playback })?;
            let active_clip = active.clip;
            let clip = state
                .clips
                .get(&active_clip)
                .ok_or(SoundError::UnknownClip { clip: active_clip })?;
            (
                clip.asset.sample_rate_hz.max(1) as f32,
                clip.asset.frame_count(),
                active.range_start_frame,
                active.range_end_frame,
            )
        };
        let requested_frame = (seconds * sample_rate).round() as usize;
        let range_end = range_end_frame.unwrap_or(frame_count).min(frame_count);
        let clamped_frame = requested_frame.max(range_start_frame).min(range_end);
        let kira_position =
            kira_slice_position_for_absolute_frame(clamped_frame, range_start_frame, sample_rate);
        state.kira.seek_to(playback, kira_position)?;
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.cursor_frame = clamped_frame;
        active.cursor_position = clamped_frame as f64;
        Ok(())
    }
}

pub(crate) fn kira_slice_position_for_absolute_frame(
    absolute_frame: usize,
    range_start_frame: usize,
    sample_rate_hz: f32,
) -> f64 {
    absolute_frame.saturating_sub(range_start_frame) as f64 / sample_rate_hz.max(1.0) as f64
}
